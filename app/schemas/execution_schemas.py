"""
Execution-related Pydantic schemas for request/response validation and serialization.
"""

from typing import Optional, List, Dict, Any
from datetime import datetime
from pydantic import BaseModel, Field
from app.core.constants import AgentExecutionStatus


class ExecutionResponseSchema(BaseModel):
    """Schema for execution response data"""
    id: int
    task_id: int
    task_title: Optional[str]
    agent_id: int
    agent_name: Optional[str]
    status: str
    conversation_log: Optional[List[Dict[str, Any]]]
    result: Optional[str]
    error_message: Optional[str]
    iterations_count: int
    tokens_used: int
    execution_time_seconds: Optional[float]
    started_at: Optional[datetime]
    completed_at: Optional[datetime]
    created_at: Optional[datetime]

    class Config:
        from_attributes = True


class ExecutionListResponseSchema(BaseModel):
    """Schema for listing executions"""
    id: int
    task_id: int
    task_title: Optional[str]
    agent_id: int
    agent_name: Optional[str]
    status: str
    iterations_count: int
    tokens_used: int
    execution_time_seconds: Optional[float]
    started_at: Optional[datetime]
    completed_at: Optional[datetime]
    error_message: Optional[str]

    class Config:
        from_attributes = True


class ExecutionQuerySchema(BaseModel):
    """Schema for execution query parameters"""
    page: int = Field(1, ge=1, description="Page number")
    per_page: int = Field(20, ge=1, le=100, description="Items per page")
    status: Optional[AgentExecutionStatus] = Field(None, description="Filter by status")
    agent_id: Optional[int] = Field(None, ge=1, description="Filter by agent ID")
    task_id: Optional[int] = Field(None, ge=1, description="Filter by task ID")

    class Config:
        use_enum_values = True


class PaginatedExecutionResponseSchema(BaseModel):
    """Schema for paginated execution responses"""
    executions: List[ExecutionListResponseSchema]
    pagination: Dict[str, Any]


# Utility functions for converting models to schemas
def execution_to_response_schema(execution) -> ExecutionResponseSchema:
    """Convert AgentExecution model to ExecutionResponseSchema"""
    return ExecutionResponseSchema(
        id=execution.id,
        task_id=execution.task_id,
        task_title=execution.task.title if execution.task else None,
        agent_id=execution.agent_id,
        agent_name=execution.agent.name if execution.agent else None,
        status=execution.status,
        conversation_log=execution.conversation_log or [],
        result=execution.result,
        error_message=execution.error_message,
        iterations_count=execution.iterations_count,
        tokens_used=execution.tokens_used,
        execution_time_seconds=execution.execution_time_seconds,
        started_at=execution.started_at,
        completed_at=execution.completed_at,
        created_at=execution.created_at
    )


def execution_to_list_schema(execution) -> ExecutionListResponseSchema:
    """Convert AgentExecution model to ExecutionListResponseSchema"""
    return ExecutionListResponseSchema(
        id=execution.id,
        task_id=execution.task_id,
        task_title=execution.task.title if execution.task else None,
        agent_id=execution.agent_id,
        agent_name=execution.agent.name if execution.agent else None,
        status=execution.status,
        iterations_count=execution.iterations_count,
        tokens_used=execution.tokens_used,
        execution_time_seconds=execution.execution_time_seconds,
        started_at=execution.started_at,
        completed_at=execution.completed_at,
        error_message=execution.error_message
    )
