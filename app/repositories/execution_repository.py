"""
Execution repository implementation
"""
from typing import List, Optional, Dict, Any
from datetime import datetime, timedelta
from sqlalchemy import desc, asc, func

from app.repositories.base import BaseRepository, PaginatedResult, paginate_query
from app.models.execution import AgentExecution
from app.core.constants import AgentExecutionStatus
from app.core.logging import get_logger

logger = get_logger(__name__)


class ExecutionRepository(BaseRepository[AgentExecution]):
    """Repository for AgentExecution entities"""
    
    def __init__(self):
        super().__init__(AgentExecution)
    
    def get_model_class(self) -> type:
        return AgentExecution
    
    def get_by_task_id(self, task_id: int,
                      limit: Optional[int] = None,
                      offset: Optional[int] = None) -> List[AgentExecution]:
        """Get executions for a specific task"""
        return self.find_by(
            filters={'task_id': task_id},
            limit=limit,
            offset=offset,
            order_by='created_at',
            order_desc=True
        )
    
    def get_by_agent_id(self, agent_id: int,
                       limit: Optional[int] = None,
                       offset: Optional[int] = None) -> List[AgentExecution]:
        """Get executions for a specific agent"""
        return self.find_by(
            filters={'agent_id': agent_id},
            limit=limit,
            offset=offset,
            order_by='created_at',
            order_desc=True
        )
    
    def get_by_status(self, status: AgentExecutionStatus,
                     limit: Optional[int] = None,
                     offset: Optional[int] = None) -> List[AgentExecution]:
        """Get executions by status"""
        return self.find_by(
            filters={'status': status.value},
            limit=limit,
            offset=offset,
            order_by='created_at',
            order_desc=True
        )
    
    def get_running_executions(self) -> List[AgentExecution]:
        """Get all currently running executions"""
        return self.find_by(
            filters={'status': AgentExecutionStatus.RUNNING.value},
            order_by='started_at',
            order_desc=False  # Oldest first
        )
    
    def get_pending_executions(self) -> List[AgentExecution]:
        """Get all pending executions"""
        return self.find_by(
            filters={'status': AgentExecutionStatus.PENDING.value},
            order_by='created_at',
            order_desc=False  # Oldest first (FIFO)
        )
    
    def get_completed_executions(self, limit: Optional[int] = None) -> List[AgentExecution]:
        """Get completed executions"""
        return self.find_by(
            filters={'status': AgentExecutionStatus.COMPLETED.value},
            limit=limit,
            order_by='completed_at',
            order_desc=True
        )
    
    def get_failed_executions(self, limit: Optional[int] = None) -> List[AgentExecution]:
        """Get failed executions"""
        return self.find_by(
            filters={'status': AgentExecutionStatus.FAILED.value},
            limit=limit,
            order_by='completed_at',
            order_desc=True
        )
    
    def get_cancelled_executions(self, limit: Optional[int] = None) -> List[AgentExecution]:
        """Get cancelled executions"""
        return self.find_by(
            filters={'status': AgentExecutionStatus.CANCELLED.value},
            limit=limit,
            order_by='completed_at',
            order_desc=True
        )
    
    def get_executions_by_date_range(self, start_date: datetime, end_date: datetime,
                                    date_field: str = 'created_at') -> List[AgentExecution]:
        """Get executions within a date range"""
        if date_field not in ['created_at', 'started_at', 'completed_at']:
            raise ValueError("date_field must be 'created_at', 'started_at', or 'completed_at'")
        
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
    
    def get_long_running_executions(self, threshold_minutes: int = 30) -> List[AgentExecution]:
        """Get executions that have been running longer than threshold"""
        threshold_time = datetime.utcnow() - timedelta(minutes=threshold_minutes)
        
        return self.find_by(
            filters={
                'status': AgentExecutionStatus.RUNNING.value,
                'started_at': {'lt': threshold_time}
            },
            order_by='started_at',
            order_desc=False
        )
    
    def get_executions_with_high_token_usage(self, min_tokens: int = 10000) -> List[AgentExecution]:
        """Get executions with high token usage"""
        return self.find_by(
            filters={'tokens_used': {'gte': min_tokens}},
            order_by='tokens_used',
            order_desc=True
        )
    
    def get_expensive_executions(self, min_cost: float = 1.0) -> List[AgentExecution]:
        """Get executions with high cost"""
        return self.find_by(
            filters={'llm_cost': {'gte': min_cost}},
            order_by='llm_cost',
            order_desc=True
        )
    
    def get_latest_execution_for_task(self, task_id: int) -> Optional[AgentExecution]:
        """Get the latest execution for a specific task"""
        executions = self.get_by_task_id(task_id, limit=1)
        return executions[0] if executions else None
    
    def get_latest_execution_for_agent(self, agent_id: int) -> Optional[AgentExecution]:
        """Get the latest execution for a specific agent"""
        executions = self.get_by_agent_id(agent_id, limit=1)
        return executions[0] if executions else None
    
    def get_paginated_executions(self, page: int = 1, per_page: int = 20,
                                status: Optional[AgentExecutionStatus] = None,
                                task_id: Optional[int] = None,
                                agent_id: Optional[int] = None) -> PaginatedResult:
        """Get paginated executions with optional filters"""
        query = self.session.query(AgentExecution)
        
        # Apply filters
        if status:
            query = query.filter(AgentExecution.status == status.value)
        
        if task_id:
            query = query.filter(AgentExecution.task_id == task_id)
        
        if agent_id:
            query = query.filter(AgentExecution.agent_id == agent_id)
        
        # Order by creation date (newest first)
        query = query.order_by(desc(AgentExecution.created_at))
        
        return paginate_query(query, page, per_page)
    
    def get_execution_statistics(self) -> Dict[str, Any]:
        """Get comprehensive execution statistics"""
        total_executions = self.count()
        
        stats = {
            'total_executions': total_executions,
            'by_status': {},
            'running_executions': self.count({'status': AgentExecutionStatus.RUNNING.value}),
            'pending_executions': self.count({'status': AgentExecutionStatus.PENDING.value}),
            'total_tokens_used': 0,
            'total_cost': 0.0,
            'average_execution_time': 0.0,
            'success_rate': 0.0
        }
        
        # Count by status
        for status in AgentExecutionStatus:
            stats['by_status'][status.value] = self.count({'status': status.value})
        
        # Calculate success rate
        completed = stats['by_status'].get(AgentExecutionStatus.COMPLETED.value, 0)
        failed = stats['by_status'].get(AgentExecutionStatus.FAILED.value, 0)
        total_finished = completed + failed
        
        if total_finished > 0:
            stats['success_rate'] = (completed / total_finished) * 100
        
        # Get aggregate statistics (this would be more efficient with SQL aggregation)
        executions = self.get_all(limit=10000)  # Get sample for stats
        
        if executions:
            total_tokens = sum(e.tokens_used or 0 for e in executions)
            total_cost = sum(e.llm_cost or 0.0 for e in executions)
            
            # Calculate average execution time for completed executions
            completed_executions = [e for e in executions if e.execution_time_seconds]
            if completed_executions:
                avg_time = sum(e.execution_time_seconds for e in completed_executions) / len(completed_executions)
                stats['average_execution_time'] = avg_time
            
            stats['total_tokens_used'] = total_tokens
            stats['total_cost'] = total_cost
        
        return stats
    
    def get_agent_execution_statistics(self, agent_id: int) -> Dict[str, Any]:
        """Get execution statistics for a specific agent"""
        base_filter = {'agent_id': agent_id}
        
        stats = {
            'total_executions': self.count(base_filter),
            'by_status': {},
            'total_tokens_used': 0,
            'total_cost': 0.0,
            'average_execution_time': 0.0,
            'success_rate': 0.0
        }
        
        # Count by status
        for status in AgentExecutionStatus:
            filter_dict = {**base_filter, 'status': status.value}
            stats['by_status'][status.value] = self.count(filter_dict)
        
        # Calculate success rate
        completed = stats['by_status'].get(AgentExecutionStatus.COMPLETED.value, 0)
        failed = stats['by_status'].get(AgentExecutionStatus.FAILED.value, 0)
        total_finished = completed + failed
        
        if total_finished > 0:
            stats['success_rate'] = (completed / total_finished) * 100
        
        # Get executions for this agent
        executions = self.get_by_agent_id(agent_id)
        
        if executions:
            total_tokens = sum(e.tokens_used or 0 for e in executions)
            total_cost = sum(e.llm_cost or 0.0 for e in executions)
            
            # Calculate average execution time
            completed_executions = [e for e in executions if e.execution_time_seconds]
            if completed_executions:
                avg_time = sum(e.execution_time_seconds for e in completed_executions) / len(completed_executions)
                stats['average_execution_time'] = avg_time
            
            stats['total_tokens_used'] = total_tokens
            stats['total_cost'] = total_cost
        
        return stats
    
    def get_task_execution_statistics(self, task_id: int) -> Dict[str, Any]:
        """Get execution statistics for a specific task"""
        base_filter = {'task_id': task_id}
        
        stats = {
            'total_executions': self.count(base_filter),
            'by_status': {},
            'total_tokens_used': 0,
            'total_cost': 0.0,
            'total_execution_time': 0.0,
            'success_rate': 0.0
        }
        
        # Count by status
        for status in AgentExecutionStatus:
            filter_dict = {**base_filter, 'status': status.value}
            stats['by_status'][status.value] = self.count(filter_dict)
        
        # Calculate success rate
        completed = stats['by_status'].get(AgentExecutionStatus.COMPLETED.value, 0)
        failed = stats['by_status'].get(AgentExecutionStatus.FAILED.value, 0)
        total_finished = completed + failed
        
        if total_finished > 0:
            stats['success_rate'] = (completed / total_finished) * 100
        
        # Get executions for this task
        executions = self.get_by_task_id(task_id)
        
        if executions:
            total_tokens = sum(e.tokens_used or 0 for e in executions)
            total_cost = sum(e.llm_cost or 0.0 for e in executions)
            total_time = sum(e.execution_time_seconds or 0.0 for e in executions)
            
            stats['total_tokens_used'] = total_tokens
            stats['total_cost'] = total_cost
            stats['total_execution_time'] = total_time
        
        return stats
    
    def get_daily_execution_counts(self, days: int = 30) -> Dict[str, int]:
        """Get daily execution counts for the last N days"""
        end_date = datetime.utcnow()
        start_date = end_date - timedelta(days=days)
        
        executions = self.get_executions_by_date_range(start_date, end_date)
        
        # Group by date
        daily_counts = {}
        for execution in executions:
            date_key = execution.created_at.strftime('%Y-%m-%d')
            daily_counts[date_key] = daily_counts.get(date_key, 0) + 1
        
        return daily_counts
    
    def cleanup_old_executions(self, days_to_keep: int = 90) -> int:
        """Clean up old completed/failed executions"""
        cutoff_date = datetime.utcnow() - timedelta(days=days_to_keep)
        
        old_executions = self.find_by(
            filters={
                'status': [
                    AgentExecutionStatus.COMPLETED.value,
                    AgentExecutionStatus.FAILED.value,
                    AgentExecutionStatus.CANCELLED.value
                ],
                'completed_at': {'lt': cutoff_date}
            }
        )
        
        count = 0
        for execution in old_executions:
            self.delete(execution)
            count += 1
        
        return count
    
    def get_performance_metrics(self, time_period_hours: int = 24) -> Dict[str, Any]:
        """Get performance metrics for a time period"""
        start_time = datetime.utcnow() - timedelta(hours=time_period_hours)
        
        executions = self.get_executions_by_date_range(start_time, datetime.utcnow())
        
        if not executions:
            return {
                'total_executions': 0,
                'executions_per_hour': 0.0,
                'average_tokens_per_execution': 0.0,
                'average_cost_per_execution': 0.0,
                'average_execution_time': 0.0,
                'success_rate': 0.0
            }
        
        completed = [e for e in executions if e.status == AgentExecutionStatus.COMPLETED.value]
        failed = [e for e in executions if e.status == AgentExecutionStatus.FAILED.value]
        
        total_tokens = sum(e.tokens_used or 0 for e in executions)
        total_cost = sum(e.llm_cost or 0.0 for e in executions)
        
        # Calculate averages
        avg_tokens = total_tokens / len(executions) if executions else 0
        avg_cost = total_cost / len(executions) if executions else 0
        
        # Calculate average execution time for completed executions
        completed_with_time = [e for e in completed if e.execution_time_seconds]
        avg_time = (sum(e.execution_time_seconds for e in completed_with_time) / 
                   len(completed_with_time)) if completed_with_time else 0
        
        # Calculate success rate
        total_finished = len(completed) + len(failed)
        success_rate = (len(completed) / total_finished * 100) if total_finished > 0 else 0
        
        return {
            'total_executions': len(executions),
            'executions_per_hour': len(executions) / time_period_hours,
            'average_tokens_per_execution': avg_tokens,
            'average_cost_per_execution': avg_cost,
            'average_execution_time': avg_time,
            'success_rate': success_rate
        }
    
    def get_recent_executions(self, limit: int = 10) -> List[AgentExecution]:
        """Get recently created executions"""
        return self.find_by(
            filters={},
            limit=limit,
            order_by='created_at',
            order_desc=True
        )
