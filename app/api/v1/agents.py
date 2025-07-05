"""
Agent management API endpoints.
"""

from flask import Blueprint, request
from app.api.response import APIResponse, handle_service_exceptions, validate_json_input
from app.services.agent_service import AgentService
from app.schemas.agent_schemas import (
    AgentCreateSchema, AgentUpdateSchema, 
    agent_to_response_schema, agent_to_list_schema,
    ToolResponseSchema, ProviderResponseSchema
)
from app.core.exceptions import AgentNotFoundError, AgentCreationError
from app.core.logging import get_logger
from llm_providers import LLMProviderFactory
from tools import tool_registry

# Create blueprint
agents_bp = Blueprint('agents', __name__, url_prefix='/api/v1/agents')
logger = get_logger(__name__)

# Initialize services (these will be injected via dependency injection in the future)
agent_service = AgentService()


@agents_bp.route('', methods=['GET'])
@handle_service_exceptions
def list_agents():
    """List all agents"""
    try:
        # Get query parameters
        page = int(request.args.get('page', 1))
        per_page = min(int(request.args.get('per_page', 20)), 100)
        is_active = request.args.get('is_active')
        
        # Build filters
        filters = {}
        if is_active is not None:
            filters['is_active'] = is_active.lower() == 'true'
        
        # Get agents
        result = agent_service.get_agents_paginated(
            page=page,
            per_page=per_page,
            filters=filters
        )
        
        # Convert to response format
        agents_data = [agent_to_list_schema(agent) for agent in result.items]
        
        response_data = {
            "agents": [agent.dict() for agent in agents_data],
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
        logger.error(f"Error listing agents: {str(e)}")
        return APIResponse.internal_error()


@agents_bp.route('', methods=['POST'])
@validate_json_input(AgentCreateSchema)
@handle_service_exceptions
def create_agent(data: AgentCreateSchema):
    """Create a new agent"""
    try:
        agent = agent_service.create_agent(data)
        agent_data = agent_to_response_schema(agent)
        
        logger.info(f"Created agent {agent.id}: {agent.name}")
        
        return APIResponse.created(
            data=agent_data.dict(),
            message="Agent created successfully"
        )
        
    except AgentCreationError as e:
        return APIResponse.error(str(e), "AGENT_CREATION_ERROR")
    except Exception as e:
        logger.error(f"Error creating agent: {str(e)}")
        return APIResponse.internal_error()


@agents_bp.route('/<int:agent_id>', methods=['GET'])
@handle_service_exceptions
def get_agent(agent_id: int):
    """Get a specific agent by ID"""
    try:
        agent = agent_service.get_agent_by_id(agent_id)
        if not agent:
            return APIResponse.not_found("Agent")
        
        agent_data = agent_to_response_schema(agent)
        return APIResponse.success(data=agent_data.dict())
        
    except AgentNotFoundError:
        return APIResponse.not_found("Agent")
    except Exception as e:
        logger.error(f"Error getting agent {agent_id}: {str(e)}")
        return APIResponse.internal_error()


@agents_bp.route('/<int:agent_id>', methods=['PUT'])
@validate_json_input(AgentUpdateSchema)
@handle_service_exceptions
def update_agent(data: AgentUpdateSchema, agent_id: int):
    """Update a specific agent"""
    try:
        agent = agent_service.update_agent(agent_id, data)
        agent_data = agent_to_response_schema(agent)
        
        logger.info(f"Updated agent {agent_id}: {agent.name}")
        
        return APIResponse.success(
            data=agent_data.dict(),
            message="Agent updated successfully"
        )
        
    except AgentNotFoundError:
        return APIResponse.not_found("Agent")
    except Exception as e:
        logger.error(f"Error updating agent {agent_id}: {str(e)}")
        return APIResponse.internal_error()


@agents_bp.route('/<int:agent_id>', methods=['DELETE'])
@handle_service_exceptions
def delete_agent(agent_id: int):
    """Delete a specific agent"""
    try:
        success = agent_service.delete_agent(agent_id)
        if not success:
            return APIResponse.not_found("Agent")
        
        logger.info(f"Deleted agent {agent_id}")
        
        return APIResponse.success(message="Agent deleted successfully")
        
    except AgentNotFoundError:
        return APIResponse.not_found("Agent")
    except Exception as e:
        logger.error(f"Error deleting agent {agent_id}: {str(e)}")
        return APIResponse.internal_error()


@agents_bp.route('/<int:agent_id>/activate', methods=['POST'])
@handle_service_exceptions
def activate_agent(agent_id: int):
    """Activate an agent"""
    try:
        agent = agent_service.activate_agent(agent_id)
        agent_data = agent_to_response_schema(agent)
        
        logger.info(f"Activated agent {agent_id}")
        
        return APIResponse.success(
            data=agent_data.dict(),
            message="Agent activated successfully"
        )
        
    except AgentNotFoundError:
        return APIResponse.not_found("Agent")
    except Exception as e:
        logger.error(f"Error activating agent {agent_id}: {str(e)}")
        return APIResponse.internal_error()


@agents_bp.route('/<int:agent_id>/deactivate', methods=['POST'])
@handle_service_exceptions
def deactivate_agent(agent_id: int):
    """Deactivate an agent"""
    try:
        agent = agent_service.deactivate_agent(agent_id)
        agent_data = agent_to_response_schema(agent)
        
        logger.info(f"Deactivated agent {agent_id}")
        
        return APIResponse.success(
            data=agent_data.dict(),
            message="Agent deactivated successfully"
        )
        
    except AgentNotFoundError:
        return APIResponse.not_found("Agent")
    except Exception as e:
        logger.error(f"Error deactivating agent {agent_id}: {str(e)}")
        return APIResponse.internal_error()


@agents_bp.route('/tools', methods=['GET'])
@handle_service_exceptions
def list_tools():
    """List available tools"""
    try:
        tools = tool_registry.get_available_tools()
        tool_details = []
        
        for tool_name in tools:
            tool = tool_registry.get_tool(tool_name)
            if tool:
                tool_details.append(ToolResponseSchema(
                    name=tool.name,
                    description=tool.description,
                    input_schema=tool.input_schema
                ))
        
        response_data = {
            "tools": [tool.dict() for tool in tool_details],
            "count": len(tool_details)
        }
        
        return APIResponse.success(data=response_data)
        
    except Exception as e:
        logger.error(f"Error listing tools: {str(e)}")
        return APIResponse.internal_error()


@agents_bp.route('/providers', methods=['GET'])
@handle_service_exceptions
def list_providers():
    """List available LLM providers"""
    try:
        providers = LLMProviderFactory.get_available_providers()
        default_models = LLMProviderFactory.get_default_models()
        
        provider_details = []
        for provider in providers:
            provider_details.append(ProviderResponseSchema(
                name=provider,
                default_model=default_models.get(provider, 'unknown')
            ))
        
        response_data = {
            "providers": [provider.dict() for provider in provider_details],
            "count": len(provider_details)
        }
        
        return APIResponse.success(data=response_data)
        
    except Exception as e:
        logger.error(f"Error listing providers: {str(e)}")
        return APIResponse.internal_error()


@agents_bp.route('/<int:agent_id>/stats', methods=['GET'])
@handle_service_exceptions
def get_agent_stats(agent_id: int):
    """Get agent statistics"""
    try:
        agent = agent_service.get_agent_by_id(agent_id)
        if not agent:
            return APIResponse.not_found("Agent")
        
        stats = agent_service.get_agent_statistics(agent_id)
        return APIResponse.success(data=stats)
        
    except AgentNotFoundError:
        return APIResponse.not_found("Agent")
    except Exception as e:
        logger.error(f"Error getting agent {agent_id} stats: {str(e)}")
        return APIResponse.internal_error()
