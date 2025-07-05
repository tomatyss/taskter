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
        
        # Get executions
        result = execution_service.get_executions_paginated(
            page=query.page,
            per_page=query.per_page,
            filters=filters
        )
        
        # Convert to response format
        executions_data = [execution_to_list_schema(execution) for execution in result.items]
        
        response_data = {
            "executions": [execution.dict() for execution in executions_data],
            "pagination": {
                "page": result.page,
                "per_page": result.per_page,
                "total": result.total,
                "pages": result.pages,
                "has_next": result.has_next,
                "has_prev": result.has_prev
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
        
        logs = execution_service.get_execution_logs(execution_id)
        
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
        
        stats = execution_service.get_execution_statistics(
            filters=filters,
            days=days
        )
        
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
        
        executions = execution_service.get_recent_executions(limit=limit)
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
        
        result = execution_service.get_executions_paginated(
            page=page,
            per_page=per_page,
            filters=filters
        )
        
        executions_data = [execution_to_list_schema(execution) for execution in result.items]
        
        response_data = {
            "executions": [execution.dict() for execution in executions_data],
            "agent_id": agent_id,
            "pagination": {
                "page": result.page,
                "per_page": result.per_page,
                "total": result.total,
                "pages": result.pages,
                "has_next": result.has_next,
                "has_prev": result.has_prev
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
        
        result = execution_service.get_executions_paginated(
            page=page,
            per_page=per_page,
            filters=filters
        )
        
        executions_data = [execution_to_list_schema(execution) for execution in result.items]
        
        response_data = {
            "executions": [execution.dict() for execution in executions_data],
            "task_id": task_id,
            "pagination": {
                "page": result.page,
                "per_page": result.per_page,
                "total": result.total,
                "pages": result.pages,
                "has_next": result.has_next,
                "has_prev": result.has_prev
            }
        }
        
        return APIResponse.success(data=response_data)
        
    except Exception as e:
        logger.error(f"Error getting executions for task {task_id}: {str(e)}")
        return APIResponse.internal_error()
