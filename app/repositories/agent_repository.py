"""
Agent repository implementation
"""
from typing import List, Optional, Dict, Any
from sqlalchemy import desc, asc

from app.repositories.base import BaseRepository, PaginatedResult, paginate_query
from app.models.agent import Agent
from app.core.constants import LLMProvider
from app.core.logging import get_logger

logger = get_logger(__name__)


class AgentRepository(BaseRepository[Agent]):
    """Repository for Agent entities"""
    
    def __init__(self):
        super().__init__(Agent)
    
    def get_model_class(self) -> type:
        return Agent
    
    def get_by_name(self, name: str) -> Optional[Agent]:
        """Get agent by name"""
        return self.find_one_by({'name': name})
    
    def get_active_agents(self, limit: Optional[int] = None, 
                         offset: Optional[int] = None) -> List[Agent]:
        """Get all active agents"""
        return self.find_by(
            filters={'is_active': True},
            limit=limit,
            offset=offset,
            order_by='created_at',
            order_desc=True
        )
    
    def get_inactive_agents(self, limit: Optional[int] = None,
                           offset: Optional[int] = None) -> List[Agent]:
        """Get all inactive agents"""
        return self.find_by(
            filters={'is_active': False},
            limit=limit,
            offset=offset,
            order_by='updated_at',
            order_desc=True
        )
    
    def get_by_provider(self, provider: LLMProvider,
                       limit: Optional[int] = None,
                       offset: Optional[int] = None) -> List[Agent]:
        """Get agents by LLM provider"""
        return self.find_by(
            filters={'llm_provider': provider.value},
            limit=limit,
            offset=offset,
            order_by='created_at',
            order_desc=True
        )
    
    def get_by_model(self, model_name: str,
                    limit: Optional[int] = None,
                    offset: Optional[int] = None) -> List[Agent]:
        """Get agents by LLM model"""
        return self.find_by(
            filters={'llm_model': model_name},
            limit=limit,
            offset=offset,
            order_by='created_at',
            order_desc=True
        )
    
    def search_by_name(self, search_term: str,
                      limit: Optional[int] = None,
                      offset: Optional[int] = None) -> List[Agent]:
        """Search agents by name"""
        return self.find_by(
            filters={'name': {'ilike': search_term}},
            limit=limit,
            offset=offset,
            order_by='name',
            order_desc=False
        )
    
    def search_by_description(self, search_term: str,
                             limit: Optional[int] = None,
                             offset: Optional[int] = None) -> List[Agent]:
        """Search agents by description"""
        return self.find_by(
            filters={'description': {'ilike': search_term}},
            limit=limit,
            offset=offset,
            order_by='created_at',
            order_desc=True
        )
    
    def get_agents_with_tool(self, tool_name: str,
                            limit: Optional[int] = None,
                            offset: Optional[int] = None) -> List[Agent]:
        """Get agents that have a specific tool"""
        # This requires a custom query since we're searching in JSON array
        try:
            query = self.session.query(Agent).filter(
                Agent.available_tools.contains([tool_name])
            )
            
            if offset:
                query = query.offset(offset)
            if limit:
                query = query.limit(limit)
            
            query = query.order_by(desc(Agent.created_at))
            
            return query.all()
            
        except Exception as e:
            logger.error(f"Failed to get agents with tool {tool_name}: {str(e)}")
            return []
    
    def get_agents_without_api_key(self) -> List[Agent]:
        """Get agents that don't have API keys configured"""
        return self.find_by(
            filters={'llm_api_key': None},
            order_by='created_at',
            order_desc=True
        )
    
    def get_agents_created_after(self, date,
                                limit: Optional[int] = None) -> List[Agent]:
        """Get agents created after a specific date"""
        return self.find_by(
            filters={'created_at': {'gte': date}},
            limit=limit,
            order_by='created_at',
            order_desc=True
        )
    
    def get_agents_updated_after(self, date,
                                limit: Optional[int] = None) -> List[Agent]:
        """Get agents updated after a specific date"""
        return self.find_by(
            filters={'updated_at': {'gte': date}},
            limit=limit,
            order_by='updated_at',
            order_desc=True
        )
    
    def get_paginated_agents(self, page: int = 1, per_page: int = 20,
                            is_active: Optional[bool] = None,
                            provider: Optional[LLMProvider] = None,
                            search: Optional[str] = None) -> PaginatedResult:
        """Get paginated agents with optional filters"""
        query = self.session.query(Agent)
        
        # Apply filters
        if is_active is not None:
            query = query.filter(Agent.is_active == is_active)
        
        if provider:
            query = query.filter(Agent.llm_provider == provider.value)
        
        if search:
            search_filter = f"%{search}%"
            query = query.filter(
                Agent.name.ilike(search_filter) | 
                Agent.description.ilike(search_filter)
            )
        
        # Order by creation date (newest first)
        query = query.order_by(desc(Agent.created_at))
        
        return paginate_query(query, page, per_page)
    
    def get_agent_statistics(self) -> Dict[str, Any]:
        """Get agent statistics"""
        total_agents = self.count()
        
        stats = {
            'total_agents': total_agents,
            'active_agents': self.count({'is_active': True}),
            'inactive_agents': self.count({'is_active': False}),
            'agents_with_api_keys': self.count({'llm_api_key': {'not': None}}),
            'agents_without_api_keys': self.count({'llm_api_key': None}),
            'by_provider': {},
            'by_model': {}
        }
        
        # Count by provider
        for provider in LLMProvider:
            stats['by_provider'][provider.value] = self.count({
                'llm_provider': provider.value
            })
        
        # Get model distribution (this would need a custom query for exact counts)
        # For now, we'll get a sample
        agents = self.get_all(limit=1000)  # Get up to 1000 agents for stats
        model_counts = {}
        for agent in agents:
            model = agent.llm_model
            model_counts[model] = model_counts.get(model, 0) + 1
        
        stats['by_model'] = model_counts
        
        return stats
    
    def get_provider_statistics(self, provider: LLMProvider) -> Dict[str, Any]:
        """Get statistics for a specific provider"""
        base_filter = {'llm_provider': provider.value}
        
        stats = {
            'total_agents': self.count(base_filter),
            'active_agents': self.count({**base_filter, 'is_active': True}),
            'inactive_agents': self.count({**base_filter, 'is_active': False}),
            'with_api_keys': self.count({**base_filter, 'llm_api_key': {'not': None}}),
            'without_api_keys': self.count({**base_filter, 'llm_api_key': None})
        }
        
        return stats
    
    def activate_agent(self, agent_id: int) -> bool:
        """Activate an agent"""
        try:
            agent = self.get_by_id_or_404(agent_id)
            agent.activate()
            self.update(agent)
            return True
        except Exception as e:
            logger.error(f"Failed to activate agent {agent_id}: {str(e)}")
            return False
    
    def deactivate_agent(self, agent_id: int) -> bool:
        """Deactivate an agent"""
        try:
            agent = self.get_by_id_or_404(agent_id)
            agent.deactivate()
            self.update(agent)
            return True
        except Exception as e:
            logger.error(f"Failed to deactivate agent {agent_id}: {str(e)}")
            return False
    
    def bulk_activate_agents(self, agent_ids: List[int]) -> int:
        """Bulk activate agents"""
        updates = [
            {'id': agent_id, 'is_active': True}
            for agent_id in agent_ids
        ]
        return self.bulk_update(updates)
    
    def bulk_deactivate_agents(self, agent_ids: List[int]) -> int:
        """Bulk deactivate agents"""
        updates = [
            {'id': agent_id, 'is_active': False}
            for agent_id in agent_ids
        ]
        return self.bulk_update(updates)
    
    def get_agents_by_tool_count(self, min_tools: int = 0, max_tools: Optional[int] = None) -> List[Agent]:
        """Get agents by number of available tools"""
        # This requires custom SQL since we're counting JSON array elements
        try:
            query = self.session.query(Agent)
            
            # Filter by minimum tools
            if min_tools > 0:
                query = query.filter(
                    self.session.query(Agent).filter(
                        Agent.id == Agent.id
                    ).filter(
                        Agent.available_tools.isnot(None)
                    ).exists()
                )
            
            # For exact filtering, we'd need raw SQL or a more complex approach
            # For now, get all and filter in Python
            agents = query.all()
            
            filtered_agents = []
            for agent in agents:
                tool_count = len(agent.available_tools or [])
                if tool_count >= min_tools:
                    if max_tools is None or tool_count <= max_tools:
                        filtered_agents.append(agent)
            
            return filtered_agents
            
        except Exception as e:
            logger.error(f"Failed to get agents by tool count: {str(e)}")
            return []
    
    def get_most_used_agents(self, limit: int = 10) -> List[Dict[str, Any]]:
        """Get agents with most task assignments"""
        try:
            # This would typically be done with a JOIN query
            # For now, we'll get all agents and count their tasks
            agents = self.get_all()
            
            agent_usage = []
            for agent in agents:
                task_count = agent.get_task_count()
                if task_count > 0:
                    agent_usage.append({
                        'agent': agent,
                        'task_count': task_count,
                        'running_tasks': agent.get_running_task_count()
                    })
            
            # Sort by task count
            agent_usage.sort(key=lambda x: x['task_count'], reverse=True)
            
            return agent_usage[:limit]
            
        except Exception as e:
            logger.error(f"Failed to get most used agents: {str(e)}")
            return []
    
    def get_recent_agents(self, limit: int = 10) -> List[Agent]:
        """Get recently created agents"""
        return self.find_by(
            filters={},
            limit=limit,
            order_by='created_at',
            order_desc=True
        )
    
    def get_recently_updated_agents(self, limit: int = 10) -> List[Agent]:
        """Get recently updated agents"""
        return self.find_by(
            filters={},
            limit=limit,
            order_by='updated_at',
            order_desc=True
        )
    
    def name_exists(self, name: str, exclude_id: Optional[int] = None) -> bool:
        """Check if agent name already exists"""
        filters = {'name': name}
        if exclude_id:
            filters['id'] = {'not': exclude_id}
        
        return self.exists(filters)
    
    def get_agents_needing_api_keys(self) -> List[Agent]:
        """Get active agents that don't have API keys"""
        return self.find_by(
            filters={
                'is_active': True,
                'llm_api_key': None
            },
            order_by='created_at',
            order_desc=True
        )
