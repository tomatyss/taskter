"""
Agent-related Pydantic schemas for request/response validation and serialization.
"""

from typing import Optional, List, Dict, Any
from datetime import datetime
from pydantic import BaseModel, Field, validator
from app.core.constants import LLMProvider, ToolName


class AgentCreateSchema(BaseModel):
    """Schema for creating a new agent"""
    name: str = Field(..., min_length=1, max_length=100, description="Agent name")
    description: Optional[str] = Field(None, max_length=500, description="Agent description")
    system_instructions: str = Field(..., min_length=1, description="System instructions for the agent")
    llm_provider: LLMProvider = Field(..., description="LLM provider to use")
    llm_model: str = Field(..., min_length=1, description="LLM model name")
    llm_api_key: Optional[str] = Field(None, description="API key for the LLM provider")
    available_tools: List[str] = Field(default_factory=list, description="List of available tools")
    config: Dict[str, Any] = Field(default_factory=dict, description="Agent configuration")
    is_active: bool = Field(True, description="Whether the agent is active")

    @validator('available_tools')
    def validate_tools(cls, v):
        """Validate that all tools are valid"""
        valid_tools = [tool.value for tool in ToolName]
        invalid_tools = [tool for tool in v if tool not in valid_tools]
        if invalid_tools:
            raise ValueError(f"Invalid tools: {invalid_tools}. Available: {valid_tools}")
        return v

    class Config:
        use_enum_values = True


class AgentUpdateSchema(BaseModel):
    """Schema for updating an existing agent"""
    name: Optional[str] = Field(None, min_length=1, max_length=100)
    description: Optional[str] = Field(None, max_length=500)
    system_instructions: Optional[str] = Field(None, min_length=1)
    llm_provider: Optional[LLMProvider] = None
    llm_model: Optional[str] = Field(None, min_length=1)
    llm_api_key: Optional[str] = None
    available_tools: Optional[List[str]] = None
    config: Optional[Dict[str, Any]] = None
    is_active: Optional[bool] = None

    @validator('available_tools')
    def validate_tools(cls, v):
        """Validate that all tools are valid"""
        if v is not None:
            valid_tools = [tool.value for tool in ToolName]
            invalid_tools = [tool for tool in v if tool not in valid_tools]
            if invalid_tools:
                raise ValueError(f"Invalid tools: {invalid_tools}. Available: {valid_tools}")
        return v

    class Config:
        use_enum_values = True


class AgentResponseSchema(BaseModel):
    """Schema for agent response data"""
    id: int
    name: str
    description: Optional[str]
    system_instructions: str
    llm_provider: str
    llm_model: str
    available_tools: List[str]
    config: Dict[str, Any]
    is_active: bool
    created_at: Optional[datetime]
    updated_at: Optional[datetime]

    class Config:
        from_attributes = True


class AgentListResponseSchema(BaseModel):
    """Schema for listing agents"""
    id: int
    name: str
    description: Optional[str]
    llm_provider: str
    llm_model: str
    available_tools: List[str]
    is_active: bool
    created_at: Optional[datetime]

    class Config:
        from_attributes = True


class ToolResponseSchema(BaseModel):
    """Schema for tool information"""
    name: str
    description: str
    input_schema: Dict[str, Any]


class ProviderResponseSchema(BaseModel):
    """Schema for LLM provider information"""
    name: str
    default_model: str


class TaskAssignmentSchema(BaseModel):
    """Schema for assigning a task to an agent"""
    agent_id: int = Field(..., gt=0, description="ID of the agent to assign the task to")


# Utility functions for converting models to schemas
def agent_to_response_schema(agent) -> AgentResponseSchema:
    """Convert Agent model to AgentResponseSchema"""
    return AgentResponseSchema(
        id=agent.id,
        name=agent.name,
        description=agent.description,
        system_instructions=agent.system_instructions,
        llm_provider=agent.llm_provider,
        llm_model=agent.llm_model,
        available_tools=agent.available_tools or [],
        config=agent.config or {},
        is_active=agent.is_active,
        created_at=agent.created_at,
        updated_at=agent.updated_at
    )


def agent_to_list_schema(agent) -> AgentListResponseSchema:
    """Convert Agent model to AgentListResponseSchema"""
    return AgentListResponseSchema(
        id=agent.id,
        name=agent.name,
        description=agent.description,
        llm_provider=agent.llm_provider,
        llm_model=agent.llm_model,
        available_tools=agent.available_tools or [],
        is_active=agent.is_active,
        created_at=agent.created_at
    )
