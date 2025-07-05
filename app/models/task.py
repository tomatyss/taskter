"""
Task model with enhanced functionality
"""
from datetime import datetime, timezone
from typing import Optional, Dict, Any
from sqlalchemy import Column, Integer, String, Text, DateTime, ForeignKey, JSON
from sqlalchemy.orm import relationship

from db import db
from app.core.constants import TaskStatus, ExecutionStatus


def utcnow():
    return datetime.now(timezone.utc)


class Task(db.Model):
    """Task model with enhanced methods"""
    
    __tablename__ = 'task'
    
    id = Column(Integer, primary_key=True)
    title = Column(String(200), nullable=False)
    description = Column(Text)
    status = Column(String(20), nullable=False, default=TaskStatus.TODO.value)
    created_at = Column(DateTime, default=utcnow)
    updated_at = Column(DateTime, default=utcnow, onupdate=utcnow)
    
    # Agent assignment fields
    assigned_agent_id = Column(Integer, ForeignKey('agent.id'))
    execution_status = Column(String(20), default=ExecutionStatus.MANUAL.value)
    
    # Relationships
    assigned_agent = relationship('Agent', back_populates='assigned_tasks')
    executions = relationship('AgentExecution', back_populates='task', cascade='all, delete-orphan')
    
    def __repr__(self):
        return f'<Task {self.title}>'
    
    def to_dict(self, include_relations: bool = False) -> Dict[str, Any]:
        """Convert task to dictionary"""
        data = {
            'id': self.id,
            'title': self.title,
            'description': self.description,
            'status': self.status,
            'execution_status': self.execution_status,
            'created_at': self.created_at.isoformat() if self.created_at else None,
            'updated_at': self.updated_at.isoformat() if self.updated_at else None,
            'assigned_agent_id': self.assigned_agent_id
        }
        
        if include_relations:
            data['assigned_agent'] = self.assigned_agent.to_dict() if self.assigned_agent else None
            data['executions'] = [exec.to_dict() for exec in self.executions]
        
        return data
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'Task':
        """Create task from dictionary"""
        return cls(
            title=data['title'],
            description=data.get('description'),
            status=data.get('status', TaskStatus.TODO.value),
            assigned_agent_id=data.get('assigned_agent_id'),
            execution_status=data.get('execution_status', ExecutionStatus.MANUAL.value)
        )
    
    def update_from_dict(self, data: Dict[str, Any]) -> None:
        """Update task from dictionary"""
        if 'title' in data:
            self.title = data['title']
        if 'description' in data:
            self.description = data['description']
        if 'status' in data:
            self.status = data['status']
        if 'assigned_agent_id' in data:
            self.assigned_agent_id = data['assigned_agent_id']
        if 'execution_status' in data:
            self.execution_status = data['execution_status']
        
        self.updated_at = utcnow()
    
    def can_be_assigned(self) -> bool:
        """Check if task can be assigned to an agent"""
        return self.execution_status not in [ExecutionStatus.RUNNING.value]
    
    def can_be_unassigned(self) -> bool:
        """Check if task can be unassigned from an agent"""
        return self.execution_status not in [ExecutionStatus.RUNNING.value]
    
    def can_be_deleted(self) -> bool:
        """Check if task can be deleted"""
        return self.execution_status not in [ExecutionStatus.RUNNING.value]
    
    def is_assigned(self) -> bool:
        """Check if task is assigned to an agent"""
        return self.assigned_agent_id is not None
    
    def is_running(self) -> bool:
        """Check if task is currently running"""
        return self.execution_status == ExecutionStatus.RUNNING.value
    
    def is_completed(self) -> bool:
        """Check if task is completed"""
        return self.status == TaskStatus.DONE.value
    
    def get_latest_execution(self) -> Optional['AgentExecution']:
        """Get the latest execution for this task"""
        if not self.executions:
            return None
        return max(self.executions, key=lambda x: x.created_at)
    
    def get_running_execution(self) -> Optional['AgentExecution']:
        """Get the currently running execution for this task"""
        from app.core.constants import AgentExecutionStatus
        for execution in self.executions:
            if execution.status == AgentExecutionStatus.RUNNING.value:
                return execution
        return None
    
    def assign_to_agent(self, agent_id: int) -> None:
        """Assign task to an agent"""
        if not self.can_be_assigned():
            raise ValueError("Task cannot be assigned while running")
        
        self.assigned_agent_id = agent_id
        self.execution_status = ExecutionStatus.ASSIGNED.value
        self.updated_at = utcnow()
    
    def unassign_from_agent(self) -> None:
        """Unassign task from agent"""
        if not self.can_be_unassigned():
            raise ValueError("Task cannot be unassigned while running")
        
        self.assigned_agent_id = None
        self.execution_status = ExecutionStatus.MANUAL.value
        self.updated_at = utcnow()
    
    def start_execution(self) -> None:
        """Mark task as running"""
        self.execution_status = ExecutionStatus.RUNNING.value
        self.updated_at = utcnow()
    
    def complete_execution(self, success: bool = True) -> None:
        """Mark task execution as completed"""
        if success:
            self.execution_status = ExecutionStatus.COMPLETED.value
            self.status = TaskStatus.DONE.value
        else:
            self.execution_status = ExecutionStatus.FAILED.value
        
        self.updated_at = utcnow()
    
    def fail_execution(self) -> None:
        """Mark task execution as failed"""
        self.execution_status = ExecutionStatus.FAILED.value
        self.updated_at = utcnow()
    
    def reset_execution(self) -> None:
        """Reset task execution status"""
        if self.assigned_agent_id:
            self.execution_status = ExecutionStatus.ASSIGNED.value
        else:
            self.execution_status = ExecutionStatus.MANUAL.value
        
        self.updated_at = utcnow()
    
    def move_to_status(self, new_status: str) -> None:
        """Move task to a new status"""
        if new_status not in [status.value for status in TaskStatus]:
            raise ValueError(f"Invalid status: {new_status}")
        
        self.status = new_status
        self.updated_at = utcnow()
    
    def get_execution_summary(self) -> Dict[str, Any]:
        """Get execution summary for this task"""
        executions = self.executions
        if not executions:
            return {
                'total_executions': 0,
                'successful_executions': 0,
                'failed_executions': 0,
                'total_tokens_used': 0,
                'total_execution_time': 0
            }
        
        from app.core.constants import AgentExecutionStatus
        successful = [e for e in executions if e.status == AgentExecutionStatus.COMPLETED.value]
        failed = [e for e in executions if e.status == AgentExecutionStatus.FAILED.value]
        
        return {
            'total_executions': len(executions),
            'successful_executions': len(successful),
            'failed_executions': len(failed),
            'total_tokens_used': sum(e.tokens_used or 0 for e in executions),
            'total_execution_time': sum(e.execution_time_seconds or 0 for e in executions)
        }
