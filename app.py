from flask import Flask, render_template, request, redirect, url_for, flash, jsonify
from flask_migrate import Migrate
from datetime import datetime
import os
import logging
from dotenv import load_dotenv
from db import db

# Load environment variables from .env file
load_dotenv()

app = Flask(__name__)
app.config['SECRET_KEY'] = os.environ.get('SECRET_KEY', 'dev-secret-key')
app.config['SQLALCHEMY_DATABASE_URI'] = os.environ.get('DATABASE_URL', 'postgresql://kanban_user:kanban_pass@localhost:5432/kanban_db')
app.config['SQLALCHEMY_TRACK_MODIFICATIONS'] = False

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

db.init_app(app)
migrate = Migrate(app, db)

from models import Task, Agent, AgentExecution, Tool
from llm_providers import LLMProviderFactory
from tools import tool_registry
from celery_app import execute_agent_task_async

# Routes
@app.route('/')
def index():
    todo_tasks = Task.query.filter_by(status='todo').order_by(Task.created_at.desc()).all()
    in_progress_tasks = Task.query.filter_by(status='in_progress').order_by(Task.created_at.desc()).all()
    done_tasks = Task.query.filter_by(status='done').order_by(Task.created_at.desc()).all()
    
    return render_template('index.html', 
                         todo_tasks=todo_tasks,
                         in_progress_tasks=in_progress_tasks,
                         done_tasks=done_tasks)

@app.route('/add_task', methods=['POST'])
def add_task():
    title = request.form.get('title')
    description = request.form.get('description')
    
    if not title:
        flash('Task title is required!', 'error')
        return redirect(url_for('index'))
    
    task = Task(title=title, description=description, status='todo')
    db.session.add(task)
    db.session.commit()
    
    flash('Task added successfully!', 'success')
    return redirect(url_for('index'))

@app.route('/move_task/<int:task_id>/<status>')
def move_task(task_id, status):
    if status not in ['todo', 'in_progress', 'done']:
        flash('Invalid status!', 'error')
        return redirect(url_for('index'))
    
    task = Task.query.get_or_404(task_id)
    task.status = status
    task.updated_at = datetime.utcnow()
    db.session.commit()
    
    flash(f'Task moved to {status.replace("_", " ").title()}!', 'success')
    return redirect(url_for('index'))

@app.route('/delete_task/<int:task_id>')
def delete_task(task_id):
    task = Task.query.get_or_404(task_id)
    db.session.delete(task)
    db.session.commit()
    
    flash('Task deleted successfully!', 'success')
    return redirect(url_for('index'))

@app.route('/edit_task/<int:task_id>', methods=['GET', 'POST'])
def edit_task(task_id):
    task = Task.query.get_or_404(task_id)
    
    if request.method == 'POST':
        task.title = request.form.get('title')
        task.description = request.form.get('description')
        task.updated_at = datetime.utcnow()
        db.session.commit()
        
        flash('Task updated successfully!', 'success')
        return redirect(url_for('index'))
    
    return render_template('edit_task.html', task=task)

# UI Routes for Agent Management

@app.route('/agents')
def agents():
    """Agents management page"""
    return render_template('agents.html')

@app.route('/executions')
def executions():
    """Agent executions monitoring page"""
    return render_template('executions.html')

# API Routes for Agent Management

@app.route('/api/agents', methods=['GET', 'POST'])
def api_agents():
    if request.method == 'GET':
        # List all agents
        agents = Agent.query.all()
        return jsonify([{
            'id': agent.id,
            'name': agent.name,
            'description': agent.description,
            'llm_provider': agent.llm_provider,
            'llm_model': agent.llm_model,
            'available_tools': agent.available_tools,
            'is_active': agent.is_active,
            'created_at': agent.created_at.isoformat() if agent.created_at else None
        } for agent in agents])
    
    elif request.method == 'POST':
        # Create new agent
        try:
            data = request.get_json()
            
            # Validate required fields
            required_fields = ['name', 'system_instructions', 'llm_provider', 'llm_model']
            for field in required_fields:
                if not data.get(field):
                    return jsonify({'error': f'{field} is required'}), 400
            
            # Validate LLM provider
            available_providers = LLMProviderFactory.get_available_providers()
            if data['llm_provider'] not in available_providers:
                return jsonify({'error': f'Invalid LLM provider. Available: {available_providers}'}), 400
            
            # Validate tools
            available_tools = tool_registry.get_available_tools()
            requested_tools = data.get('available_tools', [])
            invalid_tools = [tool for tool in requested_tools if tool not in available_tools]
            if invalid_tools:
                return jsonify({'error': f'Invalid tools: {invalid_tools}. Available: {available_tools}'}), 400
            
            # Create agent
            agent = Agent(
                name=data['name'],
                description=data.get('description', ''),
                system_instructions=data['system_instructions'],
                llm_provider=data['llm_provider'],
                llm_model=data['llm_model'],
                llm_api_key=data.get('llm_api_key'),
                available_tools=requested_tools,
                config=data.get('config', {}),
                is_active=data.get('is_active', True)
            )
            
            db.session.add(agent)
            db.session.commit()
            
            logger.info(f"Created agent {agent.id}: {agent.name}")
            
            return jsonify({
                'id': agent.id,
                'name': agent.name,
                'message': 'Agent created successfully'
            }), 201
            
        except Exception as e:
            logger.error(f"Error creating agent: {str(e)}")
            return jsonify({'error': str(e)}), 500

@app.route('/api/agents/<int:agent_id>', methods=['GET', 'PUT', 'DELETE'])
def api_agent(agent_id):
    agent = Agent.query.get_or_404(agent_id)
    
    if request.method == 'GET':
        # Get agent details
        return jsonify({
            'id': agent.id,
            'name': agent.name,
            'description': agent.description,
            'system_instructions': agent.system_instructions,
            'llm_provider': agent.llm_provider,
            'llm_model': agent.llm_model,
            'available_tools': agent.available_tools,
            'config': agent.config,
            'is_active': agent.is_active,
            'created_at': agent.created_at.isoformat() if agent.created_at else None,
            'updated_at': agent.updated_at.isoformat() if agent.updated_at else None
        })
    
    elif request.method == 'PUT':
        # Update agent
        try:
            data = request.get_json()
            
            # Update fields
            if 'name' in data:
                agent.name = data['name']
            if 'description' in data:
                agent.description = data['description']
            if 'system_instructions' in data:
                agent.system_instructions = data['system_instructions']
            if 'llm_provider' in data:
                available_providers = LLMProviderFactory.get_available_providers()
                if data['llm_provider'] not in available_providers:
                    return jsonify({'error': f'Invalid LLM provider. Available: {available_providers}'}), 400
                agent.llm_provider = data['llm_provider']
            if 'llm_model' in data:
                agent.llm_model = data['llm_model']
            if 'llm_api_key' in data:
                agent.llm_api_key = data['llm_api_key']
            if 'available_tools' in data:
                available_tools = tool_registry.get_available_tools()
                requested_tools = data['available_tools']
                invalid_tools = [tool for tool in requested_tools if tool not in available_tools]
                if invalid_tools:
                    return jsonify({'error': f'Invalid tools: {invalid_tools}. Available: {available_tools}'}), 400
                agent.available_tools = requested_tools
            if 'config' in data:
                agent.config = data['config']
            if 'is_active' in data:
                agent.is_active = data['is_active']
            
            agent.updated_at = datetime.utcnow()
            db.session.commit()
            
            logger.info(f"Updated agent {agent.id}: {agent.name}")
            
            return jsonify({'message': 'Agent updated successfully'})
            
        except Exception as e:
            logger.error(f"Error updating agent: {str(e)}")
            return jsonify({'error': str(e)}), 500
    
    elif request.method == 'DELETE':
        # Delete agent
        try:
            # Check if agent has running executions
            running_executions = AgentExecution.query.filter_by(
                agent_id=agent_id,
                status='running'
            ).count()
            
            if running_executions > 0:
                return jsonify({'error': 'Cannot delete agent with running executions'}), 400
            
            # Unassign tasks
            assigned_tasks = Task.query.filter_by(assigned_agent_id=agent_id).all()
            for task in assigned_tasks:
                task.assigned_agent_id = None
                task.execution_status = 'manual'
            
            db.session.delete(agent)
            db.session.commit()
            
            logger.info(f"Deleted agent {agent_id}")
            
            return jsonify({'message': 'Agent deleted successfully'})
            
        except Exception as e:
            logger.error(f"Error deleting agent: {str(e)}")
            return jsonify({'error': str(e)}), 500

@app.route('/api/tasks/<int:task_id>/assign/<int:agent_id>', methods=['POST'])
def api_assign_task(task_id, agent_id):
    """Assign a task to an agent"""
    try:
        task = Task.query.get_or_404(task_id)
        agent = Agent.query.get_or_404(agent_id)
        
        if not agent.is_active:
            return jsonify({'error': 'Agent is not active'}), 400
        
        # Check if task is already running
        if task.execution_status == 'running':
            return jsonify({'error': 'Task is currently running'}), 400
        
        # Assign task to agent
        task.assigned_agent_id = agent_id
        task.execution_status = 'assigned'
        db.session.commit()
        
        logger.info(f"Assigned task {task_id} to agent {agent_id}")
        
        return jsonify({
            'message': f'Task "{task.title}" assigned to agent "{agent.name}"',
            'task_id': task_id,
            'agent_id': agent_id
        })
        
    except Exception as e:
        logger.error(f"Error assigning task: {str(e)}")
        return jsonify({'error': str(e)}), 500

@app.route('/api/tasks/<int:task_id>/unassign', methods=['POST'])
def api_unassign_task(task_id):
    """Unassign a task from its agent"""
    try:
        task = Task.query.get_or_404(task_id)
        
        if task.execution_status == 'running':
            return jsonify({'error': 'Cannot unassign running task'}), 400
        
        task.assigned_agent_id = None
        task.execution_status = 'manual'
        db.session.commit()
        
        logger.info(f"Unassigned task {task_id}")
        
        return jsonify({'message': f'Task "{task.title}" unassigned'})
        
    except Exception as e:
        logger.error(f"Error unassigning task: {str(e)}")
        return jsonify({'error': str(e)}), 500

@app.route('/api/executions', methods=['GET'])
def api_executions():
    """List agent executions"""
    try:
        page = request.args.get('page', 1, type=int)
        per_page = min(request.args.get('per_page', 20, type=int), 100)
        
        executions = AgentExecution.query.order_by(
            AgentExecution.created_at.desc()
        ).paginate(
            page=page, per_page=per_page, error_out=False
        )
        
        return jsonify({
            'executions': [{
                'id': exec.id,
                'task_id': exec.task_id,
                'task_title': exec.task.title if exec.task else 'Unknown',
                'agent_id': exec.agent_id,
                'agent_name': exec.agent.name if exec.agent else 'Unknown',
                'status': exec.status,
                'iterations_count': exec.iterations_count,
                'tokens_used': exec.tokens_used,
                'execution_time_seconds': exec.execution_time_seconds,
                'started_at': exec.started_at.isoformat() if exec.started_at else None,
                'completed_at': exec.completed_at.isoformat() if exec.completed_at else None,
                'error_message': exec.error_message
            } for exec in executions.items],
            'pagination': {
                'page': page,
                'per_page': per_page,
                'total': executions.total,
                'pages': executions.pages,
                'has_next': executions.has_next,
                'has_prev': executions.has_prev
            }
        })
        
    except Exception as e:
        logger.error(f"Error listing executions: {str(e)}")
        return jsonify({'error': str(e)}), 500

@app.route('/api/executions/<int:execution_id>', methods=['GET'])
def api_execution(execution_id):
    """Get execution details"""
    try:
        execution = AgentExecution.query.get_or_404(execution_id)
        
        return jsonify({
            'id': execution.id,
            'task_id': execution.task_id,
            'task_title': execution.task.title if execution.task else 'Unknown',
            'agent_id': execution.agent_id,
            'agent_name': execution.agent.name if execution.agent else 'Unknown',
            'status': execution.status,
            'conversation_log': execution.conversation_log,
            'result': execution.result,
            'error_message': execution.error_message,
            'iterations_count': execution.iterations_count,
            'tokens_used': execution.tokens_used,
            'execution_time_seconds': execution.execution_time_seconds,
            'started_at': execution.started_at.isoformat() if execution.started_at else None,
            'completed_at': execution.completed_at.isoformat() if execution.completed_at else None,
            'created_at': execution.created_at.isoformat() if execution.created_at else None
        })
        
    except Exception as e:
        logger.error(f"Error getting execution: {str(e)}")
        return jsonify({'error': str(e)}), 500

@app.route('/api/tools', methods=['GET'])
def api_tools():
    """List available tools"""
    try:
        tools = tool_registry.get_available_tools()
        tool_details = []
        
        for tool_name in tools:
            tool = tool_registry.get_tool(tool_name)
            if tool:
                tool_details.append({
                    'name': tool.name,
                    'description': tool.description,
                    'input_schema': tool.input_schema
                })
        
        return jsonify({
            'tools': tool_details,
            'count': len(tool_details)
        })
        
    except Exception as e:
        logger.error(f"Error listing tools: {str(e)}")
        return jsonify({'error': str(e)}), 500

@app.route('/api/providers', methods=['GET'])
def api_providers():
    """List available LLM providers"""
    try:
        providers = LLMProviderFactory.get_available_providers()
        default_models = LLMProviderFactory.get_default_models()
        
        return jsonify({
            'providers': [{
                'name': provider,
                'default_model': default_models.get(provider, 'unknown')
            } for provider in providers]
        })
        
    except Exception as e:
        logger.error(f"Error listing providers: {str(e)}")
        return jsonify({'error': str(e)}), 500

if __name__ == '__main__':
    with app.app_context():
        db.create_all()
    app.run(host='0.0.0.0', port=5000, debug=True)
