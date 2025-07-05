"""
Task repository implementation
"""
from typing import List, Optional, Dict, Any
from sqlalchemy import desc, asc

from app.repositories.base import BaseRepository, PaginatedResult, paginate_query
from app.models.task import Task
from app.core.constants import TaskStatus, ExecutionStatus
from app.core.logging import get_logger

logger = get_logger(__name__)


class TaskRepository(BaseRepository[Task]):
    """Repository for Task entities"""
    
    def __init__(self):
        super().__init__(Task)
    
    def get_model_class(self) -> type:
        return Task
    
    def get_by_status(self, status: TaskStatus, 
                     limit: Optional[int] = None, 
                     offset: Optional[int] = None) -> List[Task]:
        """Get tasks by status"""
        return self.find_by(
            filters={'status': status.value},
            limit=limit,
            offset=offset,
            order_by='created_at',
            order_desc=True
        )
    
    def get_by_execution_status(self, execution_status: ExecutionStatus,
                               limit: Optional[int] = None,
                               offset: Optional[int] = None) -> List[Task]:
        """Get tasks by execution status"""
        return self.find_by(
            filters={'execution_status': execution_status.value},
            limit=limit,
            offset=offset,
            order_by='created_at',
            order_desc=True
        )
    
    def get_assigned_to_agent(self, agent_id: int,
                             limit: Optional[int] = None,
                             offset: Optional[int] = None) -> List[Task]:
        """Get tasks assigned to a specific agent"""
        return self.find_by(
            filters={'assigned_agent_id': agent_id},
            limit=limit,
            offset=offset,
            order_by='created_at',
            order_desc=True
        )
    
    def get_running_tasks(self) -> List[Task]:
        """Get all currently running tasks"""
        return self.find_by(
            filters={'execution_status': ExecutionStatus.RUNNING.value},
            order_by='created_at',
            order_desc=True
        )
    
    def get_pending_tasks(self) -> List[Task]:
        """Get tasks that are assigned but not running"""
        return self.find_by(
            filters={'execution_status': ExecutionStatus.ASSIGNED.value},
            order_by='created_at',
            order_desc=True
        )
    
    def get_completed_tasks(self, limit: Optional[int] = None) -> List[Task]:
        """Get completed tasks"""
        return self.find_by(
            filters={'status': TaskStatus.DONE.value},
            limit=limit,
            order_by='updated_at',
            order_desc=True
        )
    
    def get_failed_tasks(self, limit: Optional[int] = None) -> List[Task]:
        """Get tasks with failed executions"""
        return self.find_by(
            filters={'execution_status': ExecutionStatus.FAILED.value},
            limit=limit,
            order_by='updated_at',
            order_desc=True
        )
    
    def search_by_title(self, search_term: str,
                       limit: Optional[int] = None,
                       offset: Optional[int] = None) -> List[Task]:
        """Search tasks by title"""
        return self.find_by(
            filters={'title': {'ilike': search_term}},
            limit=limit,
            offset=offset,
            order_by='created_at',
            order_desc=True
        )
    
    def search_by_description(self, search_term: str,
                             limit: Optional[int] = None,
                             offset: Optional[int] = None) -> List[Task]:
        """Search tasks by description"""
        return self.find_by(
            filters={'description': {'ilike': search_term}},
            limit=limit,
            offset=offset,
            order_by='created_at',
            order_desc=True
        )
    
    def get_tasks_created_after(self, date, 
                               limit: Optional[int] = None) -> List[Task]:
        """Get tasks created after a specific date"""
        return self.find_by(
            filters={'created_at': {'gte': date}},
            limit=limit,
            order_by='created_at',
            order_desc=True
        )
    
    def get_tasks_updated_after(self, date,
                               limit: Optional[int] = None) -> List[Task]:
        """Get tasks updated after a specific date"""
        return self.find_by(
            filters={'updated_at': {'gte': date}},
            limit=limit,
            order_by='updated_at',
            order_desc=True
        )
    
    def get_paginated_tasks(self, page: int = 1, per_page: int = 20,
                           status: Optional[TaskStatus] = None,
                           execution_status: Optional[ExecutionStatus] = None,
                           agent_id: Optional[int] = None,
                           search: Optional[str] = None) -> PaginatedResult:
        """Get paginated tasks with optional filters"""
        query = self.session.query(Task)
        
        # Apply filters
        if status:
            query = query.filter(Task.status == status.value)
        
        if execution_status:
            query = query.filter(Task.execution_status == execution_status.value)
        
        if agent_id:
            query = query.filter(Task.assigned_agent_id == agent_id)
        
        if search:
            search_filter = f"%{search}%"
            query = query.filter(
                Task.title.ilike(search_filter) | 
                Task.description.ilike(search_filter)
            )
        
        # Order by creation date (newest first)
        query = query.order_by(desc(Task.created_at))
        
        return paginate_query(query, page, per_page)
    
    def get_task_statistics(self) -> Dict[str, Any]:
        """Get task statistics"""
        total_tasks = self.count()
        
        stats = {
            'total_tasks': total_tasks,
            'by_status': {},
            'by_execution_status': {},
            'assigned_tasks': self.count({'assigned_agent_id': {'gt': 0}}),
            'unassigned_tasks': self.count({'assigned_agent_id': None})
        }
        
        # Count by status
        for status in TaskStatus:
            stats['by_status'][status.value] = self.count({'status': status.value})
        
        # Count by execution status
        for exec_status in ExecutionStatus:
            stats['by_execution_status'][exec_status.value] = self.count({
                'execution_status': exec_status.value
            })
        
        return stats
    
    def get_agent_task_statistics(self, agent_id: int) -> Dict[str, Any]:
        """Get task statistics for a specific agent"""
        base_filter = {'assigned_agent_id': agent_id}
        
        stats = {
            'total_assigned': self.count(base_filter),
            'by_status': {},
            'by_execution_status': {}
        }
        
        # Count by status
        for status in TaskStatus:
            filter_dict = {**base_filter, 'status': status.value}
            stats['by_status'][status.value] = self.count(filter_dict)
        
        # Count by execution status
        for exec_status in ExecutionStatus:
            filter_dict = {**base_filter, 'execution_status': exec_status.value}
            stats['by_execution_status'][exec_status.value] = self.count(filter_dict)
        
        return stats
    
    def unassign_tasks_from_agent(self, agent_id: int) -> int:
        """Unassign all tasks from an agent (except running ones)"""
        tasks = self.find_by({
            'assigned_agent_id': agent_id,
            'execution_status': {'not_in': [ExecutionStatus.RUNNING.value]}
        })
        
        count = 0
        for task in tasks:
            task.unassign_from_agent()
            count += 1
        
        if count > 0:
            self.session.commit()
        
        return count
    
    def get_overdue_tasks(self, days: int = 7) -> List[Task]:
        """Get tasks that haven't been updated in specified days"""
        from datetime import datetime, timedelta
        cutoff_date = datetime.utcnow() - timedelta(days=days)
        
        return self.find_by(
            filters={
                'updated_at': {'lt': cutoff_date},
                'status': {'not_in': [TaskStatus.DONE.value]}
            },
            order_by='updated_at',
            order_desc=False
        )
    
    def get_tasks_by_date_range(self, start_date, end_date,
                               date_field: str = 'created_at') -> List[Task]:
        """Get tasks within a date range"""
        if date_field not in ['created_at', 'updated_at']:
            raise ValueError("date_field must be 'created_at' or 'updated_at'")
        
        return self.find_by(
            filters={
                date_field: {
                    'gte': start_date,
                    'lte': end_date
                }
            },
            order_by=date_field,
            order_desc=True
        )
    
    def bulk_update_status(self, task_ids: List[int], new_status: TaskStatus) -> int:
        """Bulk update task status"""
        updates = [
            {'id': task_id, 'status': new_status.value}
            for task_id in task_ids
        ]
        return self.bulk_update(updates)
    
    def bulk_assign_to_agent(self, task_ids: List[int], agent_id: int) -> int:
        """Bulk assign tasks to an agent"""
        # First check that none of the tasks are running
        running_tasks = self.find_by({
            'id': task_ids,
            'execution_status': ExecutionStatus.RUNNING.value
        })
        
        if running_tasks:
            running_ids = [task.id for task in running_tasks]
            raise ValueError(f"Cannot assign running tasks: {running_ids}")
        
        updates = [
            {
                'id': task_id,
                'assigned_agent_id': agent_id,
                'execution_status': ExecutionStatus.ASSIGNED.value
            }
            for task_id in task_ids
        ]
        return self.bulk_update(updates)
    
    def get_recent_tasks(self, limit: int = 10) -> List[Task]:
        """Get recently created tasks"""
        return self.find_by(
            filters={},
            limit=limit,
            order_by='created_at',
            order_desc=True
        )
    
    def get_recently_updated_tasks(self, limit: int = 10) -> List[Task]:
        """Get recently updated tasks"""
        return self.find_by(
            filters={},
            limit=limit,
            order_by='updated_at',
            order_desc=True
        )
