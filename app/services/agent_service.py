"""
Agent service layer for business logic
"""
from typing import List, Optional, Dict, Any

from app.repositories.agent_repository import AgentRepository
from app.models.agent import Agent
from app.core.constants import LLMProvider
from app.core.exceptions import (
    AgentNotFoundError, AgentValidationError, AgentNotActiveError,
    ValidationError, ConflictError
)
from app.core.logging import get_logger

logger = get_logger(__name__)


class AgentService:
    """Service for agent business logic"""
    
    def __init__(self, agent_repository: Optional[AgentRepository] = None):
        self.agent_repo = agent_repository or AgentRepository()
    
    def create_agent(self, name: str, system_instructions: str,
                    llm_provider: LLMProvider, llm_model: str,
                    description: Optional[str] = None,
                    llm_api_key: Optional[str] = None,
                    available_tools: Optional[List[str]] = None,
                    config: Optional[Dict[str, Any]] = None,
                    is_active: bool = True) -> Agent:
        """Create a new agent"""
        try:
            # Validate input
            if not name or not name.strip():
                raise AgentValidationError("Agent name is required")
            
            if len(name) > 100:
                raise AgentValidationError("Agent name must be 100 characters or less")
            
            if not system_instructions or not system_instructions.strip():
                raise AgentValidationError("System instructions are required")
            
            if not llm_model or not llm_model.strip():
                raise AgentValidationError("LLM model is required")
            
            # Check if name already exists
            if self.agent_repo.name_exists(name):
                raise ConflictError(f"Agent with name '{name}' already exists")
            
            # Validate provider
            if not self._is_provider_supported(llm_provider):
                provider_value = llm_provider.value if hasattr(llm_provider, 'value') else str(llm_provider)
                raise AgentValidationError(f"Unsupported LLM provider: {provider_value}")
            
            # Create agent
            provider_value = llm_provider.value if hasattr(llm_provider, 'value') else str(llm_provider)
            agent_data = {
                'name': name.strip(),
                'description': description.strip() if description else None,
                'system_instructions': system_instructions.strip(),
                'llm_provider': provider_value,
                'llm_model': llm_model.strip(),
                'llm_api_key': llm_api_key,
                'available_tools': available_tools or [],
                'config': config or {},
                'is_active': is_active
            }
            
            agent = Agent.from_dict(agent_data)
            
            # Validate agent configuration
            validation_issues = agent.validate_configuration()
            if validation_issues:
                raise AgentValidationError(f"Agent validation failed: {', '.join(validation_issues)}")
            
            created_agent = self.agent_repo.create(agent)
            
            logger.info(f"Created agent {created_agent.id}: {created_agent.name}")
            return created_agent
            
        except Exception as e:
            logger.error(f"Failed to create agent: {str(e)}")
            raise
    
    def get_agent_by_id(self, agent_id: int) -> Agent:
        """Get agent by ID"""
        agent = self.agent_repo.get_by_id(agent_id)
        if not agent:
            raise AgentNotFoundError(agent_id)
        return agent
    
    def get_agent_by_name(self, name: str) -> Agent:
        """Get agent by name"""
        agent = self.agent_repo.get_by_name(name)
        if not agent:
            raise AgentNotFoundError(f"Agent with name '{name}' not found")
        return agent
    
    def update_agent(self, agent_id: int, name: Optional[str] = None,
                    description: Optional[str] = None,
                    system_instructions: Optional[str] = None,
                    llm_provider: Optional[LLMProvider] = None,
                    llm_model: Optional[str] = None,
                    llm_api_key: Optional[str] = None,
                    available_tools: Optional[List[str]] = None,
                    config: Optional[Dict[str, Any]] = None,
                    is_active: Optional[bool] = None) -> Agent:
        """Update an existing agent"""
        try:
            agent = self.get_agent_by_id(agent_id)
            
            # Validate input
            if name is not None:
                if not name.strip():
                    raise AgentValidationError("Agent name cannot be empty")
                if len(name) > 100:
                    raise AgentValidationError("Agent name must be 100 characters or less")
                
                # Check if name already exists (excluding current agent)
                if self.agent_repo.name_exists(name, exclude_id=agent_id):
                    raise ConflictError(f"Agent with name '{name}' already exists")
            
            if system_instructions is not None and not system_instructions.strip():
                raise AgentValidationError("System instructions cannot be empty")
            
            if llm_model is not None and not llm_model.strip():
                raise AgentValidationError("LLM model cannot be empty")
            
            # Check if agent can be updated
            if agent.get_running_task_count() > 0:
                # Only allow certain updates when agent has running tasks
                restricted_fields = ['llm_provider', 'llm_model', 'system_instructions']
                if any(locals().get(field) is not None for field in restricted_fields):
                    raise ConflictError("Cannot update core agent settings while tasks are running")
            
            # Update agent
            update_data = {}
            if name is not None:
                update_data['name'] = name.strip()
            if description is not None:
                update_data['description'] = description.strip() if description else None
            if system_instructions is not None:
                update_data['system_instructions'] = system_instructions.strip()
            if llm_provider is not None:
                if not self._is_provider_supported(llm_provider):
                    provider_value = llm_provider.value if hasattr(llm_provider, 'value') else str(llm_provider)
                    raise AgentValidationError(f"Unsupported LLM provider: {provider_value}")
                provider_value = llm_provider.value if hasattr(llm_provider, 'value') else str(llm_provider)
                update_data['llm_provider'] = provider_value
            if llm_model is not None:
                update_data['llm_model'] = llm_model.strip()
            if llm_api_key is not None:
                update_data['llm_api_key'] = llm_api_key
            if available_tools is not None:
                update_data['available_tools'] = available_tools
            if config is not None:
                update_data['config'] = config
            if is_active is not None:
                update_data['is_active'] = is_active
            
            if update_data:
                agent.update_from_dict(update_data)
                
                # Validate updated agent configuration
                validation_issues = agent.validate_configuration()
                if validation_issues:
                    raise AgentValidationError(f"Agent validation failed: {', '.join(validation_issues)}")
                
                updated_agent = self.agent_repo.update(agent)
                
                logger.info(f"Updated agent {agent_id}")
                return updated_agent
            
            return agent
            
        except Exception as e:
            logger.error(f"Failed to update agent {agent_id}: {str(e)}")
            raise
    
    def delete_agent(self, agent_id: int) -> bool:
        """Delete an agent"""
        try:
            agent = self.get_agent_by_id(agent_id)
            
            # Check if agent can be deleted
            if not agent.can_be_deleted():
                raise ConflictError("Cannot delete agent with running tasks")
            
            success = self.agent_repo.delete(agent)
            
            if success:
                logger.info(f"Deleted agent {agent_id}")
            
            return success
            
        except Exception as e:
            logger.error(f"Failed to delete agent {agent_id}: {str(e)}")
            raise
    
    def activate_agent(self, agent_id: int) -> Agent:
        """Activate an agent"""
        try:
            agent = self.get_agent_by_id(agent_id)
            
            # Validate that agent can be activated
            validation_issues = agent.validate_configuration()
            if validation_issues:
                raise AgentValidationError(f"Cannot activate agent: {', '.join(validation_issues)}")
            
            agent.activate()
            updated_agent = self.agent_repo.update(agent)
            
            logger.info(f"Activated agent {agent_id}")
            return updated_agent
            
        except Exception as e:
            logger.error(f"Failed to activate agent {agent_id}: {str(e)}")
            raise
    
    def deactivate_agent(self, agent_id: int) -> Agent:
        """Deactivate an agent"""
        try:
            agent = self.get_agent_by_id(agent_id)
            
            # Check if agent has running tasks
            if agent.get_running_task_count() > 0:
                raise ConflictError("Cannot deactivate agent with running tasks")
            
            agent.deactivate()
            updated_agent = self.agent_repo.update(agent)
            
            logger.info(f"Deactivated agent {agent_id}")
            return updated_agent
            
        except Exception as e:
            logger.error(f"Failed to deactivate agent {agent_id}: {str(e)}")
            raise
    
    def add_tool_to_agent(self, agent_id: int, tool_name: str) -> Agent:
        """Add a tool to an agent"""
        try:
            agent = self.get_agent_by_id(agent_id)
            
            # Validate tool name
            if not tool_name or not tool_name.strip():
                raise ValidationError("Tool name is required")
            
            # Check if tool is already added
            if agent.has_tool(tool_name):
                raise ConflictError(f"Agent already has tool '{tool_name}'")
            
            agent.add_tool(tool_name)
            updated_agent = self.agent_repo.update(agent)
            
            logger.info(f"Added tool '{tool_name}' to agent {agent_id}")
            return updated_agent
            
        except Exception as e:
            logger.error(f"Failed to add tool to agent {agent_id}: {str(e)}")
            raise
    
    def remove_tool_from_agent(self, agent_id: int, tool_name: str) -> Agent:
        """Remove a tool from an agent"""
        try:
            agent = self.get_agent_by_id(agent_id)
            
            # Check if tool exists
            if not agent.has_tool(tool_name):
                raise ValidationError(f"Agent does not have tool '{tool_name}'")
            
            agent.remove_tool(tool_name)
            updated_agent = self.agent_repo.update(agent)
            
            logger.info(f"Removed tool '{tool_name}' from agent {agent_id}")
            return updated_agent
            
        except Exception as e:
            logger.error(f"Failed to remove tool from agent {agent_id}: {str(e)}")
            raise
    
    def get_active_agents(self, limit: Optional[int] = None,
                         offset: Optional[int] = None) -> List[Agent]:
        """Get all active agents"""
        return self.agent_repo.get_active_agents(limit, offset)
    
    def get_inactive_agents(self, limit: Optional[int] = None,
                           offset: Optional[int] = None) -> List[Agent]:
        """Get all inactive agents"""
        return self.agent_repo.get_inactive_agents(limit, offset)
    
    def get_agents_by_provider(self, provider: LLMProvider,
                              limit: Optional[int] = None,
                              offset: Optional[int] = None) -> List[Agent]:
        """Get agents by LLM provider"""
        return self.agent_repo.get_by_provider(provider, limit, offset)
    
    def search_agents(self, search_term: str,
                     limit: Optional[int] = None,
                     offset: Optional[int] = None) -> List[Agent]:
        """Search agents by name and description"""
        # Search in both name and description
        name_results = self.agent_repo.search_by_name(search_term, limit, offset)
        desc_results = self.agent_repo.search_by_description(search_term, limit, offset)
        
        # Combine and deduplicate results
        seen_ids = set()
        combined_results = []
        
        for agent in name_results + desc_results:
            if agent.id not in seen_ids:
                combined_results.append(agent)
                seen_ids.add(agent.id)
        
        # Sort by creation date (newest first)
        combined_results.sort(key=lambda x: x.created_at, reverse=True)
        
        return combined_results[:limit] if limit else combined_results
    
    def get_agents_with_tool(self, tool_name: str,
                            limit: Optional[int] = None,
                            offset: Optional[int] = None) -> List[Agent]:
        """Get agents that have a specific tool"""
        return self.agent_repo.get_agents_with_tool(tool_name, limit, offset)
    
    def get_agent_statistics(self) -> Dict[str, Any]:
        """Get comprehensive agent statistics"""
        return self.agent_repo.get_agent_statistics()
    
    def get_provider_statistics(self, provider: LLMProvider) -> Dict[str, Any]:
        """Get statistics for a specific provider"""
        return self.agent_repo.get_provider_statistics(provider)
    
    def bulk_activate_agents(self, agent_ids: List[int]) -> int:
        """Bulk activate agents"""
        try:
            # Validate that all agents exist and can be activated
            agents = []
            for agent_id in agent_ids:
                agent = self.get_agent_by_id(agent_id)
                validation_issues = agent.validate_configuration()
                if validation_issues:
                    raise AgentValidationError(f"Cannot activate agent {agent_id}: {', '.join(validation_issues)}")
                agents.append(agent)
            
            # Perform bulk activation
            updated_count = self.agent_repo.bulk_activate_agents(agent_ids)
            
            logger.info(f"Bulk activated {updated_count} agents")
            return updated_count
            
        except Exception as e:
            logger.error(f"Failed to bulk activate agents: {str(e)}")
            raise
    
    def bulk_deactivate_agents(self, agent_ids: List[int]) -> int:
        """Bulk deactivate agents"""
        try:
            # Validate that all agents can be deactivated
            for agent_id in agent_ids:
                agent = self.get_agent_by_id(agent_id)
                if agent.get_running_task_count() > 0:
                    raise ConflictError(f"Cannot deactivate agent {agent_id} with running tasks")
            
            # Perform bulk deactivation
            updated_count = self.agent_repo.bulk_deactivate_agents(agent_ids)
            
            logger.info(f"Bulk deactivated {updated_count} agents")
            return updated_count
            
        except Exception as e:
            logger.error(f"Failed to bulk deactivate agents: {str(e)}")
            raise
    
    def get_agents_needing_api_keys(self) -> List[Agent]:
        """Get active agents that don't have API keys"""
        return self.agent_repo.get_agents_needing_api_keys()
    
    def validate_agent_for_execution(self, agent_id: int) -> bool:
        """Validate if agent can execute tasks"""
        try:
            agent = self.get_agent_by_id(agent_id)
            
            if not agent.is_active:
                raise AgentNotActiveError(f"Agent {agent_id} is not active")
            
            if not agent.can_execute_tasks():
                raise AgentValidationError(f"Agent {agent_id} cannot execute tasks (missing API key)")
            
            validation_issues = agent.validate_configuration()
            if validation_issues:
                raise AgentValidationError(f"Agent {agent_id} configuration invalid: {', '.join(validation_issues)}")
            
            return True
            
        except Exception as e:
            logger.error(f"Agent {agent_id} validation failed: {str(e)}")
            raise
    
    def get_most_used_agents(self, limit: int = 10) -> List[Dict[str, Any]]:
        """Get agents with most task assignments"""
        return self.agent_repo.get_most_used_agents(limit)
    
    def get_recent_agents(self, limit: int = 10) -> List[Agent]:
        """Get recently created agents"""
        return self.agent_repo.get_recent_agents(limit)
    
    def _is_provider_supported(self, provider) -> bool:
        """Check if the LLM provider is supported"""
        # Handle both string and enum inputs
        if isinstance(provider, LLMProvider):
            return True
        elif isinstance(provider, str):
            # Check if the string matches any of the enum values
            return provider in [p.value for p in LLMProvider]
        return False
