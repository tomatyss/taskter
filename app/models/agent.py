"""
Agent model with enhanced functionality
"""
from datetime import datetime, timezone
from typing import Optional, Dict, Any, List
from sqlalchemy import Column, Integer, String, Text, DateTime, Boolean, JSON
from sqlalchemy.orm import relationship

from db import db
from app.core.constants import LLMProvider


def utcnow():
    return datetime.now(timezone.utc)


class Agent(db.Model):
    """Agent model with enhanced methods"""
    
    __tablename__ = 'agent'
    
    id = Column(Integer, primary_key=True)
    name = Column(String(100), nullable=False, unique=True)
    description = Column(Text)
    system_instructions = Column(Text, nullable=False)
    llm_provider = Column(String(50), nullable=False)
    llm_model = Column(String(100), nullable=False)
    llm_api_key = Column(String(500))
    available_tools = Column(JSON, default=list)
    config = Column(JSON, default=dict)
    is_active = Column(Boolean, default=True)
    created_at = Column(DateTime, default=utcnow)
    updated_at = Column(DateTime, default=utcnow, onupdate=utcnow)
    
    # Relationships
    assigned_tasks = relationship('Task', back_populates='assigned_agent')
    executions = relationship('AgentExecution', back_populates='agent', cascade='all, delete-orphan')
    
    def __repr__(self):
        return f'<Agent {self.name}>'
    
    def to_dict(self, include_relations: bool = False, include_sensitive: bool = False) -> Dict[str, Any]:
        """Convert agent to dictionary"""
        data = {
            'id': self.id,
            'name': self.name,
            'description': self.description,
            'system_instructions': self.system_instructions,
            'llm_provider': self.llm_provider,
            'llm_model': self.llm_model,
            'available_tools': self.available_tools or [],
            'config': self.config or {},
            'is_active': self.is_active,
            'created_at': self.created_at.isoformat() if self.created_at else None,
            'updated_at': self.updated_at.isoformat() if self.updated_at else None
        }
        
        if include_sensitive:
            data['llm_api_key'] = self.llm_api_key
        
        if include_relations:
            data['assigned_tasks'] = [task.to_dict() for task in self.assigned_tasks]
            data['executions'] = [exec.to_dict() for exec in self.executions]
        
        return data
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'Agent':
        """Create agent from dictionary"""
        return cls(
            name=data['name'],
            description=data.get('description'),
            system_instructions=data['system_instructions'],
            llm_provider=data['llm_provider'],
            llm_model=data['llm_model'],
            llm_api_key=data.get('llm_api_key'),
            available_tools=data.get('available_tools', []),
            config=data.get('config', {}),
            is_active=data.get('is_active', True)
        )
    
    def update_from_dict(self, data: Dict[str, Any]) -> None:
        """Update agent from dictionary"""
        if 'name' in data:
            self.name = data['name']
        if 'description' in data:
            self.description = data['description']
        if 'system_instructions' in data:
            self.system_instructions = data['system_instructions']
        if 'llm_provider' in data:
            self.llm_provider = data['llm_provider']
        if 'llm_model' in data:
            self.llm_model = data['llm_model']
        if 'llm_api_key' in data:
            self.llm_api_key = data['llm_api_key']
        if 'available_tools' in data:
            self.available_tools = data['available_tools']
        if 'config' in data:
            self.config = data['config']
        if 'is_active' in data:
            self.is_active = data['is_active']
        
        self.updated_at = utcnow()
    
    def activate(self) -> None:
        """Activate the agent"""
        self.is_active = True
        self.updated_at = utcnow()
    
    def deactivate(self) -> None:
        """Deactivate the agent"""
        self.is_active = False
        self.updated_at = utcnow()
    
    def can_be_deleted(self) -> bool:
        """Check if agent can be deleted"""
        # Check if agent has running tasks
        from app.core.constants import ExecutionStatus
        running_tasks = [task for task in self.assigned_tasks 
                        if task.execution_status == ExecutionStatus.RUNNING.value]
        return len(running_tasks) == 0
    
    def can_execute_tasks(self) -> bool:
        """Check if agent can execute tasks"""
        return self.is_active and self.llm_api_key is not None
    
    def has_tool(self, tool_name: str) -> bool:
        """Check if agent has a specific tool"""
        return tool_name in (self.available_tools or [])
    
    def add_tool(self, tool_name: str) -> None:
        """Add a tool to the agent"""
        if not self.available_tools:
            self.available_tools = []
        
        if tool_name not in self.available_tools:
            self.available_tools.append(tool_name)
            self.updated_at = utcnow()
    
    def remove_tool(self, tool_name: str) -> None:
        """Remove a tool from the agent"""
        if self.available_tools and tool_name in self.available_tools:
            self.available_tools.remove(tool_name)
            self.updated_at = utcnow()
    
    def get_config_value(self, key: str, default=None):
        """Get a configuration value"""
        return (self.config or {}).get(key, default)
    
    def set_config_value(self, key: str, value: Any) -> None:
        """Set a configuration value"""
        if not self.config:
            self.config = {}
        
        self.config[key] = value
        self.updated_at = utcnow()
    
    def get_task_count(self) -> int:
        """Get number of assigned tasks"""
        return len(self.assigned_tasks)
    
    def get_running_task_count(self) -> int:
        """Get number of currently running tasks"""
        from app.core.constants import ExecutionStatus
        return len([task for task in self.assigned_tasks 
                   if task.execution_status == ExecutionStatus.RUNNING.value])
    
    def get_execution_statistics(self) -> Dict[str, Any]:
        """Get execution statistics for this agent"""
        executions = self.executions
        if not executions:
            return {
                'total_executions': 0,
                'successful_executions': 0,
                'failed_executions': 0,
                'total_tokens_used': 0,
                'total_execution_time': 0,
                'average_execution_time': 0
            }
        
        from app.core.constants import AgentExecutionStatus
        successful = [e for e in executions if e.status == AgentExecutionStatus.COMPLETED.value]
        failed = [e for e in executions if e.status == AgentExecutionStatus.FAILED.value]
        
        total_time = sum(e.execution_time_seconds or 0 for e in executions)
        avg_time = total_time / len(executions) if executions else 0
        
        return {
            'total_executions': len(executions),
            'successful_executions': len(successful),
            'failed_executions': len(failed),
            'total_tokens_used': sum(e.tokens_used or 0 for e in executions),
            'total_execution_time': total_time,
            'average_execution_time': avg_time
        }
    
    def get_latest_execution(self) -> Optional['AgentExecution']:
        """Get the latest execution for this agent"""
        if not self.executions:
            return None
        return max(self.executions, key=lambda x: x.created_at)
    
    def is_provider_supported(self, provider: str) -> bool:
        """Check if the agent's provider is supported"""
        try:
            LLMProvider(provider)
            return True
        except ValueError:
            return False
    
    def validate_configuration(self) -> List[str]:
        """Validate agent configuration and return list of issues"""
        issues = []
        
        if not self.name or not self.name.strip():
            issues.append("Agent name is required")
        
        if not self.system_instructions or not self.system_instructions.strip():
            issues.append("System instructions are required")
        
        if not self.llm_provider:
            issues.append("LLM provider is required")
        elif not self.is_provider_supported(self.llm_provider):
            issues.append(f"Unsupported LLM provider: {self.llm_provider}")
        
        if not self.llm_model:
            issues.append("LLM model is required")
        
        if self.is_active and not self.llm_api_key:
            issues.append("API key is required for active agents")
        
        return issues
    
    def is_valid(self) -> bool:
        """Check if agent configuration is valid"""
        return len(self.validate_configuration()) == 0
