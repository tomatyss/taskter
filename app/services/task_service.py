"""
Task service layer for business logic
"""
from typing import List, Optional, Dict, Any
from datetime import datetime

from app.repositories.task_repository import TaskRepository
from app.models.task import Task
from app.core.constants import TaskStatus, ExecutionStatus, API_MESSAGES, ERROR_MESSAGES
from app.core.exceptions import (
    TaskNotFoundError, TaskValidationError, TaskStatusError,
    ValidationError, ConflictError
)
from app.core.logging import get_logger, log_database_operation

logger = get_logger(__name__)


class TaskService:
    """Service for task business logic"""
    
    def __init__(self, task_repository: Optional[TaskRepository] = None):
        self.task_repo = task_repository or TaskRepository()
    
    def create_task(self, title: str, description: Optional[str] = None,
                   status: Optional[TaskStatus] = None) -> Task:
        """Create a new task"""
        try:
            # Validate input
            if not title or not title.strip():
                raise TaskValidationError("Task title is required")
            
            if len(title) > 200:
                raise TaskValidationError("Task title must be 200 characters or less")
            
            # Create task
            task_data = {
                'title': title.strip(),
                'description': description.strip() if description else None,
                'status': status.value if status else TaskStatus.TODO.value
            }
            
            task = Task.from_dict(task_data)
            created_task = self.task_repo.create(task)
            
            logger.info(f"Created task {created_task.id}: {created_task.title}")
            return created_task
            
        except Exception as e:
            logger.error(f"Failed to create task: {str(e)}")
            raise
    
    def get_task_by_id(self, task_id: int) -> Task:
        """Get task by ID"""
        task = self.task_repo.get_by_id(task_id)
        if not task:
            raise TaskNotFoundError(task_id)
        return task
    
    def update_task(self, task_id: int, title: Optional[str] = None,
                   description: Optional[str] = None,
                   status: Optional[TaskStatus] = None) -> Task:
        """Update an existing task"""
        try:
            task = self.get_task_by_id(task_id)
            
            # Validate input
            if title is not None:
                if not title.strip():
                    raise TaskValidationError("Task title cannot be empty")
                if len(title) > 200:
                    raise TaskValidationError("Task title must be 200 characters or less")
            
            # Check if task can be updated
            if task.is_running():
                raise ConflictError("Cannot update task while it's running")
            
            # Update task
            update_data = {}
            if title is not None:
                update_data['title'] = title.strip()
            if description is not None:
                update_data['description'] = description.strip() if description else None
            if status is not None:
                update_data['status'] = status.value
            
            if update_data:
                task.update_from_dict(update_data)
                updated_task = self.task_repo.update(task)
                
                logger.info(f"Updated task {task_id}")
                return updated_task
            
            return task
            
        except Exception as e:
            logger.error(f"Failed to update task {task_id}: {str(e)}")
            raise
    
    def delete_task(self, task_id: int) -> bool:
        """Delete a task"""
        try:
            task = self.get_task_by_id(task_id)
            
            # Check if task can be deleted
            if not task.can_be_deleted():
                raise ConflictError("Cannot delete task while it's running")
            
            success = self.task_repo.delete(task)
            
            if success:
                logger.info(f"Deleted task {task_id}")
            
            return success
            
        except Exception as e:
            logger.error(f"Failed to delete task {task_id}: {str(e)}")
            raise
    
    def move_task_to_status(self, task_id: int, new_status: TaskStatus) -> Task:
        """Move task to a new status"""
        try:
            task = self.get_task_by_id(task_id)
            
            # Validate status transition
            if not self._is_valid_status_transition(task.status, new_status.value):
                raise TaskStatusError(
                    f"Invalid status transition from {task.status} to {new_status.value}"
                )
            
            task.move_to_status(new_status.value)
            updated_task = self.task_repo.update(task)
            
            logger.info(f"Moved task {task_id} to status {new_status.value}")
            return updated_task
            
        except Exception as e:
            logger.error(f"Failed to move task {task_id} to status {new_status.value}: {str(e)}")
            raise
    
    def assign_task_to_agent(self, task_id: int, agent_id: int) -> Task:
        """Assign a task to an agent"""
        try:
            task = self.get_task_by_id(task_id)
            
            # Check if task can be assigned
            if not task.can_be_assigned():
                raise ConflictError(ERROR_MESSAGES["TASK_ALREADY_RUNNING"])
            
            # Validate agent exists (this would typically check agent service)
            # For now, we'll assume the agent_id is valid
            
            task.assign_to_agent(agent_id)
            updated_task = self.task_repo.update(task)
            
            logger.info(f"Assigned task {task_id} to agent {agent_id}")
            return updated_task
            
        except Exception as e:
            logger.error(f"Failed to assign task {task_id} to agent {agent_id}: {str(e)}")
            raise
    
    def unassign_task_from_agent(self, task_id: int) -> Task:
        """Unassign a task from its agent"""
        try:
            task = self.get_task_by_id(task_id)
            
            # Check if task can be unassigned
            if not task.can_be_unassigned():
                raise ConflictError(ERROR_MESSAGES["CANNOT_UNASSIGN_RUNNING_TASK"])
            
            task.unassign_from_agent()
            updated_task = self.task_repo.update(task)
            
            logger.info(f"Unassigned task {task_id} from agent")
            return updated_task
            
        except Exception as e:
            logger.error(f"Failed to unassign task {task_id}: {str(e)}")
            raise
    
    def start_task_execution(self, task_id: int) -> Task:
        """Start task execution"""
        try:
            task = self.get_task_by_id(task_id)
            
            if not task.is_assigned():
                raise ConflictError("Task must be assigned to an agent before execution")
            
            if task.is_running():
                raise ConflictError("Task is already running")
            
            task.start_execution()
            updated_task = self.task_repo.update(task)
            
            logger.info(f"Started execution for task {task_id}")
            return updated_task
            
        except Exception as e:
            logger.error(f"Failed to start execution for task {task_id}: {str(e)}")
            raise
    
    def complete_task_execution(self, task_id: int, success: bool = True) -> Task:
        """Complete task execution"""
        try:
            task = self.get_task_by_id(task_id)
            
            if not task.is_running():
                raise ConflictError("Task is not currently running")
            
            task.complete_execution(success)
            updated_task = self.task_repo.update(task)
            
            status = "successfully" if success else "with failure"
            logger.info(f"Completed execution for task {task_id} {status}")
            return updated_task
            
        except Exception as e:
            logger.error(f"Failed to complete execution for task {task_id}: {str(e)}")
            raise
    
    def fail_task_execution(self, task_id: int) -> Task:
        """Mark task execution as failed"""
        try:
            task = self.get_task_by_id(task_id)
            
            task.fail_execution()
            updated_task = self.task_repo.update(task)
            
            logger.info(f"Marked execution as failed for task {task_id}")
            return updated_task
            
        except Exception as e:
            logger.error(f"Failed to mark task {task_id} execution as failed: {str(e)}")
            raise
    
    def reset_task_execution(self, task_id: int) -> Task:
        """Reset task execution status"""
        try:
            task = self.get_task_by_id(task_id)
            
            task.reset_execution()
            updated_task = self.task_repo.update(task)
            
            logger.info(f"Reset execution status for task {task_id}")
            return updated_task
            
        except Exception as e:
            logger.error(f"Failed to reset execution for task {task_id}: {str(e)}")
            raise
    
    def get_tasks_by_status(self, status: TaskStatus, 
                           limit: Optional[int] = None,
                           offset: Optional[int] = None) -> List[Task]:
        """Get tasks by status"""
        return self.task_repo.get_by_status(status, limit, offset)
    
    def get_tasks_by_execution_status(self, execution_status: ExecutionStatus,
                                     limit: Optional[int] = None,
                                     offset: Optional[int] = None) -> List[Task]:
        """Get tasks by execution status"""
        return self.task_repo.get_by_execution_status(execution_status, limit, offset)
    
    def get_agent_tasks(self, agent_id: int,
                       limit: Optional[int] = None,
                       offset: Optional[int] = None) -> List[Task]:
        """Get tasks assigned to an agent"""
        return self.task_repo.get_assigned_to_agent(agent_id, limit, offset)
    
    def get_running_tasks(self) -> List[Task]:
        """Get all currently running tasks"""
        return self.task_repo.get_running_tasks()
    
    def get_pending_tasks(self) -> List[Task]:
        """Get tasks that are assigned but not running"""
        return self.task_repo.get_pending_tasks()
    
    def search_tasks(self, search_term: str,
                    limit: Optional[int] = None,
                    offset: Optional[int] = None) -> List[Task]:
        """Search tasks by title and description"""
        # Search in both title and description
        title_results = self.task_repo.search_by_title(search_term, limit, offset)
        desc_results = self.task_repo.search_by_description(search_term, limit, offset)
        
        # Combine and deduplicate results
        seen_ids = set()
        combined_results = []
        
        for task in title_results + desc_results:
            if task.id not in seen_ids:
                combined_results.append(task)
                seen_ids.add(task.id)
        
        # Sort by creation date (newest first)
        combined_results.sort(key=lambda x: x.created_at, reverse=True)
        
        return combined_results[:limit] if limit else combined_results
    
    def get_task_statistics(self) -> Dict[str, Any]:
        """Get comprehensive task statistics"""
        return self.task_repo.get_task_statistics()
    
    def get_agent_task_statistics(self, agent_id: int) -> Dict[str, Any]:
        """Get task statistics for a specific agent"""
        return self.task_repo.get_agent_task_statistics(agent_id)
    
    def get_overdue_tasks(self, days: int = 7) -> List[Task]:
        """Get tasks that haven't been updated in specified days"""
        return self.task_repo.get_overdue_tasks(days)
    
    def bulk_update_task_status(self, task_ids: List[int], 
                               new_status: TaskStatus) -> int:
        """Bulk update task status"""
        try:
            # Validate that all tasks exist and can be updated
            tasks = []
            for task_id in task_ids:
                task = self.get_task_by_id(task_id)
                if task.is_running():
                    raise ConflictError(f"Cannot update running task {task_id}")
                tasks.append(task)
            
            # Perform bulk update
            updated_count = self.task_repo.bulk_update_status(task_ids, new_status)
            
            logger.info(f"Bulk updated {updated_count} tasks to status {new_status.value}")
            return updated_count
            
        except Exception as e:
            logger.error(f"Failed to bulk update task status: {str(e)}")
            raise
    
    def bulk_assign_tasks(self, task_ids: List[int], agent_id: int) -> int:
        """Bulk assign tasks to an agent"""
        try:
            # Validate that all tasks can be assigned
            for task_id in task_ids:
                task = self.get_task_by_id(task_id)
                if not task.can_be_assigned():
                    raise ConflictError(f"Cannot assign running task {task_id}")
            
            # Perform bulk assignment
            updated_count = self.task_repo.bulk_assign_to_agent(task_ids, agent_id)
            
            logger.info(f"Bulk assigned {updated_count} tasks to agent {agent_id}")
            return updated_count
            
        except Exception as e:
            logger.error(f"Failed to bulk assign tasks: {str(e)}")
            raise
    
    def unassign_agent_tasks(self, agent_id: int) -> int:
        """Unassign all tasks from an agent"""
        try:
            unassigned_count = self.task_repo.unassign_tasks_from_agent(agent_id)
            
            logger.info(f"Unassigned {unassigned_count} tasks from agent {agent_id}")
            return unassigned_count
            
        except Exception as e:
            logger.error(f"Failed to unassign tasks from agent {agent_id}: {str(e)}")
            raise
    
    def copy_task(self, task_id: int) -> Task:
        """Create a copy of an existing task"""
        try:
            original_task = self.get_task_by_id(task_id)
            
            # Create copy with modified title and reset properties
            copy_title = f"Copy of {original_task.title}"
            if len(copy_title) > 200:
                # Truncate if too long
                copy_title = copy_title[:197] + "..."
            
            copy_data = {
                'title': copy_title,
                'description': original_task.description,
                'status': TaskStatus.TODO.value,  # Always start as TODO
                'execution_status': ExecutionStatus.MANUAL.value,  # Reset execution status
                'assigned_agent_id': None  # Don't copy agent assignment
            }
            
            copied_task = Task.from_dict(copy_data)
            created_copy = self.task_repo.create(copied_task)
            
            logger.info(f"Created copy of task {task_id} as task {created_copy.id}")
            return created_copy
            
        except Exception as e:
            logger.error(f"Failed to copy task {task_id}: {str(e)}")
            raise
    
    def _is_valid_status_transition(self, current_status: str, new_status: str) -> bool:
        """Validate if status transition is allowed"""
        # Define valid transitions
        valid_transitions = {
            TaskStatus.TODO.value: [
                TaskStatus.IN_PROGRESS.value,
                TaskStatus.BLOCKED.value,
                TaskStatus.DONE.value
            ],
            TaskStatus.IN_PROGRESS.value: [
                TaskStatus.TODO.value,
                TaskStatus.DONE.value,
                TaskStatus.BLOCKED.value
            ],
            TaskStatus.BLOCKED.value: [
                TaskStatus.TODO.value,
                TaskStatus.IN_PROGRESS.value
            ],
            TaskStatus.DONE.value: [
                TaskStatus.TODO.value,
                TaskStatus.IN_PROGRESS.value
            ]
        }
        
        return new_status in valid_transitions.get(current_status, [])
