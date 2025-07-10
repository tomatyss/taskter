from app.repositories.base import BaseRepository
from app.models.user import User


class UserRepository(BaseRepository[User]):
    """Repository for User model"""

    def __init__(self):
        super().__init__(User)
