"""
Schemas package initialization.
Exports all schema classes for easy importing.
"""

# Task schemas
from .task_schemas import (
    TaskCreate,
    TaskUpdate,
    TaskStatusUpdate,
    TaskResponse,
    TaskListResponse,
    TaskStatistics,
    PaginatedTaskResponse,
    TaskAssignment,
    TaskFilterRequest,
    task_to_response,
    tasks_to_list_response
)

# Agent schemas
from .agent_schemas import (
    AgentCreateSchema,
    AgentUpdateSchema,
    AgentResponseSchema,
    AgentListResponseSchema,
    TaskAssignmentSchema,
    agent_to_response_schema,
    agent_to_list_schema
)

# Execution schemas
from .execution_schemas import (
    ExecutionResponseSchema,
    ExecutionListResponseSchema,
    ExecutionQuerySchema,
    PaginatedExecutionResponseSchema,
    execution_to_response_schema,
    execution_to_list_schema
)

__all__ = [
    # Task schemas
    'TaskCreate',
    'TaskUpdate', 
    'TaskStatusUpdate',
    'TaskResponse',
    'TaskListResponse',
    'TaskStatistics',
    'PaginatedTaskResponse',
    'TaskAssignment',
    'TaskFilterRequest',
    'task_to_response',
    'tasks_to_list_response',
    
    # Agent schemas
    'AgentCreateSchema',
    'AgentUpdateSchema',
    'AgentResponseSchema',
    'AgentListResponseSchema',
    'TaskAssignmentSchema',
    'agent_to_response_schema',
    'agent_to_list_schema',
    
    # Execution schemas
    'ExecutionResponseSchema',
    'ExecutionListResponseSchema',
    'ExecutionQuerySchema',
    'PaginatedExecutionResponseSchema',
    'execution_to_response_schema',
    'execution_to_list_schema'
]
