"""
Agent execution model with enhanced functionality
"""
from datetime import datetime, timezone
from typing import Optional, Dict, Any, List
from sqlalchemy import Column, Integer, String, Text, DateTime, ForeignKey, JSON, Float
from sqlalchemy.orm import relationship

from db import db
from app.core.constants import AgentExecutionStatus


def utcnow():
    return datetime.now(timezone.utc)


class AgentExecution(db.Model):
    """Agent execution model with enhanced methods"""
    
    __tablename__ = 'agent_execution'
    
    id = Column(Integer, primary_key=True)
    task_id = Column(Integer, ForeignKey('task.id'), nullable=False)
    agent_id = Column(Integer, ForeignKey('agent.id'), nullable=False)
    status = Column(String(20), nullable=False, default=AgentExecutionStatus.PENDING.value)
    
    # Execution details
    started_at = Column(DateTime)
    completed_at = Column(DateTime)
    execution_time_seconds = Column(Float)
    
    # LLM interaction details
    tokens_used = Column(Integer)
    llm_requests = Column(Integer, default=0)
    llm_cost = Column(Float)
    
    # Results and logs
    result = Column(Text)
    error_message = Column(Text)
    logs = Column(JSON, default=list)  # Conversation logs
    tool_logs = Column(JSON, default=list)  # Tool execution logs
    execution_metadata = Column(JSON, default=dict)
    
    # Timestamps
    created_at = Column(DateTime, default=utcnow)
    updated_at = Column(DateTime, default=utcnow, onupdate=utcnow)
    
    # Relationships
    task = relationship('Task', back_populates='executions')
    agent = relationship('Agent', back_populates='executions')
    
    def __repr__(self):
        return f'<AgentExecution {self.id} - Task {self.task_id} by Agent {self.agent_id}>'
    
    def to_dict(self, include_relations: bool = False) -> Dict[str, Any]:
        """Convert execution to dictionary"""
        data = {
            'id': self.id,
            'task_id': self.task_id,
            'agent_id': self.agent_id,
            'status': self.status,
            'started_at': self.started_at.isoformat() if self.started_at else None,
            'completed_at': self.completed_at.isoformat() if self.completed_at else None,
            'execution_time_seconds': self.execution_time_seconds,
            'tokens_used': self.tokens_used,
            'llm_requests': self.llm_requests,
            'llm_cost': self.llm_cost,
            'result': self.result,
            'error_message': self.error_message,
            'logs': self.logs or [],
            'tool_logs': self.tool_logs or [],
            'metadata': self.execution_metadata or {},
            'created_at': self.created_at.isoformat() if self.created_at else None,
            'updated_at': self.updated_at.isoformat() if self.updated_at else None
        }
        
        if include_relations:
            data['task'] = self.task.to_dict() if self.task else None
            data['agent'] = self.agent.to_dict() if self.agent else None
        
        return data
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'AgentExecution':
        """Create execution from dictionary"""
        return cls(
            task_id=data['task_id'],
            agent_id=data['agent_id'],
            status=data.get('status', AgentExecutionStatus.PENDING.value),
            tokens_used=data.get('tokens_used'),
            llm_requests=data.get('llm_requests', 0),
            llm_cost=data.get('llm_cost'),
            result=data.get('result'),
            error_message=data.get('error_message'),
            logs=data.get('logs', []),
            execution_metadata=data.get('metadata', {})
        )
    
    def update_from_dict(self, data: Dict[str, Any]) -> None:
        """Update execution from dictionary"""
        if 'status' in data:
            self.status = data['status']
        if 'tokens_used' in data:
            self.tokens_used = data['tokens_used']
        if 'llm_requests' in data:
            self.llm_requests = data['llm_requests']
        if 'llm_cost' in data:
            self.llm_cost = data['llm_cost']
        if 'result' in data:
            self.result = data['result']
        if 'error_message' in data:
            self.error_message = data['error_message']
        if 'logs' in data:
            self.logs = data['logs']
        if 'metadata' in data:
            self.execution_metadata = data['metadata']
        
        self.updated_at = utcnow()
    
    def start_execution(self) -> None:
        """Mark execution as started"""
        self.status = AgentExecutionStatus.RUNNING.value
        self.started_at = utcnow()
        self.updated_at = utcnow()
    
    def complete_execution(self, result: Optional[str] = None) -> None:
        """Mark execution as completed successfully"""
        self.status = AgentExecutionStatus.COMPLETED.value
        self.completed_at = utcnow()
        self.updated_at = utcnow()
        
        if result is not None:
            self.result = result
        
        # Calculate execution time
        if self.started_at:
            self.execution_time_seconds = (self.completed_at - self.started_at).total_seconds()
    
    def fail_execution(self, error_message: Optional[str] = None) -> None:
        """Mark execution as failed"""
        self.status = AgentExecutionStatus.FAILED.value
        self.completed_at = utcnow()
        self.updated_at = utcnow()
        
        if error_message is not None:
            self.error_message = error_message
        
        # Calculate execution time
        if self.started_at:
            self.execution_time_seconds = (self.completed_at - self.started_at).total_seconds()
    
    def cancel_execution(self) -> None:
        """Mark execution as cancelled"""
        self.status = AgentExecutionStatus.CANCELLED.value
        self.completed_at = utcnow()
        self.updated_at = utcnow()
        
        # Calculate execution time
        if self.started_at:
            self.execution_time_seconds = (self.completed_at - self.started_at).total_seconds()
    
    def add_log_entry(self, level: str, message: str, timestamp: Optional[datetime] = None) -> None:
        """Add a log entry to the execution"""
        if not self.logs:
            self.logs = []
        
        log_entry = {
            'level': level,
            'message': message,
            'timestamp': (timestamp or utcnow()).isoformat()
        }
        
        self.logs.append(log_entry)
        self.updated_at = utcnow()
    
    def add_tool_log_entry(self, tool_name: str, arguments: Dict[str, Any], status: str, 
                          result: Optional[Dict[str, Any]] = None, execution_time: Optional[float] = None,
                          timestamp: Optional[datetime] = None) -> None:
        """Add a tool execution log entry"""
        if not self.tool_logs:
            self.tool_logs = []
        
        tool_log_entry = {
            'tool_name': tool_name,
            'arguments': arguments,
            'status': status,  # 'started', 'completed', 'failed', 'error'
            'result': result,
            'execution_time': execution_time,
            'timestamp': (timestamp or utcnow()).isoformat()
        }
        
        self.tool_logs.append(tool_log_entry)
        self.updated_at = utcnow()
    
    def get_tool_logs(self) -> List[Dict[str, Any]]:
        """Get all tool execution logs"""
        return self.tool_logs or []
    
    def get_tool_logs_by_status(self, status: str) -> List[Dict[str, Any]]:
        """Get tool logs filtered by status"""
        return [log for log in (self.tool_logs or []) if log.get('status') == status]
    
    def get_failed_tool_logs(self) -> List[Dict[str, Any]]:
        """Get all failed tool execution logs"""
        return [log for log in (self.tool_logs or []) if log.get('status') in ['failed', 'error']]
    
    def get_successful_tool_logs(self) -> List[Dict[str, Any]]:
        """Get all successful tool execution logs"""
        return [log for log in (self.tool_logs or []) if log.get('status') == 'completed']
    
    def add_metadata(self, key: str, value: Any) -> None:
        """Add metadata to the execution"""
        if not self.execution_metadata:
            self.execution_metadata = {}
        
        self.execution_metadata[key] = value
        self.updated_at = utcnow()
    
    def get_metadata(self, key: str, default=None):
        """Get metadata value"""
        return (self.execution_metadata or {}).get(key, default)
    
    def increment_llm_requests(self, tokens_used: Optional[int] = None, cost: Optional[float] = None) -> None:
        """Increment LLM request counter and update usage stats"""
        self.llm_requests = (self.llm_requests or 0) + 1
        
        if tokens_used is not None:
            self.tokens_used = (self.tokens_used or 0) + tokens_used
        
        if cost is not None:
            self.llm_cost = (self.llm_cost or 0.0) + cost
        
        self.updated_at = utcnow()
    
    def is_running(self) -> bool:
        """Check if execution is currently running"""
        return self.status == AgentExecutionStatus.RUNNING.value
    
    def is_completed(self) -> bool:
        """Check if execution is completed (successfully or failed)"""
        return self.status in [
            AgentExecutionStatus.COMPLETED.value,
            AgentExecutionStatus.FAILED.value,
            AgentExecutionStatus.CANCELLED.value
        ]
    
    def is_successful(self) -> bool:
        """Check if execution completed successfully"""
        return self.status == AgentExecutionStatus.COMPLETED.value
    
    def is_failed(self) -> bool:
        """Check if execution failed"""
        return self.status == AgentExecutionStatus.FAILED.value
    
    def is_cancelled(self) -> bool:
        """Check if execution was cancelled"""
        return self.status == AgentExecutionStatus.CANCELLED.value
    
    def can_be_cancelled(self) -> bool:
        """Check if execution can be cancelled"""
        return self.status in [
            AgentExecutionStatus.PENDING.value,
            AgentExecutionStatus.RUNNING.value
        ]
    
    def get_duration(self) -> Optional[float]:
        """Get execution duration in seconds"""
        if self.execution_time_seconds is not None:
            return self.execution_time_seconds
        
        if self.started_at and self.completed_at:
            return (self.completed_at - self.started_at).total_seconds()
        
        if self.started_at and not self.completed_at:
            # Still running
            return (utcnow() - self.started_at).total_seconds()
        
        return None
    
    def get_cost_per_token(self) -> Optional[float]:
        """Get cost per token if available"""
        if self.llm_cost and self.tokens_used and self.tokens_used > 0:
            return self.llm_cost / self.tokens_used
        return None
    
    def get_tokens_per_second(self) -> Optional[float]:
        """Get tokens processed per second"""
        duration = self.get_duration()
        if duration and self.tokens_used and duration > 0:
            return self.tokens_used / duration
        return None
    
    def get_summary(self) -> Dict[str, Any]:
        """Get execution summary"""
        return {
            'id': self.id,
            'status': self.status,
            'duration_seconds': self.get_duration(),
            'tokens_used': self.tokens_used,
            'llm_requests': self.llm_requests,
            'cost': self.llm_cost,
            'cost_per_token': self.get_cost_per_token(),
            'tokens_per_second': self.get_tokens_per_second(),
            'has_result': bool(self.result),
            'has_error': bool(self.error_message),
            'log_entries': len(self.logs or [])
        }
    
    def get_performance_metrics(self) -> Dict[str, Any]:
        """Get performance metrics"""
        duration = self.get_duration()
        
        return {
            'execution_time': duration,
            'tokens_used': self.tokens_used or 0,
            'llm_requests': self.llm_requests or 0,
            'tokens_per_request': (self.tokens_used / self.llm_requests) if (self.tokens_used and self.llm_requests) else 0,
            'tokens_per_second': self.get_tokens_per_second() or 0,
            'cost': self.llm_cost or 0.0,
            'cost_per_token': self.get_cost_per_token() or 0.0,
            'efficiency_score': self._calculate_efficiency_score()
        }
    
    def _calculate_efficiency_score(self) -> float:
        """Calculate efficiency score (0-100)"""
        # Simple efficiency calculation based on tokens per second and cost
        tokens_per_second = self.get_tokens_per_second() or 0
        cost_per_token = self.get_cost_per_token() or 0
        
        # Normalize and combine metrics (this is a simplified example)
        speed_score = min(tokens_per_second * 10, 50)  # Max 50 points for speed
        cost_score = max(50 - (cost_per_token * 10000), 0)  # Max 50 points for low cost
        
        return speed_score + cost_score
