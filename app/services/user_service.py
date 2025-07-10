from typing import Optional

from app.repositories.user_repository import UserRepository
from app.models.user import User


class UserService:
    """Service for user-related operations"""

    def __init__(self, user_repo: Optional[UserRepository] = None):
        self.user_repo = user_repo or UserRepository()

    def create_user(self, username: str, password: str) -> User:
        user = User(username=username)
        user.set_password(password)
        return self.user_repo.create(user)

    def authenticate(self, username: str, password: str) -> Optional[User]:
        user = self.user_repo.find_one_by({"username": username})
        if user and user.check_password(password):
            return user
        return None
