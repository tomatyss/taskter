"""
Common API response utilities for consistent response formatting.
"""

from typing import Any, Optional, Dict, List
from flask import jsonify
from pydantic import ValidationError
from app.core.exceptions import TaskterException
from app.core.constants import APIResponseStatus


class APIResponse:
    """Utility class for creating consistent API responses"""
    
    @staticmethod
    def success(data: Any = None, message: Optional[str] = None, status_code: int = 200):
        """Create a successful API response"""
        response_data = {
            "success": True,
            "status": APIResponseStatus.SUCCESS.value
        }
        
        if data is not None:
            response_data["data"] = data
            
        if message:
            response_data["message"] = message
            
        return jsonify(response_data), status_code
    
    @staticmethod
    def error(message: str, error_code: Optional[str] = None, details: Any = None, status_code: int = 400):
        """Create an error API response"""
        response_data = {
            "success": False,
            "status": APIResponseStatus.ERROR.value,
            "error": {
                "message": message
            }
        }
        
        if error_code:
            response_data["error"]["code"] = error_code
            
        if details:
            response_data["error"]["details"] = details
            
        return jsonify(response_data), status_code
    
    @staticmethod
    def validation_error(errors: List[Dict[str, Any]], status_code: int = 400):
        """Create a validation error response"""
        return APIResponse.error(
            message="Validation failed",
            error_code="VALIDATION_ERROR",
            details=errors,
            status_code=status_code
        )
    
    @staticmethod
    def not_found(resource: str = "Resource", status_code: int = 404):
        """Create a not found error response"""
        return APIResponse.error(
            message=f"{resource} not found",
            error_code="NOT_FOUND",
            status_code=status_code
        )
    
    @staticmethod
    def forbidden(message: str = "Access forbidden", status_code: int = 403):
        """Create a forbidden error response"""
        return APIResponse.error(
            message=message,
            error_code="FORBIDDEN",
            status_code=status_code
        )
    
    @staticmethod
    def internal_error(message: str = "Internal server error", status_code: int = 500):
        """Create an internal server error response"""
        return APIResponse.error(
            message=message,
            error_code="INTERNAL_ERROR",
            status_code=status_code
        )
    
    @staticmethod
    def created(data: Any = None, message: Optional[str] = None):
        """Create a resource created response"""
        return APIResponse.success(data=data, message=message, status_code=201)
    
    @staticmethod
    def no_content(message: Optional[str] = None):
        """Create a no content response"""
        response_data = {
            "success": True,
            "status": APIResponseStatus.SUCCESS.value
        }
        
        if message:
            response_data["message"] = message
            
        return jsonify(response_data), 204


def handle_service_exceptions(func):
    """Decorator to handle common service exceptions and convert them to API responses"""
    def wrapper(*args, **kwargs):
        try:
            return func(*args, **kwargs)
        except ValidationError as e:
            errors = []
            for error in e.errors():
                errors.append({
                    "field": ".".join(str(x) for x in error["loc"]),
                    "message": error["msg"],
                    "type": error["type"]
                })
            return APIResponse.validation_error(errors)
        except TaskterException as e:
            return APIResponse.error(
                message=str(e),
                error_code=e.error_code,
                details=e.details,
                status_code=e.status_code
            )
        except Exception as e:
            # Log the unexpected error
            import logging
            logger = logging.getLogger(__name__)
            logger.exception(f"Unexpected error in {func.__name__}: {str(e)}")
            
            return APIResponse.internal_error(
                message="An unexpected error occurred"
            )
    
    wrapper.__name__ = func.__name__
    wrapper.__doc__ = func.__doc__
    return wrapper


def validate_json_input(schema_class):
    """Decorator to validate JSON input against a Pydantic schema"""
    def decorator(func):
        def wrapper(*args, **kwargs):
            from flask import request
            
            if not request.is_json:
                return APIResponse.error(
                    message="Content-Type must be application/json",
                    error_code="INVALID_CONTENT_TYPE",
                    status_code=400
                )
            
            try:
                json_data = request.get_json()
                if json_data is None:
                    return APIResponse.error(
                        message="Invalid JSON data",
                        error_code="INVALID_JSON",
                        status_code=400
                    )
                
                # Validate the data against the schema
                validated_data = schema_class(**json_data)
                
                # Pass the validated data to the function
                return func(validated_data, *args, **kwargs)
                
            except ValidationError as e:
                errors = []
                for error in e.errors():
                    errors.append({
                        "field": ".".join(str(x) for x in error["loc"]),
                        "message": error["msg"],
                        "type": error["type"]
                    })
                return APIResponse.validation_error(errors)
            except Exception as e:
                return APIResponse.error(
                    message="Failed to process request data",
                    error_code="REQUEST_PROCESSING_ERROR",
                    status_code=400
                )
        
        wrapper.__name__ = func.__name__
        wrapper.__doc__ = func.__doc__
        return wrapper
    return decorator
