"""
Application constants and enums to replace magic strings
"""
from enum import Enum


class TaskStatus(str, Enum):
    """Task status enumeration"""
    TODO = "todo"
    IN_PROGRESS = "in_progress"
    BLOCKED = "blocked"
    DONE = "done"


class ExecutionStatus(str, Enum):
    """Task execution status enumeration"""
    MANUAL = "manual"
    ASSIGNED = "assigned"
    RUNNING = "running"
    COMPLETED = "completed"
    FAILED = "failed"


class AgentExecutionStatus(str, Enum):
    """Agent execution status enumeration"""
    PENDING = "pending"
    RUNNING = "running"
    COMPLETED = "completed"
    FAILED = "failed"
    STOPPED = "stopped"


class LLMProvider(str, Enum):
    """LLM provider enumeration"""
    OPENAI = "openai"
    ANTHROPIC = "anthropic"
    GEMINI = "gemini"


class ToolName(str, Enum):
    """Available tool names"""
    WEB_SEARCH = "web_search"
    SEND_EMAIL = "send_email"
    EXECUTE_SCRIPT = "execute_script"


class APIResponseStatus(str, Enum):
    """API response status"""
    SUCCESS = "success"
    ERROR = "error"


class LogLevel(str, Enum):
    """Logging levels"""
    DEBUG = "DEBUG"
    INFO = "INFO"
    WARNING = "WARNING"
    ERROR = "ERROR"
    CRITICAL = "CRITICAL"


class Environment(str, Enum):
    """Application environments"""
    DEVELOPMENT = "development"
    TESTING = "testing"
    PRODUCTION = "production"


# Default models for each provider
DEFAULT_MODELS = {
    LLMProvider.OPENAI: "gpt-4.1",
    LLMProvider.ANTHROPIC: "claude-sonnet-4",
    LLMProvider.GEMINI: "gemini-2.5-pro"
}

# Available models for each provider
AVAILABLE_MODELS = {
    LLMProvider.OPENAI: [
        "gpt-4.1",      # Flagship GPT model for complex tasks
        "gpt-4o",       # Fast, intelligent, flexible GPT model
        "o4-mini",      # Faster, more affordable reasoning model
        "o3",           # Our most powerful reasoning model
        "o3-pro",       # Version of o3 with more compute for better responses
        "o3-mini",      # A small model alternative to o3
        "o1",           # Previous full o-series reasoning model
        "o1-pro"        # Version of o1 with more compute for better responses
    ],
    LLMProvider.ANTHROPIC: [
        "claude-opus-4",     # Most capable model
        "claude-sonnet-4",   # Balanced performance and speed
        "claude-3-7-sonnet", # Enhanced version with improved capabilities
        "claude-3-5-haiku",  # Fast and cost-effective
        "claude-3-5-sonnet", # High quality, natural conversational audio
        "claude-3-haiku"     # Fastest model for simple tasks
    ],
    LLMProvider.GEMINI: [
        "gemini-2.5-pro",   # Enhanced thinking and reasoning, multimodal understanding
        "gemini-2.5-flash", # Adaptive thinking, cost efficiency
        "gemini-2.0-flash"  # Next generation features, speed, and realtime streaming
    ]
}

# API response messages
API_MESSAGES = {
    "TASK_CREATED": "Task created successfully",
    "TASK_UPDATED": "Task updated successfully",
    "TASK_DELETED": "Task deleted successfully",
    "TASK_ASSIGNED": "Task assigned to agent successfully",
    "TASK_UNASSIGNED": "Task unassigned successfully",
    "AGENT_CREATED": "Agent created successfully",
    "AGENT_UPDATED": "Agent updated successfully",
    "AGENT_DELETED": "Agent deleted successfully",
    "EXECUTION_STARTED": "Agent execution started",
    "EXECUTION_STOPPED": "Agent execution stopped",
    "EXECUTION_COMPLETED": "Agent execution completed",
}

# Error messages
ERROR_MESSAGES = {
    "TASK_NOT_FOUND": "Task not found",
    "AGENT_NOT_FOUND": "Agent not found",
    "EXECUTION_NOT_FOUND": "Execution not found",
    "INVALID_STATUS": "Invalid status",
    "INVALID_PROVIDER": "Invalid LLM provider",
    "INVALID_TOOLS": "Invalid tools specified",
    "AGENT_NOT_ACTIVE": "Agent is not active",
    "TASK_ALREADY_RUNNING": "Task is currently running",
    "CANNOT_DELETE_RUNNING_AGENT": "Cannot delete agent with running executions",
    "CANNOT_UNASSIGN_RUNNING_TASK": "Cannot unassign running task",
    "REQUIRED_FIELD_MISSING": "Required field is missing",
    "SMTP_NOT_CONFIGURED": "SMTP credentials not configured",
    "API_KEY_MISSING": "API key is required",
    "MAX_ITERATIONS_REACHED": "Maximum iterations reached without completion",
}

# Validation constraints
VALIDATION_CONSTRAINTS = {
    "TASK_TITLE_MAX_LENGTH": 200,
    "AGENT_NAME_MAX_LENGTH": 100,
    "TOOL_NAME_MAX_LENGTH": 50,
    "MAX_ITERATIONS_DEFAULT": 20,
    "MAX_ITERATIONS_LIMIT": 100,
    "MAX_TOKENS_DEFAULT": 1000,
    "MAX_TOKENS_LIMIT": 4000,
    "TEMPERATURE_MIN": 0.0,
    "TEMPERATURE_MAX": 2.0,
    "EXECUTION_TIMEOUT_DEFAULT": 300,  # 5 minutes
    "EXECUTION_TIMEOUT_MAX": 3600,     # 1 hour
}

# Tool execution constraints
TOOL_CONSTRAINTS = {
    "SCRIPT_TIMEOUT_DEFAULT": 30,
    "SCRIPT_TIMEOUT_MAX": 60,
    "SEARCH_RESULTS_DEFAULT": 5,
    "SEARCH_RESULTS_MAX": 10,
    "EMAIL_RECIPIENTS_MAX": 50,
}

# Database constraints
DB_CONSTRAINTS = {
    "CONNECTION_POOL_SIZE": 10,
    "CONNECTION_MAX_OVERFLOW": 20,
    "CONNECTION_TIMEOUT": 30,
}

# Task completion indicators
TASK_COMPLETION_INDICATORS = [
    "TASK_COMPLETED",
    "task completed",
    "task is completed",
    "successfully completed",
    "finished the task"
]

# Indicators that signify a task is blocked
TASK_BLOCKED_INDICATORS = [
    "TASK_BLOCKED",
    "task blocked",
    "unable to proceed",
    "blocked"
]

# Dangerous code patterns for script execution
DANGEROUS_CODE_PATTERNS = [
    "import subprocess",
    "import os.system", 
    "import shutil",
    "import socket",
    "import threading",
    "import multiprocessing",
    "exec(",
    "eval(",
    "__import__",
    "open("
]

# Allowed imports for script execution
ALLOWED_SCRIPT_IMPORTS = {
    "json", "csv", "datetime", "time", "math", "random", "os", "sys",
    "requests", "urllib", "base64", "hashlib", "uuid", "re",
    "collections", "itertools", "functools", "operator"
}


# Utility functions for model management
def get_available_models_for_provider(provider: str) -> list:
    """Get list of available models for a specific provider"""
    provider_enum = LLMProvider(provider.lower())
    return AVAILABLE_MODELS.get(provider_enum, [])


def get_default_model_for_provider(provider: str) -> str:
    """Get default model for a specific provider"""
    provider_enum = LLMProvider(provider.lower())
    return DEFAULT_MODELS.get(provider_enum, "")


def is_valid_model_for_provider(provider: str, model: str) -> bool:
    """Check if a model is valid for a specific provider"""
    available_models = get_available_models_for_provider(provider)
    return model in available_models


def get_all_available_models() -> dict:
    """Get all available models organized by provider"""
    return {
        provider.value: models 
        for provider, models in AVAILABLE_MODELS.items()
    }


def get_model_info() -> dict:
    """Get comprehensive model information including defaults and available models"""
    return {
        "providers": [provider.value for provider in LLMProvider],
        "default_models": {provider.value: model for provider, model in DEFAULT_MODELS.items()},
        "available_models": get_all_available_models()
    }
