"""
API v1 blueprint registration and initialization.
"""

from flask import Blueprint
from app.api.v1.tasks import tasks_bp
from app.api.v1.agents import agents_bp
from app.api.v1.executions import executions_bp

# Create main API v1 blueprint
api_v1 = Blueprint('api_v1', __name__, url_prefix='/api/v1')

def register_blueprints(app):
    """Register all API v1 blueprints with the Flask app"""
    
    # Register individual controller blueprints
    app.register_blueprint(tasks_bp)
    app.register_blueprint(agents_bp)
    app.register_blueprint(executions_bp)
    
    # Register main API blueprint (for any shared routes)
    app.register_blueprint(api_v1)


# Health check endpoint for the API
@api_v1.route('/health')
def health_check():
    """API health check endpoint"""
    from app.api.response import APIResponse
    return APIResponse.success(
        data={
            "status": "healthy",
            "version": "1.0.0",
            "endpoints": {
                "tasks": "/api/v1/tasks",
                "agents": "/api/v1/agents", 
                "executions": "/api/v1/executions"
            }
        },
        message="API is running"
    )


# API info endpoint
@api_v1.route('/info')
def api_info():
    """API information endpoint"""
    from app.api.response import APIResponse
    return APIResponse.success(
        data={
            "name": "Taskter API",
            "version": "1.0.0",
            "description": "Task management and agent execution API",
            "endpoints": {
                "tasks": {
                    "base_url": "/api/v1/tasks",
                    "methods": ["GET", "POST", "PUT", "DELETE"],
                    "description": "Task management endpoints"
                },
                "agents": {
                    "base_url": "/api/v1/agents",
                    "methods": ["GET", "POST", "PUT", "DELETE"],
                    "description": "Agent management endpoints"
                },
                "executions": {
                    "base_url": "/api/v1/executions",
                    "methods": ["GET", "POST"],
                    "description": "Execution monitoring endpoints"
                }
            }
        }
    )
