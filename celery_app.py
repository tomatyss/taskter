"""
Celery configuration for background task processing
"""
import os
from celery import Celery
from celery.schedules import crontab
from dotenv import load_dotenv

# Load environment variables from .env file
load_dotenv()

# Create Celery app
celery_app = Celery('taskter')

# Configuration
celery_app.conf.update(
    broker_url=os.getenv('REDIS_URL', 'redis://localhost:6379/0'),
    result_backend=os.getenv('REDIS_URL', 'redis://localhost:6379/0'),
    task_serializer='json',
    accept_content=['json'],
    result_serializer='json',
    timezone='UTC',
    enable_utc=True,
    task_track_started=True,
    task_time_limit=30 * 60,  # 30 minutes
    task_soft_time_limit=25 * 60,  # 25 minutes
    worker_prefetch_multiplier=1,
    worker_max_tasks_per_child=1000,
)

# Beat schedule for periodic tasks
celery_app.conf.beat_schedule = {
    'check-pending-agent-tasks': {
        'task': 'celery_app.check_pending_agent_tasks',
        'schedule': 30.0,  # Every 30 seconds
    },
    'cleanup-old-executions': {
        'task': 'celery_app.cleanup_old_executions',
        'schedule': crontab(hour=2, minute=0),  # Daily at 2 AM
    },
}

# Import Flask app context for database operations
def create_app_context():
    """Create Flask app context for Celery tasks"""
    import os
    import sys
    from flask import Flask
    
    # Add current directory to Python path
    sys.path.insert(0, '/app')
    
    # Import the existing db instance and models
    from db import db
    from models import Task, Agent, AgentExecution, Tool
    
    # Create a minimal Flask app for Celery tasks
    app = Flask(__name__)
    app.config['SECRET_KEY'] = os.environ.get('SECRET_KEY', 'dev-secret-key')
    app.config['SQLALCHEMY_DATABASE_URI'] = os.environ.get('DATABASE_URL', 'postgresql://kanban_user:kanban_pass@localhost:5432/kanban_db')
    app.config['SQLALCHEMY_TRACK_MODIFICATIONS'] = False
    
    # Initialize the existing db instance with this app
    db.init_app(app)
    
    return app.app_context()

@celery_app.task(bind=True, autoretry_for=(Exception,), retry_kwargs={'max_retries': 3, 'countdown': 60})
def execute_agent_task_async(self, task_id, agent_id):
    """
    Celery task to execute an agent task asynchronously
    
    Args:
        task_id: ID of the task to execute
        agent_id: ID of the agent to use
    
    Retry configuration:
        - Retries up to 3 times on any exception
        - Waits 60 seconds between retries
        - Exponential backoff for API rate limits
    """
    with create_app_context():
        from agent_executor import agent_executor
        import logging
        
        logger = logging.getLogger(__name__)
        logger.info(f"Starting async execution: Task {task_id} with Agent {agent_id}")
        
        try:
            # Update task to show it's being processed
            self.update_state(
                state='PROGRESS',
                meta={'status': 'Starting agent execution...'}
            )
            
            # Execute the agent task
            result = agent_executor.execute_agent_task(task_id, agent_id)
            
            if result['success']:
                logger.info(f"Successfully completed async execution: Task {task_id}")
                return {
                    'status': 'completed',
                    'result': result['result'],
                    'iterations': result.get('iterations', 0),
                    'tokens_used': result.get('total_tokens', 0)
                }
            else:
                logger.error(f"Failed async execution: Task {task_id} - {result.get('error', 'Unknown error')}")
                return {
                    'status': 'failed',
                    'error': result.get('error', 'Unknown error'),
                    'iterations': result.get('iterations', 0),
                    'tokens_used': result.get('total_tokens', 0)
                }
                
        except Exception as e:
            logger.error(f"Exception in async execution: Task {task_id} - {str(e)}")
            
            # Check if this is a retryable error
            error_str = str(e).lower()
            retryable_errors = [
                'rate limit',
                'quota exceeded',
                'timeout',
                'connection error',
                'service unavailable',
                'internal server error',
                'bad gateway',
                'gateway timeout'
            ]
            
            is_retryable = any(error in error_str for error in retryable_errors)
            
            if is_retryable and self.request.retries < self.max_retries:
                logger.warning(f"Retryable error for task {task_id}, attempt {self.request.retries + 1}/{self.max_retries + 1}: {str(e)}")
                # Exponential backoff: 60s, 120s, 240s
                countdown = 60 * (2 ** self.request.retries)
                raise self.retry(countdown=countdown, exc=e)
            
            return {
                'status': 'failed',
                'error': str(e),
                'iterations': 0,
                'tokens_used': 0
            }

@celery_app.task
def check_pending_agent_tasks():
    """
    Periodic task to check for pending agent tasks and start execution
    """
    with create_app_context():
        from models import Task, Agent, AgentExecution
        from db import db
        import logging
        
        logger = logging.getLogger(__name__)
        
        try:
            # Find tasks assigned to agents that aren't currently running
            pending_tasks = db.session.query(Task).filter(
                Task.assigned_agent_id.isnot(None),
                Task.execution_status == 'assigned'
            ).all()
            
            if not pending_tasks:
                return {"message": "No pending agent tasks found"}
            
            started_count = 0
            
            for task in pending_tasks:
                # Check if agent is active
                agent = Agent.query.get(task.assigned_agent_id)
                if not agent or not agent.is_active:
                    logger.warning(f"Skipping task {task.id}: Agent {task.assigned_agent_id} not active")
                    continue
                
                # Check if there's already a running execution for this task
                running_execution = AgentExecution.query.filter_by(
                    task_id=task.id,
                    status='running'
                ).first()
                
                if running_execution:
                    logger.info(f"Task {task.id} already has running execution {running_execution.id}")
                    continue
                
                # Start async execution
                logger.info(f"Starting async execution for task {task.id} with agent {task.assigned_agent_id}")
                execute_agent_task_async.delay(task.id, task.assigned_agent_id)
                started_count += 1
            
            return {"message": f"Started {started_count} agent executions"}
            
        except Exception as e:
            logger.error(f"Error checking pending tasks: {str(e)}")
            return {"error": str(e)}

@celery_app.task
def cleanup_old_executions():
    """
    Periodic task to cleanup old execution records
    """
    with create_app_context():
        from models import AgentExecution
        from db import db
        from datetime import datetime, timedelta
        import logging
        
        logger = logging.getLogger(__name__)
        
        try:
            # Delete executions older than 30 days
            cutoff_date = datetime.utcnow() - timedelta(days=30)
            
            old_executions = AgentExecution.query.filter(
                AgentExecution.created_at < cutoff_date,
                AgentExecution.status.in_(['completed', 'failed', 'stopped'])
            ).all()
            
            deleted_count = len(old_executions)
            
            for execution in old_executions:
                db.session.delete(execution)
            
            db.session.commit()
            
            logger.info(f"Cleaned up {deleted_count} old execution records")
            return {"message": f"Cleaned up {deleted_count} old execution records"}
            
        except Exception as e:
            logger.error(f"Error cleaning up executions: {str(e)}")
            return {"error": str(e)}

@celery_app.task
def test_celery():
    """Test task to verify Celery is working"""
    from datetime import datetime
    return {"message": "Celery is working!", "timestamp": str(datetime.utcnow())}

if __name__ == '__main__':
    celery_app.start()
