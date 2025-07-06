"""
API v1 blueprint registration and initialization.
"""

from flask import Blueprint, redirect, request
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
    
    # Register legacy API blueprint for backward compatibility
    app.register_blueprint(legacy_api)


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


# Create legacy API blueprint for compatibility
legacy_api = Blueprint('legacy_api', __name__, url_prefix='/api')

# Create compatibility routes for legacy API paths

# Redirect /api/agents to /api/v1/agents
@legacy_api.route('/agents', defaults={'path': ''}, methods=['GET', 'POST', 'PUT', 'DELETE', 'PATCH'])
@legacy_api.route('/agents/<path:path>', methods=['GET', 'POST', 'PUT', 'DELETE', 'PATCH'])
def redirect_agents(path):
    """Redirect legacy /api/agents paths to /api/v1/agents"""
    if path:
        return redirect(f'/api/v1/agents/{path}', code=307)  # 307 preserves method and body
    else:
        # Preserve query parameters
        if request.query_string:
            return redirect(f'/api/v1/agents?{request.query_string.decode()}', code=307)
        return redirect('/api/v1/agents', code=307)

# Redirect /api/tasks to /api/v1/tasks  
@legacy_api.route('/tasks', defaults={'path': ''}, methods=['GET', 'POST', 'PUT', 'DELETE', 'PATCH'])
@legacy_api.route('/tasks/<path:path>', methods=['GET', 'POST', 'PUT', 'DELETE', 'PATCH'])
def redirect_tasks(path):
    """Redirect legacy /api/tasks paths to /api/v1/tasks"""
    if path:
        return redirect(f'/api/v1/tasks/{path}', code=307)
    else:
        if request.query_string:
            return redirect(f'/api/v1/tasks?{request.query_string.decode()}', code=307)
        return redirect('/api/v1/tasks', code=307)

# Redirect /api/executions to /api/v1/executions
@legacy_api.route('/executions', defaults={'path': ''}, methods=['GET', 'POST', 'PUT', 'DELETE', 'PATCH'])
@legacy_api.route('/executions/<path:path>', methods=['GET', 'POST', 'PUT', 'DELETE', 'PATCH'])
def redirect_executions(path):
    """Redirect legacy /api/executions paths to /api/v1/executions"""
    if path:
        return redirect(f'/api/v1/executions/{path}', code=307)
    else:
        if request.query_string:
            return redirect(f'/api/v1/executions?{request.query_string.decode()}', code=307)
        return redirect('/api/v1/executions', code=307)
