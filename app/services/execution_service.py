"""
Execution service layer for business logic
"""
from typing import List, Optional, Dict, Any

from app.repositories.execution_repository import ExecutionRepository
from app.models.execution import AgentExecution
from app.core.constants import AgentExecutionStatus
from app.core.exceptions import (
    ExecutionNotFoundError, ValidationError, ConflictError
)
from app.core.logging import get_logger

logger = get_logger(__name__)


class ExecutionService:
    """Service for execution business logic"""
    
    def __init__(self, execution_repository: Optional[ExecutionRepository] = None):
        self.execution_repo = execution_repository or ExecutionRepository()
    
    def create_execution(self, task_id: int, agent_id: int,
                        metadata: Optional[Dict[str, Any]] = None) -> AgentExecution:
        """Create a new execution"""
        try:
            # Validate input
            if not task_id or task_id <= 0:
                raise ValidationError("Valid task ID is required")
            
            if not agent_id or agent_id <= 0:
                raise ValidationError("Valid agent ID is required")
            
            # Create execution
            execution_data = {
                'task_id': task_id,
                'agent_id': agent_id,
                'status': AgentExecutionStatus.PENDING.value,
                'metadata': metadata or {}
            }
            
            execution = AgentExecution.from_dict(execution_data)
            created_execution = self.execution_repo.create(execution)
            
            logger.info(f"Created execution {created_execution.id} for task {task_id} with agent {agent_id}")
            return created_execution
            
        except Exception as e:
            logger.error(f"Failed to create execution: {str(e)}")
            raise
    
    def get_execution_by_id(self, execution_id: int) -> AgentExecution:
        """Get execution by ID"""
        execution = self.execution_repo.get_by_id(execution_id)
        if not execution:
            raise ExecutionNotFoundError(execution_id)
        return execution
    
    def start_execution(self, execution_id: int) -> AgentExecution:
        """Start an execution"""
        try:
            execution = self.get_execution_by_id(execution_id)
            
            # Validate execution can be started
            if execution.status != AgentExecutionStatus.PENDING.value:
                raise ConflictError(f"Execution {execution_id} is not in pending state")
            
            execution.start_execution()
            updated_execution = self.execution_repo.update(execution)
            
            logger.info(f"Started execution {execution_id}")
            return updated_execution
            
        except Exception as e:
            logger.error(f"Failed to start execution {execution_id}: {str(e)}")
            raise
    
    def complete_execution(self, execution_id: int, result: Optional[str] = None) -> AgentExecution:
        """Complete an execution successfully"""
        try:
            execution = self.get_execution_by_id(execution_id)
            
            # Validate execution can be completed
            if not execution.is_running():
                raise ConflictError(f"Execution {execution_id} is not running")
            
            execution.complete_execution(result)
            updated_execution = self.execution_repo.update(execution)
            
            logger.info(f"Completed execution {execution_id}")
            return updated_execution
            
        except Exception as e:
            logger.error(f"Failed to complete execution {execution_id}: {str(e)}")
            raise
    
    def fail_execution(self, execution_id: int, error_message: Optional[str] = None) -> AgentExecution:
        """Mark an execution as failed"""
        try:
            execution = self.get_execution_by_id(execution_id)
            
            # Validate execution can be failed
            if execution.is_completed():
                raise ConflictError(f"Execution {execution_id} is already completed")
            
            execution.fail_execution(error_message)
            updated_execution = self.execution_repo.update(execution)
            
            logger.info(f"Failed execution {execution_id}: {error_message}")
            return updated_execution
            
        except Exception as e:
            logger.error(f"Failed to mark execution {execution_id} as failed: {str(e)}")
            raise
    
    def cancel_execution(self, execution_id: int) -> AgentExecution:
        """Cancel an execution"""
        try:
            execution = self.get_execution_by_id(execution_id)
            
            # Validate execution can be cancelled
            if not execution.can_be_cancelled():
                raise ConflictError(f"Execution {execution_id} cannot be cancelled")
            
            execution.cancel_execution()
            updated_execution = self.execution_repo.update(execution)
            
            logger.info(f"Cancelled execution {execution_id}")
            return updated_execution
            
        except Exception as e:
            logger.error(f"Failed to cancel execution {execution_id}: {str(e)}")
            raise
    
    def add_log_entry(self, execution_id: int, level: str, message: str) -> AgentExecution:
        """Add a log entry to an execution"""
        try:
            execution = self.get_execution_by_id(execution_id)
            
            execution.add_log_entry(level, message)
            updated_execution = self.execution_repo.update(execution)
            
            return updated_execution
            
        except Exception as e:
            logger.error(f"Failed to add log entry to execution {execution_id}: {str(e)}")
            raise
    
    def update_execution_metrics(self, execution_id: int,
                                tokens_used: Optional[int] = None,
                                cost: Optional[float] = None) -> AgentExecution:
        """Update execution metrics"""
        try:
            execution = self.get_execution_by_id(execution_id)
            
            execution.increment_llm_requests(tokens_used, cost)
            updated_execution = self.execution_repo.update(execution)
            
            return updated_execution
            
        except Exception as e:
            logger.error(f"Failed to update execution metrics {execution_id}: {str(e)}")
            raise
    
    def get_executions_by_task(self, task_id: int,
                              limit: Optional[int] = None,
                              offset: Optional[int] = None) -> List[AgentExecution]:
        """Get executions for a specific task"""
        return self.execution_repo.get_by_task_id(task_id, limit, offset)
    
    def get_executions_by_agent(self, agent_id: int,
                               limit: Optional[int] = None,
                               offset: Optional[int] = None) -> List[AgentExecution]:
        """Get executions for a specific agent"""
        return self.execution_repo.get_by_agent_id(agent_id, limit, offset)
    
    def get_running_executions(self) -> List[AgentExecution]:
        """Get all currently running executions"""
        return self.execution_repo.get_running_executions()
    
    def get_pending_executions(self) -> List[AgentExecution]:
        """Get all pending executions"""
        return self.execution_repo.get_pending_executions()
    
    def get_execution_statistics(self) -> Dict[str, Any]:
        """Get comprehensive execution statistics"""
        return self.execution_repo.get_execution_statistics()
    
    def get_agent_execution_statistics(self, agent_id: int) -> Dict[str, Any]:
        """Get execution statistics for a specific agent"""
        return self.execution_repo.get_agent_execution_statistics(agent_id)
    
    def get_task_execution_statistics(self, task_id: int) -> Dict[str, Any]:
        """Get execution statistics for a specific task"""
        return self.execution_repo.get_task_execution_statistics(task_id)
    
    def get_long_running_executions(self, threshold_minutes: int = 30) -> List[AgentExecution]:
        """Get executions that have been running longer than threshold"""
        return self.execution_repo.get_long_running_executions(threshold_minutes)
    
    def cleanup_old_executions(self, days_to_keep: int = 90) -> int:
        """Clean up old completed/failed executions"""
        try:
            count = self.execution_repo.cleanup_old_executions(days_to_keep)
            logger.info(f"Cleaned up {count} old executions")
            return count
            
        except Exception as e:
            logger.error(f"Failed to cleanup old executions: {str(e)}")
            raise
    
    def get_performance_metrics(self, time_period_hours: int = 24) -> Dict[str, Any]:
        """Get performance metrics for a time period"""
        return self.execution_repo.get_performance_metrics(time_period_hours)
