"""
Taskter Application Package

This package contains the main application components organized in a clean architecture:

- models/: Database models and entities
- services/: Business logic layer
- repositories/: Data access layer
- schemas/: Request/response validation schemas
- api/: API controllers and routes
- core/: Core configuration and utilities
"""

__version__ = "1.0.0"
__author__ = "Taskter Team"

# Import main models for easy access
from .models.task import Task
from .models.agent import Agent
from .models.execution import AgentExecution

__all__ = [
    "Task",
    "Agent", 
    "AgentExecution"
]
