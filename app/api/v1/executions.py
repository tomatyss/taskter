"""
Execution management API endpoints.
"""

from flask import Blueprint, request
from app.api.response import APIResponse, handle_service_exceptions
from app.services.execution_service import ExecutionService
from app.schemas.execution_schemas import (
    ExecutionQuerySchema, execution_to_response_schema, execution_to_list_schema
)
from app.core.exceptions import ExecutionNotFoundError
from app.core.logging import get_logger

# Create blueprint
executions_bp = Blueprint('executions', __name__, url_prefix='/api/v1/executions')
logger = get_logger(__name__)

# Initialize services (these will be injected via dependency injection in the future)
execution_service = ExecutionService()


@executions_bp.route('', methods=['GET'])
@handle_service_exceptions
def list_executions():
    """List agent executions with filtering and pagination"""
    try:
        # Get and validate query parameters
        page = int(request.args.get('page', 1))
        per_page = min(int(request.args.get('per_page', 20)), 100)
        status = request.args.get('status')
        agent_id = request.args.get('agent_id', type=int)
        task_id = request.args.get('task_id', type=int)
        
        # Build query schema
        query_data = {
            'page': page,
            'per_page': per_page
        }
        
        if status:
            query_data['status'] = status
        if agent_id:
            query_data['agent_id'] = agent_id
        if task_id:
            query_data['task_id'] = task_id
        
        # Validate query parameters
        query = ExecutionQuerySchema(**query_data)
        
        # Build filters
        filters = {}
        if query.status:
            filters['status'] = query.status
        if query.agent_id:
            filters['agent_id'] = query.agent_id
        if query.task_id:
            filters['task_id'] = query.task_id
        
        # Get executions based on filters
        if query.agent_id:
            executions = execution_service.get_executions_by_agent(query.agent_id)
        elif query.task_id:
            executions = execution_service.get_executions_by_task(query.task_id)
        else:
            # Get all executions from repository directly
            executions = execution_service.execution_repo.get_all()
        
        # Filter by status if specified
        if query.status:
            executions = [e for e in executions if e.status == query.status]
        
        # Convert to response format
        executions_data = [execution_to_list_schema(execution) for execution in executions]
        
        # Simple pagination
        start_idx = (query.page - 1) * query.per_page
        end_idx = start_idx + query.per_page
        paginated_executions = executions[start_idx:end_idx]
        
        response_data = {
            "executions": [execution.dict() for execution in [execution_to_list_schema(execution) for execution in paginated_executions]],
            "pagination": {
                "page": query.page,
                "per_page": query.per_page,
                "total": len(executions),
                "pages": (len(executions) + query.per_page - 1) // query.per_page,
                "has_next": end_idx < len(executions),
                "has_prev": query.page > 1
            }
        }
        
        return APIResponse.success(data=response_data)
        
    except Exception as e:
        logger.error(f"Error listing executions: {str(e)}")
        return APIResponse.internal_error()


@executions_bp.route('/<int:execution_id>', methods=['GET'])
@handle_service_exceptions
def get_execution(execution_id: int):
    """Get detailed execution information"""
    try:
        execution = execution_service.get_execution_by_id(execution_id)
        if not execution:
            return APIResponse.not_found("Execution")
        
        execution_data = execution_to_response_schema(execution)
        return APIResponse.success(data=execution_data.dict())
        
    except ExecutionNotFoundError:
        return APIResponse.not_found("Execution")
    except Exception as e:
        logger.error(f"Error getting execution {execution_id}: {str(e)}")
        return APIResponse.internal_error()


@executions_bp.route('/<int:execution_id>/logs', methods=['GET'])
@handle_service_exceptions
def get_execution_logs(execution_id: int):
    """Get execution conversation logs"""
    try:
        execution = execution_service.get_execution_by_id(execution_id)
        if not execution:
            return APIResponse.not_found("Execution")
        
        # Get logs from execution object directly
        logs = execution.logs if hasattr(execution, 'logs') and execution.logs else []
        
        response_data = {
            "execution_id": execution_id,
            "logs": logs,
            "total_entries": len(logs) if logs else 0
        }
        
        return APIResponse.success(data=response_data)
        
    except ExecutionNotFoundError:
        return APIResponse.not_found("Execution")
    except Exception as e:
        logger.error(f"Error getting execution {execution_id} logs: {str(e)}")
        return APIResponse.internal_error()


@executions_bp.route('/<int:execution_id>/tool-logs', methods=['GET'])
@handle_service_exceptions
def get_execution_tool_logs(execution_id: int):
    """Get execution tool logs"""
    try:
        execution = execution_service.get_execution_by_id(execution_id)
        if not execution:
            return APIResponse.not_found("Execution")
        
        # Get tool logs from execution object
        tool_logs = execution.get_tool_logs() if hasattr(execution, 'get_tool_logs') else []
        
        # Get optional status filter
        status_filter = request.args.get('status')
        if status_filter:
            tool_logs = [log for log in tool_logs if log.get('status') == status_filter]
        
        response_data = {
            "execution_id": execution_id,
            "tool_logs": tool_logs,
            "total_entries": len(tool_logs),
            "status_filter": status_filter
        }
        
        return APIResponse.success(data=response_data)
        
    except ExecutionNotFoundError:
        return APIResponse.not_found("Execution")
    except Exception as e:
        logger.error(f"Error getting execution {execution_id} tool logs: {str(e)}")
        return APIResponse.internal_error()


@executions_bp.route('/<int:execution_id>/cancel', methods=['POST'])
@handle_service_exceptions
def cancel_execution(execution_id: int):
    """Cancel a running execution"""
    try:
        execution = execution_service.get_execution_by_id(execution_id)
        if not execution:
            return APIResponse.not_found("Execution")
        
        if execution.status != 'running':
            return APIResponse.error(
                message="Only running executions can be cancelled",
                error_code="EXECUTION_NOT_RUNNING"
            )
        
        success = execution_service.cancel_execution(execution_id)
        if not success:
            return APIResponse.error(
                message="Failed to cancel execution",
                error_code="CANCELLATION_FAILED"
            )
        
        logger.info(f"Cancelled execution {execution_id}")
        
        return APIResponse.success(message="Execution cancelled successfully")
        
    except ExecutionNotFoundError:
        return APIResponse.not_found("Execution")
    except Exception as e:
        logger.error(f"Error cancelling execution {execution_id}: {str(e)}")
        return APIResponse.internal_error()


@executions_bp.route('/stats', methods=['GET'])
@handle_service_exceptions
def get_execution_stats():
    """Get execution statistics"""
    try:
        # Get optional filters
        agent_id = request.args.get('agent_id', type=int)
        task_id = request.args.get('task_id', type=int)
        days = request.args.get('days', type=int, default=30)
        
        filters = {}
        if agent_id:
            filters['agent_id'] = agent_id
        if task_id:
            filters['task_id'] = task_id
        
        stats = execution_service.get_execution_statistics()
        
        return APIResponse.success(data=stats)
        
    except Exception as e:
        logger.error(f"Error getting execution stats: {str(e)}")
        return APIResponse.internal_error()


@executions_bp.route('/running', methods=['GET'])
@handle_service_exceptions
def get_running_executions():
    """Get all currently running executions"""
    try:
        executions = execution_service.get_running_executions()
        executions_data = [execution_to_list_schema(execution) for execution in executions]
        
        response_data = {
            "running_executions": [execution.dict() for execution in executions_data],
            "count": len(executions_data)
        }
        
        return APIResponse.success(data=response_data)
        
    except Exception as e:
        logger.error(f"Error getting running executions: {str(e)}")
        return APIResponse.internal_error()


@executions_bp.route('/recent', methods=['GET'])
@handle_service_exceptions
def get_recent_executions():
    """Get recent executions (last 24 hours)"""
    try:
        limit = min(int(request.args.get('limit', 50)), 100)
        
        # Get all executions and filter for recent ones (last 24 hours)
        executions = execution_service.execution_repo.get_all()
        # Sort by created_at descending and limit
        executions = sorted(executions, key=lambda x: x.created_at, reverse=True)[:limit]
        executions_data = [execution_to_list_schema(execution) for execution in executions]
        
        response_data = {
            "recent_executions": [execution.dict() for execution in executions_data],
            "count": len(executions_data)
        }
        
        return APIResponse.success(data=response_data)
        
    except Exception as e:
        logger.error(f"Error getting recent executions: {str(e)}")
        return APIResponse.internal_error()


@executions_bp.route('/agent/<int:agent_id>', methods=['GET'])
@handle_service_exceptions
def get_agent_executions(agent_id: int):
    """Get executions for a specific agent"""
    try:
        page = int(request.args.get('page', 1))
        per_page = min(int(request.args.get('per_page', 20)), 100)
        status = request.args.get('status')
        
        filters = {'agent_id': agent_id}
        if status:
            filters['status'] = status
        
        executions = execution_service.get_executions_by_agent(agent_id)
        
        # Filter by status if specified
        if status:
            executions = [e for e in executions if e.status == status]
        
        # Simple pagination
        start_idx = (page - 1) * per_page
        end_idx = start_idx + per_page
        paginated_executions = executions[start_idx:end_idx]
        
        executions_data = [execution_to_list_schema(execution) for execution in paginated_executions]
        
        response_data = {
            "executions": [execution.dict() for execution in executions_data],
            "agent_id": agent_id,
            "pagination": {
                "page": page,
                "per_page": per_page,
                "total": len(executions),
                "pages": (len(executions) + per_page - 1) // per_page,
                "has_next": end_idx < len(executions),
                "has_prev": page > 1
            }
        }
        
        return APIResponse.success(data=response_data)
        
    except Exception as e:
        logger.error(f"Error getting executions for agent {agent_id}: {str(e)}")
        return APIResponse.internal_error()


@executions_bp.route('/task/<int:task_id>', methods=['GET'])
@handle_service_exceptions
def get_task_executions(task_id: int):
    """Get executions for a specific task"""
    try:
        page = int(request.args.get('page', 1))
        per_page = min(int(request.args.get('per_page', 20)), 100)
        status = request.args.get('status')
        
        filters = {'task_id': task_id}
        if status:
            filters['status'] = status
        
        executions = execution_service.get_executions_by_task(task_id)
        
        # Filter by status if specified
        if status:
            executions = [e for e in executions if e.status == status]
        
        # Simple pagination
        start_idx = (page - 1) * per_page
        end_idx = start_idx + per_page
        paginated_executions = executions[start_idx:end_idx]
        
        executions_data = [execution_to_list_schema(execution) for execution in paginated_executions]
        
        response_data = {
            "executions": [execution.dict() for execution in executions_data],
            "task_id": task_id,
            "pagination": {
                "page": page,
                "per_page": per_page,
                "total": len(executions),
                "pages": (len(executions) + per_page - 1) // per_page,
                "has_next": end_idx < len(executions),
                "has_prev": page > 1
            }
        }
        
        return APIResponse.success(data=response_data)
        
    except Exception as e:
        logger.error(f"Error getting executions for task {task_id}: {str(e)}")
        return APIResponse.internal_error()
