"""
Centralized logging configuration with structured logging support
"""
import logging
import logging.handlers
import sys
from typing import Optional
from datetime import datetime
import json
import traceback

from app.core.config import get_config


class StructuredFormatter(logging.Formatter):
    """Custom formatter for structured logging"""
    
    def format(self, record: logging.LogRecord) -> str:
        """Format log record as structured JSON"""
        log_data = {
            "timestamp": datetime.utcnow().isoformat() + "Z",
            "level": record.levelname,
            "logger": record.name,
            "message": record.getMessage(),
            "module": record.module,
            "function": record.funcName,
            "line": record.lineno,
        }
        
        # Add correlation ID if available
        if hasattr(record, 'correlation_id'):
            log_data["correlation_id"] = record.correlation_id
        
        # Add user ID if available
        if hasattr(record, 'user_id'):
            log_data["user_id"] = record.user_id
        
        # Add request ID if available
        if hasattr(record, 'request_id'):
            log_data["request_id"] = record.request_id
        
        # Add execution ID if available
        if hasattr(record, 'execution_id'):
            log_data["execution_id"] = record.execution_id
        
        # Add task ID if available
        if hasattr(record, 'task_id'):
            log_data["task_id"] = record.task_id
        
        # Add agent ID if available
        if hasattr(record, 'agent_id'):
            log_data["agent_id"] = record.agent_id
        
        # Add exception information if present
        if record.exc_info:
            log_data["exception"] = {
                "type": record.exc_info[0].__name__,
                "message": str(record.exc_info[1]),
                "traceback": traceback.format_exception(*record.exc_info)
            }
        
        # Add extra fields
        if hasattr(record, 'extra_data'):
            log_data["extra"] = record.extra_data
        
        return json.dumps(log_data, ensure_ascii=False)


class StandardFormatter(logging.Formatter):
    """Standard text formatter for development"""
    
    def __init__(self):
        super().__init__(
            fmt="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
            datefmt="%Y-%m-%d %H:%M:%S"
        )


class ContextFilter(logging.Filter):
    """Filter to add context information to log records"""
    
    def __init__(self):
        super().__init__()
        self.correlation_id = None
        self.user_id = None
        self.request_id = None
    
    def filter(self, record: logging.LogRecord) -> bool:
        """Add context information to the log record"""
        if self.correlation_id:
            record.correlation_id = self.correlation_id
        if self.user_id:
            record.user_id = self.user_id
        if self.request_id:
            record.request_id = self.request_id
        return True
    
    def set_correlation_id(self, correlation_id: str):
        """Set correlation ID for request tracking"""
        self.correlation_id = correlation_id
    
    def set_user_id(self, user_id: str):
        """Set user ID for user tracking"""
        self.user_id = user_id
    
    def set_request_id(self, request_id: str):
        """Set request ID for request tracking"""
        self.request_id = request_id
    
    def clear_context(self):
        """Clear all context information"""
        self.correlation_id = None
        self.user_id = None
        self.request_id = None


# Global context filter instance
context_filter = ContextFilter()


def setup_logging():
    """Setup application logging configuration"""
    config = get_config()
    
    # Get root logger
    root_logger = logging.getLogger()
    root_logger.setLevel(getattr(logging, config.logging.level))
    
    # Clear existing handlers
    root_logger.handlers.clear()
    
    # Console handler
    console_handler = logging.StreamHandler(sys.stdout)
    
    if config.is_production:
        # Use structured logging in production
        console_handler.setFormatter(StructuredFormatter())
    else:
        # Use standard formatting in development
        console_handler.setFormatter(StandardFormatter())
    
    console_handler.addFilter(context_filter)
    root_logger.addHandler(console_handler)
    
    # File handler if configured
    if config.logging.file_path:
        file_handler = logging.handlers.RotatingFileHandler(
            filename=config.logging.file_path,
            maxBytes=config.logging.max_file_size,
            backupCount=config.logging.backup_count,
            encoding='utf-8'
        )
        
        if config.is_production:
            file_handler.setFormatter(StructuredFormatter())
        else:
            file_handler.setFormatter(StandardFormatter())
        
        file_handler.addFilter(context_filter)
        root_logger.addHandler(file_handler)
    
    # Set specific logger levels
    logging.getLogger('werkzeug').setLevel(logging.WARNING)
    logging.getLogger('urllib3').setLevel(logging.WARNING)
    logging.getLogger('requests').setLevel(logging.WARNING)
    
    # Application loggers
    logging.getLogger('app').setLevel(getattr(logging, config.logging.level))


def get_logger(name: str) -> logging.Logger:
    """Get a logger with the specified name"""
    return logging.getLogger(name)


def log_execution_start(logger: logging.Logger, execution_id: int, task_id: int, agent_id: int):
    """Log execution start with context"""
    logger.info(
        "Agent execution started",
        extra={
            'execution_id': execution_id,
            'task_id': task_id,
            'agent_id': agent_id,
            'extra_data': {
                'event': 'execution_start'
            }
        }
    )


def log_execution_complete(logger: logging.Logger, execution_id: int, 
                          iterations: int, tokens_used: int, execution_time: float):
    """Log execution completion with metrics"""
    logger.info(
        "Agent execution completed",
        extra={
            'execution_id': execution_id,
            'extra_data': {
                'event': 'execution_complete',
                'iterations': iterations,
                'tokens_used': tokens_used,
                'execution_time_seconds': execution_time
            }
        }
    )


def log_execution_error(logger: logging.Logger, execution_id: int, error: Exception):
    """Log execution error with details"""
    logger.error(
        f"Agent execution failed: {str(error)}",
        exc_info=True,
        extra={
            'execution_id': execution_id,
            'extra_data': {
                'event': 'execution_error',
                'error_type': error.__class__.__name__
            }
        }
    )


def log_tool_execution(logger: logging.Logger, tool_name: str, 
                      execution_id: Optional[int] = None, success: bool = True, 
                      execution_time: Optional[float] = None):
    """Log tool execution"""
    message = f"Tool '{tool_name}' executed {'successfully' if success else 'with error'}"
    
    extra_data = {
        'event': 'tool_execution',
        'tool_name': tool_name,
        'success': success
    }
    
    if execution_time is not None:
        extra_data['execution_time_seconds'] = execution_time
    
    extra = {'extra_data': extra_data}
    if execution_id:
        extra['execution_id'] = execution_id
    
    if success:
        logger.info(message, extra=extra)
    else:
        logger.error(message, extra=extra)


def log_api_request(logger: logging.Logger, method: str, path: str, 
                   status_code: int, response_time: float, 
                   request_id: Optional[str] = None):
    """Log API request"""
    message = f"{method} {path} - {status_code}"
    
    extra_data = {
        'event': 'api_request',
        'method': method,
        'path': path,
        'status_code': status_code,
        'response_time_ms': response_time * 1000
    }
    
    extra = {'extra_data': extra_data}
    if request_id:
        extra['request_id'] = request_id
    
    if status_code < 400:
        logger.info(message, extra=extra)
    elif status_code < 500:
        logger.warning(message, extra=extra)
    else:
        logger.error(message, extra=extra)


def log_database_operation(logger: logging.Logger, operation: str, table: str, 
                          success: bool = True, execution_time: Optional[float] = None):
    """Log database operation"""
    message = f"Database {operation} on {table} {'succeeded' if success else 'failed'}"
    
    extra_data = {
        'event': 'database_operation',
        'operation': operation,
        'table': table,
        'success': success
    }
    
    if execution_time is not None:
        extra_data['execution_time_seconds'] = execution_time
    
    if success:
        logger.debug(message, extra={'extra_data': extra_data})
    else:
        logger.error(message, extra={'extra_data': extra_data})


def log_llm_api_call(logger: logging.Logger, provider: str, model: str, 
                    tokens_used: int, success: bool = True, 
                    execution_time: Optional[float] = None):
    """Log LLM API call"""
    message = f"LLM API call to {provider}/{model} {'succeeded' if success else 'failed'}"
    
    extra_data = {
        'event': 'llm_api_call',
        'provider': provider,
        'model': model,
        'tokens_used': tokens_used,
        'success': success
    }
    
    if execution_time is not None:
        extra_data['execution_time_seconds'] = execution_time
    
    if success:
        logger.info(message, extra={'extra_data': extra_data})
    else:
        logger.error(message, extra={'extra_data': extra_data})


# Context managers for logging context
class LoggingContext:
    """Context manager for setting logging context"""
    
    def __init__(self, correlation_id: Optional[str] = None, 
                 user_id: Optional[str] = None, 
                 request_id: Optional[str] = None):
        self.correlation_id = correlation_id
        self.user_id = user_id
        self.request_id = request_id
        self.previous_correlation_id = None
        self.previous_user_id = None
        self.previous_request_id = None
    
    def __enter__(self):
        # Save previous context
        self.previous_correlation_id = context_filter.correlation_id
        self.previous_user_id = context_filter.user_id
        self.previous_request_id = context_filter.request_id
        
        # Set new context
        if self.correlation_id:
            context_filter.set_correlation_id(self.correlation_id)
        if self.user_id:
            context_filter.set_user_id(self.user_id)
        if self.request_id:
            context_filter.set_request_id(self.request_id)
        
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        # Restore previous context
        context_filter.correlation_id = self.previous_correlation_id
        context_filter.user_id = self.previous_user_id
        context_filter.request_id = self.previous_request_id


def with_logging_context(correlation_id: Optional[str] = None,
                        user_id: Optional[str] = None,
                        request_id: Optional[str] = None):
    """Decorator for setting logging context"""
    def decorator(func):
        def wrapper(*args, **kwargs):
            with LoggingContext(correlation_id, user_id, request_id):
                return func(*args, **kwargs)
        return wrapper
    return decorator
