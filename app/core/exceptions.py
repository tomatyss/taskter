"""
Custom exception hierarchy for the application
"""


class TaskterException(Exception):
    """Base exception for all application-specific errors"""
    
    def __init__(self, message: str, code: str = None, details: dict = None):
        self.message = message
        self.code = code or self.__class__.__name__
        self.details = details or {}
        super().__init__(self.message)


class ValidationError(TaskterException):
    """Raised when input validation fails"""
    pass


class NotFoundError(TaskterException):
    """Raised when a requested resource is not found"""
    pass


class ConflictError(TaskterException):
    """Raised when there's a conflict with the current state"""
    pass


class ConfigurationError(TaskterException):
    """Raised when there's a configuration issue"""
    pass


class AuthenticationError(TaskterException):
    """Raised when authentication fails"""
    pass


class AuthorizationError(TaskterException):
    """Raised when authorization fails"""
    pass


# Task-specific exceptions
class TaskNotFoundError(NotFoundError):
    """Raised when a task is not found"""
    
    def __init__(self, task_id: int):
        super().__init__(f"Task with ID {task_id} not found", "TASK_NOT_FOUND")


class TaskValidationError(ValidationError):
    """Raised when task validation fails"""
    pass


class TaskStatusError(ConflictError):
    """Raised when task status transition is invalid"""
    pass


# Agent-specific exceptions
class AgentNotFoundError(NotFoundError):
    """Raised when an agent is not found"""
    
    def __init__(self, agent_id: int):
        super().__init__(f"Agent with ID {agent_id} not found", "AGENT_NOT_FOUND")


class AgentValidationError(ValidationError):
    """Raised when agent validation fails"""
    pass


class AgentNotActiveError(ConflictError):
    """Raised when trying to use an inactive agent"""
    
    def __init__(self, agent_id: int):
        super().__init__(f"Agent with ID {agent_id} is not active", "AGENT_NOT_ACTIVE")


class AgentCreationError(TaskterException):
    """Raised when agent creation fails"""
    pass


class AgentDeletionError(ConflictError):
    """Raised when agent cannot be deleted"""
    pass


# Execution-specific exceptions
class ExecutionNotFoundError(NotFoundError):
    """Raised when an execution is not found"""
    
    def __init__(self, execution_id: int):
        super().__init__(f"Execution with ID {execution_id} not found", "EXECUTION_NOT_FOUND")


class ExecutionError(TaskterException):
    """Raised when execution fails"""
    pass


class ExecutionTimeoutError(ExecutionError):
    """Raised when execution times out"""
    
    def __init__(self, timeout: int):
        super().__init__(f"Execution timed out after {timeout} seconds", "EXECUTION_TIMEOUT")


class MaxIterationsReachedError(ExecutionError):
    """Raised when maximum iterations are reached"""
    
    def __init__(self, max_iterations: int):
        super().__init__(
            f"Maximum iterations ({max_iterations}) reached without completion",
            "MAX_ITERATIONS_REACHED"
        )


# LLM Provider exceptions
class LLMProviderError(TaskterException):
    """Base exception for LLM provider errors"""
    pass


class InvalidProviderError(LLMProviderError):
    """Raised when an invalid LLM provider is specified"""
    
    def __init__(self, provider: str, available_providers: list):
        super().__init__(
            f"Invalid LLM provider '{provider}'. Available providers: {available_providers}",
            "INVALID_PROVIDER"
        )


class LLMAPIError(LLMProviderError):
    """Raised when LLM API call fails"""
    pass


class LLMQuotaExceededError(LLMAPIError):
    """Raised when LLM API quota is exceeded"""
    pass


class LLMAuthenticationError(LLMAPIError):
    """Raised when LLM API authentication fails"""
    pass


# Tool-specific exceptions
class ToolError(TaskterException):
    """Base exception for tool errors"""
    pass


class ToolNotFoundError(ToolError):
    """Raised when a tool is not found"""
    
    def __init__(self, tool_name: str):
        super().__init__(f"Tool '{tool_name}' not found", "TOOL_NOT_FOUND")


class InvalidToolError(ToolError):
    """Raised when invalid tools are specified"""
    
    def __init__(self, invalid_tools: list, available_tools: list):
        super().__init__(
            f"Invalid tools: {invalid_tools}. Available tools: {available_tools}",
            "INVALID_TOOLS"
        )


class ToolExecutionError(ToolError):
    """Raised when tool execution fails"""
    pass


class ToolConfigurationError(ToolError):
    """Raised when tool configuration is invalid"""
    pass


class ScriptExecutionError(ToolExecutionError):
    """Raised when script execution fails"""
    pass


class DangerousCodeError(ScriptExecutionError):
    """Raised when dangerous code is detected in script"""
    
    def __init__(self, pattern: str):
        super().__init__(
            f"Potentially dangerous code detected: {pattern}",
            "DANGEROUS_CODE_DETECTED"
        )


class EmailError(ToolExecutionError):
    """Raised when email sending fails"""
    pass


class SMTPNotConfiguredError(EmailError):
    """Raised when SMTP is not configured"""
    
    def __init__(self):
        super().__init__("SMTP credentials not configured", "SMTP_NOT_CONFIGURED")


class WebSearchError(ToolExecutionError):
    """Raised when web search fails"""
    pass


# Database exceptions
class DatabaseError(TaskterException):
    """Base exception for database errors"""
    pass


class DatabaseConnectionError(DatabaseError):
    """Raised when database connection fails"""
    pass


class DatabaseIntegrityError(DatabaseError):
    """Raised when database integrity constraint is violated"""
    pass


# API exceptions
class APIError(TaskterException):
    """Base exception for API errors"""
    pass


class BadRequestError(APIError):
    """Raised for HTTP 400 Bad Request errors"""
    
    def __init__(self, message: str, details: dict = None):
        super().__init__(message, "BAD_REQUEST", details)


class UnauthorizedError(APIError):
    """Raised for HTTP 401 Unauthorized errors"""
    
    def __init__(self, message: str = "Unauthorized"):
        super().__init__(message, "UNAUTHORIZED")


class ForbiddenError(APIError):
    """Raised for HTTP 403 Forbidden errors"""
    
    def __init__(self, message: str = "Forbidden"):
        super().__init__(message, "FORBIDDEN")


class MethodNotAllowedError(APIError):
    """Raised for HTTP 405 Method Not Allowed errors"""
    
    def __init__(self, method: str, allowed_methods: list):
        super().__init__(
            f"Method {method} not allowed. Allowed methods: {allowed_methods}",
            "METHOD_NOT_ALLOWED"
        )


class InternalServerError(APIError):
    """Raised for HTTP 500 Internal Server Error"""
    
    def __init__(self, message: str = "Internal server error"):
        super().__init__(message, "INTERNAL_SERVER_ERROR")


# Utility functions for exception handling
def get_exception_details(exc: Exception) -> dict:
    """Get detailed information about an exception"""
    details = {
        "type": exc.__class__.__name__,
        "message": str(exc)
    }
    
    if isinstance(exc, TaskterException):
        details.update({
            "code": exc.code,
            "details": exc.details
        })
    
    return details


def is_client_error(exc: Exception) -> bool:
    """Check if exception represents a client error (4xx)"""
    client_errors = (
        ValidationError,
        NotFoundError,
        BadRequestError,
        UnauthorizedError,
        ForbiddenError,
        MethodNotAllowedError,
        ConflictError
    )
    return isinstance(exc, client_errors)


def is_server_error(exc: Exception) -> bool:
    """Check if exception represents a server error (5xx)"""
    server_errors = (
        ConfigurationError,
        DatabaseError,
        LLMAPIError,
        ToolExecutionError,
        ExecutionError,
        InternalServerError
    )
    return isinstance(exc, server_errors)
