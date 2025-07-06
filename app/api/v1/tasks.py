"""
Task management API endpoints.
"""

from flask import Blueprint, request
from app.api.response import APIResponse, handle_service_exceptions, validate_json_input
from app.services.task_service import TaskService
from app.services.agent_service import AgentService
from app.schemas import (
    TaskCreate, TaskUpdate, TaskStatusUpdate,
    task_to_response, TaskAssignmentSchema
)
from app.core.exceptions import TaskNotFoundError, AgentNotFoundError, AgentNotActiveError
from app.core.constants import TaskStatus
from app.core.logging import get_logger

# Create blueprint
tasks_bp = Blueprint('tasks', __name__, url_prefix='/api/v1/tasks')
logger = get_logger(__name__)

# Initialize services (these will be injected via dependency injection in the future)
task_service = TaskService()
agent_service = AgentService()


@tasks_bp.route('', methods=['GET'])
@handle_service_exceptions
def list_tasks():
    """List all tasks with optional filtering"""
    try:
        # Get query parameters
        status = request.args.get('status')
        page = int(request.args.get('page', 1))
        per_page = min(int(request.args.get('per_page', 20)), 100)
        
        # Validate status if provided
        if status and status not in [s.value for s in TaskStatus]:
            return APIResponse.error(
                message=f"Invalid status. Valid options: {[s.value for s in TaskStatus]}",
                error_code="INVALID_STATUS"
            )
        
        # Get tasks by status or all tasks
        if status:
            status_enum = TaskStatus(status)
            tasks = task_service.get_tasks_by_status(status_enum)
        else:
            # Get all tasks from repository directly
            tasks = task_service.task_repo.get_all()
        
        # Convert to response format
        tasks_data = [task_to_response(task) for task in tasks]
        
        # Simple pagination (for now)
        start_idx = (page - 1) * per_page
        end_idx = start_idx + per_page
        paginated_tasks = tasks[start_idx:end_idx]
        
        response_data = {
            "tasks": [task.dict() for task in [task_to_response(task) for task in paginated_tasks]],
            "pagination": {
                "page": page,
                "per_page": per_page,
                "total": len(tasks),
                "pages": (len(tasks) + per_page - 1) // per_page,
                "has_next": end_idx < len(tasks),
                "has_prev": page > 1
            }
        }
        
        return APIResponse.success(data=response_data)
        
    except Exception as e:
        logger.error(f"Error listing tasks: {str(e)}")
        return APIResponse.internal_error()


@tasks_bp.route('', methods=['POST'])
@validate_json_input(TaskCreate)
@handle_service_exceptions
def create_task(data: TaskCreate):
    """Create a new task"""
    try:
        task = task_service.create_task(data.title, data.description)
        task_data = task_to_response(task)
        
        logger.info(f"Created task {task.id}: {task.title}")
        
        return APIResponse.created(
            data=task_data.dict(),
            message="Task created successfully"
        )
        
    except Exception as e:
        logger.error(f"Error creating task: {str(e)}")
        return APIResponse.internal_error()


@tasks_bp.route('/<int:task_id>', methods=['GET'])
@handle_service_exceptions
def get_task(task_id: int):
    """Get a specific task by ID"""
    try:
        task = task_service.get_task_by_id(task_id)
        if not task:
            return APIResponse.not_found("Task")
        
        task_data = task_to_response(task)
        return APIResponse.success(data=task_data.dict())
        
    except TaskNotFoundError:
        return APIResponse.not_found("Task")
    except Exception as e:
        logger.error(f"Error getting task {task_id}: {str(e)}")
        return APIResponse.internal_error()


@tasks_bp.route('/<int:task_id>', methods=['PUT'])
@validate_json_input(TaskUpdate)
@handle_service_exceptions
def update_task(data: TaskUpdate, task_id: int):
    """Update a specific task"""
    try:
        task = task_service.update_task(task_id, data.title, data.description)
        task_data = task_to_response(task)
        
        logger.info(f"Updated task {task_id}")
        
        return APIResponse.success(
            data=task_data.dict(),
            message="Task updated successfully"
        )
        
    except TaskNotFoundError:
        return APIResponse.not_found("Task")
    except Exception as e:
        logger.error(f"Error updating task {task_id}: {str(e)}")
        return APIResponse.internal_error()


@tasks_bp.route('/<int:task_id>', methods=['DELETE'])
@handle_service_exceptions
def delete_task(task_id: int):
    """Delete a specific task"""
    try:
        success = task_service.delete_task(task_id)
        if not success:
            return APIResponse.not_found("Task")
        
        logger.info(f"Deleted task {task_id}")
        
        return APIResponse.success(message="Task deleted successfully")
        
    except TaskNotFoundError:
        return APIResponse.not_found("Task")
    except Exception as e:
        logger.error(f"Error deleting task {task_id}: {str(e)}")
        return APIResponse.internal_error()


@tasks_bp.route('/<int:task_id>/status', methods=['PUT'])
@validate_json_input(TaskStatusUpdate)
@handle_service_exceptions
def update_task_status(data: TaskStatusUpdate, task_id: int):
    """Update task status"""
    try:
        status_enum = TaskStatus(data.status)
        task = task_service.move_task_to_status(task_id, status_enum)
        task_data = task_to_response(task)
        
        logger.info(f"Updated task {task_id} status to {data.status}")
        
        return APIResponse.success(
            data=task_data.dict(),
            message=f"Task status updated to {data.status}"
        )
        
    except TaskNotFoundError:
        return APIResponse.not_found("Task")
    except Exception as e:
        logger.error(f"Error updating task {task_id} status: {str(e)}")
        return APIResponse.internal_error()


@tasks_bp.route('/<int:task_id>/assign/<int:agent_id>', methods=['POST'])
@handle_service_exceptions
def assign_task_simple(task_id: int, agent_id: int):
    """Assign a task to an agent (simple URL-based endpoint)"""
    try:
        # Get the task and agent
        task = task_service.get_task_by_id(task_id)
        if not task:
            return APIResponse.not_found("Task")
        
        agent = agent_service.get_agent_by_id(agent_id)
        if not agent:
            return APIResponse.not_found("Agent")
        
        if not agent.is_active:
            return APIResponse.error(
                message="Agent is not active",
                error_code="AGENT_NOT_ACTIVE"
            )
        
        # Check if task is already running
        if task.execution_status == 'running':
            return APIResponse.error(
                message="Task is currently running",
                error_code="TASK_RUNNING"
            )
        
        # Assign the task
        updated_task = task_service.assign_task_to_agent(task_id, agent_id)
        task_data = task_to_response(updated_task)
        
        logger.info(f"Assigned task {task_id} to agent {agent_id}")
        
        return APIResponse.success(
            data=task_data.dict(),
            message=f'Task "{task.title}" assigned to agent "{agent.name}"'
        )
        
    except TaskNotFoundError:
        return APIResponse.not_found("Task")
    except AgentNotFoundError:
        return APIResponse.not_found("Agent")
    except AgentNotActiveError as e:
        return APIResponse.error(str(e), "AGENT_NOT_ACTIVE")
    except Exception as e:
        logger.error(f"Error assigning task {task_id}: {str(e)}")
        return APIResponse.internal_error()


@tasks_bp.route('/<int:task_id>/assign', methods=['POST'])
@validate_json_input(TaskAssignmentSchema)
@handle_service_exceptions
def assign_task(data: TaskAssignmentSchema, task_id: int):
    """Assign a task to an agent"""
    try:
        # Get the task and agent
        task = task_service.get_task_by_id(task_id)
        if not task:
            return APIResponse.not_found("Task")
        
        agent = agent_service.get_agent_by_id(data.agent_id)
        if not agent:
            return APIResponse.not_found("Agent")
        
        if not agent.is_active:
            return APIResponse.error(
                message="Agent is not active",
                error_code="AGENT_NOT_ACTIVE"
            )
        
        # Check if task is already running
        if task.execution_status == 'running':
            return APIResponse.error(
                message="Task is currently running",
                error_code="TASK_RUNNING"
            )
        
        # Assign the task
        updated_task = task_service.assign_task_to_agent(task_id, data.agent_id)
        task_data = task_to_response(updated_task)
        
        logger.info(f"Assigned task {task_id} to agent {data.agent_id}")
        
        return APIResponse.success(
            data=task_data.dict(),
            message=f'Task "{task.title}" assigned to agent "{agent.name}"'
        )
        
    except TaskNotFoundError:
        return APIResponse.not_found("Task")
    except AgentNotFoundError:
        return APIResponse.not_found("Agent")
    except AgentNotActiveError as e:
        return APIResponse.error(str(e), "AGENT_NOT_ACTIVE")
    except Exception as e:
        logger.error(f"Error assigning task {task_id}: {str(e)}")
        return APIResponse.internal_error()


@tasks_bp.route('/<int:task_id>/unassign', methods=['POST'])
@handle_service_exceptions
def unassign_task(task_id: int):
    """Unassign a task from its agent"""
    try:
        task = task_service.get_task_by_id(task_id)
        if not task:
            return APIResponse.not_found("Task")
        
        if task.execution_status == 'running':
            return APIResponse.error(
                message="Cannot unassign running task",
                error_code="TASK_RUNNING"
            )
        
        # Unassign the task
        updated_task = task_service.unassign_task_from_agent(task_id)
        task_data = task_to_response(updated_task)
        
        logger.info(f"Unassigned task {task_id}")
        
        return APIResponse.success(
            data=task_data.dict(),
            message=f'Task "{task.title}" unassigned'
        )
        
    except TaskNotFoundError:
        return APIResponse.not_found("Task")
    except Exception as e:
        logger.error(f"Error unassigning task {task_id}: {str(e)}")
        return APIResponse.internal_error()


@tasks_bp.route('/stats', methods=['GET'])
@handle_service_exceptions
def get_task_stats():
    """Get task statistics"""
    try:
        stats = task_service.get_task_statistics()
        return APIResponse.success(data=stats)
        
    except Exception as e:
        logger.error(f"Error getting task stats: {str(e)}")
        return APIResponse.internal_error()
