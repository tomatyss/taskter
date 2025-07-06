"""
Base repository pattern implementation for data access layer
"""
from abc import ABC, abstractmethod
from typing import TypeVar, Generic, List, Optional, Dict, Any
from sqlalchemy.orm import Session
from sqlalchemy.exc import SQLAlchemyError
from sqlalchemy import desc, asc

from app.core.exceptions import DatabaseError, NotFoundError
from app.core.logging import get_logger, log_database_operation
from db import db

T = TypeVar('T')

logger = get_logger(__name__)


class BaseRepository(Generic[T], ABC):
    """Base repository with common CRUD operations"""
    
    def __init__(self, model_class: type):
        self.model_class = model_class
        self.session: Session = db.session
    
    def create(self, entity: T) -> T:
        """Create a new entity"""
        try:
            start_time = self._get_time()
            self.session.add(entity)
            self.session.commit()
            self.session.refresh(entity)
            
            execution_time = self._get_time() - start_time
            log_database_operation(
                logger, "CREATE", self.model_class.__tablename__, 
                True, execution_time
            )
            
            return entity
            
        except SQLAlchemyError as e:
            self.session.rollback()
            log_database_operation(
                logger, "CREATE", self.model_class.__tablename__, False
            )
            raise DatabaseError(f"Failed to create {self.model_class.__name__}: {str(e)}")
    
    def get_by_id(self, entity_id: int) -> Optional[T]:
        """Get entity by ID"""
        try:
            start_time = self._get_time()
            entity = self.session.query(self.model_class).get(entity_id)
            
            execution_time = self._get_time() - start_time
            log_database_operation(
                logger, "SELECT", self.model_class.__tablename__, 
                True, execution_time
            )
            
            return entity
            
        except SQLAlchemyError as e:
            log_database_operation(
                logger, "SELECT", self.model_class.__tablename__, False
            )
            raise DatabaseError(f"Failed to get {self.model_class.__name__} by ID: {str(e)}")
    
    def get_by_id_or_404(self, entity_id: int) -> T:
        """Get entity by ID or raise NotFoundError"""
        entity = self.get_by_id(entity_id)
        if entity is None:
            raise NotFoundError(f"{self.model_class.__name__} with ID {entity_id} not found")
        return entity
    
    def get_all(self, limit: Optional[int] = None, offset: Optional[int] = None) -> List[T]:
        """Get all entities with optional pagination"""
        try:
            start_time = self._get_time()
            query = self.session.query(self.model_class)
            
            if offset:
                query = query.offset(offset)
            if limit:
                query = query.limit(limit)
            
            entities = query.all()
            
            execution_time = self._get_time() - start_time
            log_database_operation(
                logger, "SELECT", self.model_class.__tablename__, 
                True, execution_time
            )
            
            return entities
            
        except SQLAlchemyError as e:
            log_database_operation(
                logger, "SELECT", self.model_class.__tablename__, False
            )
            raise DatabaseError(f"Failed to get all {self.model_class.__name__}: {str(e)}")
    
    def update(self, entity: T) -> T:
        """Update an existing entity"""
        try:
            start_time = self._get_time()
            self.session.commit()
            self.session.refresh(entity)
            
            execution_time = self._get_time() - start_time
            log_database_operation(
                logger, "UPDATE", self.model_class.__tablename__, 
                True, execution_time
            )
            
            return entity
            
        except SQLAlchemyError as e:
            self.session.rollback()
            log_database_operation(
                logger, "UPDATE", self.model_class.__tablename__, False
            )
            raise DatabaseError(f"Failed to update {self.model_class.__name__}: {str(e)}")
    
    def delete(self, entity: T) -> bool:
        """Delete an entity"""
        try:
            start_time = self._get_time()
            self.session.delete(entity)
            self.session.commit()
            
            execution_time = self._get_time() - start_time
            log_database_operation(
                logger, "DELETE", self.model_class.__tablename__, 
                True, execution_time
            )
            
            return True
            
        except SQLAlchemyError as e:
            self.session.rollback()
            log_database_operation(
                logger, "DELETE", self.model_class.__tablename__, False
            )
            raise DatabaseError(f"Failed to delete {self.model_class.__name__}: {str(e)}")
    
    def delete_by_id(self, entity_id: int) -> bool:
        """Delete entity by ID"""
        entity = self.get_by_id_or_404(entity_id)
        return self.delete(entity)
    
    def count(self, filters: Optional[Dict[str, Any]] = None) -> int:
        """Count entities with optional filters"""
        try:
            start_time = self._get_time()
            query = self.session.query(self.model_class)
            
            if filters:
                query = self._apply_filters(query, filters)
            
            count = query.count()
            
            execution_time = self._get_time() - start_time
            log_database_operation(
                logger, "COUNT", self.model_class.__tablename__, 
                True, execution_time
            )
            
            return count
            
        except SQLAlchemyError as e:
            log_database_operation(
                logger, "COUNT", self.model_class.__tablename__, False
            )
            raise DatabaseError(f"Failed to count {self.model_class.__name__}: {str(e)}")
    
    def find_by(self, filters: Dict[str, Any], 
                limit: Optional[int] = None, 
                offset: Optional[int] = None,
                order_by: Optional[str] = None,
                order_desc: bool = False) -> List[T]:
        """Find entities by filters"""
        try:
            start_time = self._get_time()
            query = self.session.query(self.model_class)
            
            # Apply filters
            query = self._apply_filters(query, filters)
            
            # Apply ordering
            if order_by:
                column = getattr(self.model_class, order_by, None)
                if column:
                    if order_desc:
                        query = query.order_by(desc(column))
                    else:
                        query = query.order_by(asc(column))
            
            # Apply pagination
            if offset:
                query = query.offset(offset)
            if limit:
                query = query.limit(limit)
            
            entities = query.all()
            
            execution_time = self._get_time() - start_time
            log_database_operation(
                logger, "SELECT", self.model_class.__tablename__, 
                True, execution_time
            )
            
            return entities
            
        except SQLAlchemyError as e:
            log_database_operation(
                logger, "SELECT", self.model_class.__tablename__, False
            )
            raise DatabaseError(f"Failed to find {self.model_class.__name__}: {str(e)}")
    
    def find_one_by(self, filters: Dict[str, Any]) -> Optional[T]:
        """Find one entity by filters"""
        entities = self.find_by(filters, limit=1)
        return entities[0] if entities else None
    
    def exists(self, filters: Dict[str, Any]) -> bool:
        """Check if entity exists with given filters"""
        return self.count(filters) > 0
    
    def bulk_create(self, entities: List[T]) -> List[T]:
        """Create multiple entities in bulk"""
        try:
            start_time = self._get_time()
            self.session.add_all(entities)
            self.session.commit()
            
            for entity in entities:
                self.session.refresh(entity)
            
            execution_time = self._get_time() - start_time
            log_database_operation(
                logger, "BULK_CREATE", self.model_class.__tablename__, 
                True, execution_time
            )
            
            return entities
            
        except SQLAlchemyError as e:
            self.session.rollback()
            log_database_operation(
                logger, "BULK_CREATE", self.model_class.__tablename__, False
            )
            raise DatabaseError(f"Failed to bulk create {self.model_class.__name__}: {str(e)}")
    
    def bulk_update(self, updates: List[Dict[str, Any]]) -> int:
        """Update multiple entities in bulk"""
        try:
            start_time = self._get_time()
            result = self.session.bulk_update_mappings(self.model_class, updates)
            self.session.commit()
            
            execution_time = self._get_time() - start_time
            log_database_operation(
                logger, "BULK_UPDATE", self.model_class.__tablename__, 
                True, execution_time
            )
            
            return result.rowcount if hasattr(result, 'rowcount') else len(updates)
            
        except SQLAlchemyError as e:
            self.session.rollback()
            log_database_operation(
                logger, "BULK_UPDATE", self.model_class.__tablename__, False
            )
            raise DatabaseError(f"Failed to bulk update {self.model_class.__name__}: {str(e)}")
    
    def _apply_filters(self, query, filters: Dict[str, Any]):
        """Apply filters to query"""
        for key, value in filters.items():
            if hasattr(self.model_class, key):
                column = getattr(self.model_class, key)
                
                if isinstance(value, list):
                    # IN clause for lists
                    query = query.filter(column.in_(value))
                elif isinstance(value, dict):
                    # Handle complex filters
                    if 'gt' in value:
                        query = query.filter(column > value['gt'])
                    if 'gte' in value:
                        query = query.filter(column >= value['gte'])
                    if 'lt' in value:
                        query = query.filter(column < value['lt'])
                    if 'lte' in value:
                        query = query.filter(column <= value['lte'])
                    if 'like' in value:
                        query = query.filter(column.like(f"%{value['like']}%"))
                    if 'ilike' in value:
                        query = query.filter(column.ilike(f"%{value['ilike']}%"))
                else:
                    # Exact match
                    query = query.filter(column == value)
        
        return query
    
    def _get_time(self) -> float:
        """Get current time for performance measurement"""
        import time
        return time.time()
    
    @abstractmethod
    def get_model_class(self) -> type:
        """Get the model class for this repository"""
        return self.model_class


class PaginatedResult:
    """Container for paginated query results"""
    
    def __init__(self, items: List[T], total: int, page: int, per_page: int):
        self.items = items
        self.total = total
        self.page = page
        self.per_page = per_page
        self.pages = (total + per_page - 1) // per_page if per_page > 0 else 0
        self.has_prev = page > 1
        self.has_next = page < self.pages
        self.prev_num = page - 1 if self.has_prev else None
        self.next_num = page + 1 if self.has_next else None
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for API responses"""
        return {
            'items': self.items,
            'pagination': {
                'page': self.page,
                'per_page': self.per_page,
                'total': self.total,
                'pages': self.pages,
                'has_prev': self.has_prev,
                'has_next': self.has_next,
                'prev_num': self.prev_num,
                'next_num': self.next_num
            }
        }


def paginate_query(query, page: int = 1, per_page: int = 20) -> PaginatedResult:
    """Paginate a SQLAlchemy query"""
    total = query.count()
    items = query.offset((page - 1) * per_page).limit(per_page).all()
    return PaginatedResult(items, total, page, per_page)
