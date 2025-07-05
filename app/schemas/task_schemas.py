"""
Pydantic schemas for task data validation and serialization
"""
from typing import Optional, List, Dict, Any
from datetime import datetime
from pydantic import BaseModel, Field, validator

from app.core.constants import TaskStatus, ExecutionStatus


class TaskBase(BaseModel):
    """Base task schema with common fields"""
    title: str = Field(..., min_length=1, max_length=200, description="Task title")
    description: Optional[str] = Field(None, description="Task description")
    status: TaskStatus = Field(default=TaskStatus.TODO, description="Task status")


class TaskCreate(TaskBase):
    """Schema for creating a new task"""
    
    @validator('title')
    def validate_title(cls, v):
        if not v or not v.strip():
            raise ValueError('Title cannot be empty')
        return v.strip()
    
    @validator('description')
    def validate_description(cls, v):
        if v is not None:
            return v.strip() if v.strip() else None
        return v


class TaskUpdate(BaseModel):
    """Schema for updating a task"""
    title: Optional[str] = Field(None, min_length=1, max_length=200)
    description: Optional[str] = Field(None)
    status: Optional[TaskStatus] = Field(None)
    
    @validator('title')
    def validate_title(cls, v):
        if v is not None:
            if not v or not v.strip():
                raise ValueError('Title cannot be empty')
            return v.strip()
        return v
    
    @validator('description')
    def validate_description(cls, v):
        if v is not None:
            return v.strip() if v.strip() else None
        return v


class TaskAssignment(BaseModel):
    """Schema for task assignment"""
    agent_id: int = Field(..., gt=0, description="Agent ID to assign task to")


class TaskStatusUpdate(BaseModel):
    """Schema for updating task status"""
    status: TaskStatus = Field(..., description="New task status")


class TaskBulkUpdate(BaseModel):
    """Schema for bulk task updates"""
    task_ids: List[int] = Field(..., min_items=1, description="List of task IDs")
    status: Optional[TaskStatus] = Field(None, description="New status for all tasks")
    agent_id: Optional[int] = Field(None, gt=0, description="Agent ID to assign tasks to")
    
    @validator('task_ids')
    def validate_task_ids(cls, v):
        if not v:
            raise ValueError('At least one task ID is required')
        if len(set(v)) != len(v):
            raise ValueError('Duplicate task IDs are not allowed')
        return v


class TaskExecutionSummary(BaseModel):
    """Schema for task execution summary"""
    total_executions: int = Field(..., ge=0)
    successful_executions: int = Field(..., ge=0)
    failed_executions: int = Field(..., ge=0)
    total_tokens_used: int = Field(..., ge=0)
    total_execution_time: float = Field(..., ge=0)


class TaskResponse(TaskBase):
    """Schema for task response"""
    id: int = Field(..., description="Task ID")
    execution_status: ExecutionStatus = Field(..., description="Task execution status")
    assigned_agent_id: Optional[int] = Field(None, description="Assigned agent ID")
    created_at: datetime = Field(..., description="Creation timestamp")
    updated_at: datetime = Field(..., description="Last update timestamp")
    execution_summary: Optional[TaskExecutionSummary] = Field(None, description="Execution summary")
    
    class Config:
        from_attributes = True
        json_encoders = {
            datetime: lambda v: v.isoformat()
        }


class TaskListResponse(BaseModel):
    """Schema for task list response"""
    tasks: List[TaskResponse] = Field(..., description="List of tasks")
    total: int = Field(..., ge=0, description="Total number of tasks")


class TaskSearchRequest(BaseModel):
    """Schema for task search request"""
    query: str = Field(..., min_length=1, description="Search query")
    limit: Optional[int] = Field(None, ge=1, le=100, description="Maximum results")
    offset: Optional[int] = Field(None, ge=0, description="Results offset")
    
    @validator('query')
    def validate_query(cls, v):
        if not v or not v.strip():
            raise ValueError('Search query cannot be empty')
        return v.strip()


class TaskFilterRequest(BaseModel):
    """Schema for task filtering request"""
    status: Optional[TaskStatus] = Field(None, description="Filter by status")
    execution_status: Optional[ExecutionStatus] = Field(None, description="Filter by execution status")
    agent_id: Optional[int] = Field(None, gt=0, description="Filter by assigned agent")
    search: Optional[str] = Field(None, min_length=1, description="Search in title/description")
    page: int = Field(default=1, ge=1, description="Page number")
    per_page: int = Field(default=20, ge=1, le=100, description="Items per page")
    
    @validator('search')
    def validate_search(cls, v):
        if v is not None:
            return v.strip() if v.strip() else None
        return v


class TaskStatistics(BaseModel):
    """Schema for task statistics"""
    total_tasks: int = Field(..., ge=0)
    by_status: Dict[str, int] = Field(..., description="Task count by status")
    by_execution_status: Dict[str, int] = Field(..., description="Task count by execution status")
    assigned_tasks: int = Field(..., ge=0)
    unassigned_tasks: int = Field(..., ge=0)


class AgentTaskStatistics(BaseModel):
    """Schema for agent-specific task statistics"""
    total_assigned: int = Field(..., ge=0)
    by_status: Dict[str, int] = Field(..., description="Task count by status")
    by_execution_status: Dict[str, int] = Field(..., description="Task count by execution status")


class TaskDateRangeRequest(BaseModel):
    """Schema for date range task requests"""
    start_date: datetime = Field(..., description="Start date")
    end_date: datetime = Field(..., description="End date")
    date_field: str = Field(default="created_at", description="Date field to filter on")
    
    @validator('date_field')
    def validate_date_field(cls, v):
        if v not in ['created_at', 'updated_at']:
            raise ValueError('date_field must be "created_at" or "updated_at"')
        return v
    
    @validator('end_date')
    def validate_date_range(cls, v, values):
        if 'start_date' in values and v <= values['start_date']:
            raise ValueError('end_date must be after start_date')
        return v


class PaginationInfo(BaseModel):
    """Schema for pagination information"""
    page: int = Field(..., ge=1)
    per_page: int = Field(..., ge=1)
    total: int = Field(..., ge=0)
    pages: int = Field(..., ge=0)
    has_prev: bool = Field(...)
    has_next: bool = Field(...)
    prev_num: Optional[int] = Field(None)
    next_num: Optional[int] = Field(None)


class PaginatedTaskResponse(BaseModel):
    """Schema for paginated task response"""
    items: List[TaskResponse] = Field(..., description="Task items")
    pagination: PaginationInfo = Field(..., description="Pagination information")


# Response schemas for different operations
class TaskCreatedResponse(BaseModel):
    """Schema for task creation response"""
    message: str = Field(..., description="Success message")
    task: TaskResponse = Field(..., description="Created task")


class TaskUpdatedResponse(BaseModel):
    """Schema for task update response"""
    message: str = Field(..., description="Success message")
    task: TaskResponse = Field(..., description="Updated task")


class TaskDeletedResponse(BaseModel):
    """Schema for task deletion response"""
    message: str = Field(..., description="Success message")
    task_id: int = Field(..., description="Deleted task ID")


class TaskAssignedResponse(BaseModel):
    """Schema for task assignment response"""
    message: str = Field(..., description="Success message")
    task: TaskResponse = Field(..., description="Assigned task")


class TaskBulkOperationResponse(BaseModel):
    """Schema for bulk operation response"""
    message: str = Field(..., description="Success message")
    affected_count: int = Field(..., ge=0, description="Number of affected tasks")
    task_ids: List[int] = Field(..., description="List of affected task IDs")


class TaskExecutionResponse(BaseModel):
    """Schema for task execution response"""
    message: str = Field(..., description="Success message")
    task: TaskResponse = Field(..., description="Task with updated execution status")


# Error response schemas
class TaskErrorResponse(BaseModel):
    """Schema for task error responses"""
    error: str = Field(..., description="Error type")
    message: str = Field(..., description="Error message")
    details: Optional[Dict[str, Any]] = Field(None, description="Additional error details")


class ValidationErrorResponse(BaseModel):
    """Schema for validation error responses"""
    error: str = Field(default="validation_error")
    message: str = Field(..., description="Validation error message")
    field_errors: Optional[Dict[str, List[str]]] = Field(None, description="Field-specific errors")


# Utility functions for schema conversion
def task_to_response(task, include_execution_summary: bool = False) -> TaskResponse:
    """Convert Task model to TaskResponse schema"""
    data = {
        'id': task.id,
        'title': task.title,
        'description': task.description,
        'status': TaskStatus(task.status),
        'execution_status': ExecutionStatus(task.execution_status),
        'assigned_agent_id': task.assigned_agent_id,
        'created_at': task.created_at,
        'updated_at': task.updated_at
    }
    
    if include_execution_summary:
        summary = task.get_execution_summary()
        data['execution_summary'] = TaskExecutionSummary(**summary)
    
    return TaskResponse(**data)


def tasks_to_list_response(tasks: List, total: int = None) -> TaskListResponse:
    """Convert list of tasks to TaskListResponse schema"""
    task_responses = [task_to_response(task) for task in tasks]
    return TaskListResponse(
        tasks=task_responses,
        total=total if total is not None else len(tasks)
    )
