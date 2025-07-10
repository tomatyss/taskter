"""
Unit tests for the TaskService.
"""
import pytest
from unittest.mock import MagicMock

from app.services.task_service import TaskService
from app.core.exceptions import TaskValidationError, TaskNotFoundError
from app.core.constants import TaskStatus

@pytest.fixture
def mock_task_repo():
    """Fixture for a mocked TaskRepository."""
    return MagicMock()

@pytest.fixture
def mock_logger():
    """Fixture for a mocked logger."""
    return MagicMock()

@pytest.fixture
def task_service(monkeypatch, mock_task_repo, mock_logger):
    """Fixture to create a TaskService instance with mocked dependencies."""
    monkeypatch.setattr('app.services.task_service.TaskRepository', lambda: mock_task_repo)
    monkeypatch.setattr('app.services.task_service.get_logger', lambda name: mock_logger)
    return TaskService()

class TestTaskService:
    """Test suite for the TaskService."""

    def test_create_task_success(self, task_service, mock_task_repo):
        """Test successful task creation."""
        # Configure the mock repository to return the task passed to it
        mock_task_repo.create.side_effect = lambda task: task

        title = "My Test Task"
        description = "A description for the test task."
        
        task = task_service.create_task(title, description)

        assert task.title == title
        assert task.description == description
        assert task.status == TaskStatus.TODO.value
        
        # Verify that the repository's create method was called once
        mock_task_repo.create.assert_called_once()
        created_task_arg = mock_task_repo.create.call_args[0][0]
        assert created_task_arg.title == title

    def test_create_task_with_empty_title(self, task_service):
        """Test that creating a task with an empty or whitespace title raises a validation error."""
        with pytest.raises(TaskValidationError, match="Task title is required"):
            task_service.create_task("   ", "A description")

    def test_create_task_no_description(self, task_service, mock_task_repo):
        """Test creating a task with no description."""
        mock_task_repo.create.side_effect = lambda task: task
        
        title = "Task without description"
        task = task_service.create_task(title)

        assert task.title == title
        assert task.description is None
        assert task.status == TaskStatus.TODO.value
        mock_task_repo.create.assert_called_once()

    def test_get_task_by_id_found(self, task_service, mock_task_repo):
        """Test retrieving a task by its ID when it exists."""
        mock_task = MagicMock()
        mock_task.id = 1
        mock_task.title = "Found Task"
        mock_task_repo.get_by_id.return_value = mock_task

        task = task_service.get_task_by_id(1)

        assert task is not None
        assert task.id == 1
        assert task.title == "Found Task"
        mock_task_repo.get_by_id.assert_called_once_with(1)

    def test_get_task_by_id_not_found(self, task_service, mock_task_repo):
        """Test retrieving a task by its ID when it does not exist."""
        mock_task_repo.get_by_id.return_value = None

        with pytest.raises(TaskNotFoundError):
            task_service.get_task_by_id(999)
        
        mock_task_repo.get_by_id.assert_called_once_with(999)

    def test_move_task_to_status(self, task_service, mock_task_repo):
        """Test moving a task to a new status."""
        mock_task = MagicMock()
        mock_task.id = 1
        mock_task.status = TaskStatus.TODO.value
        # The real method calls move_to_status on the task object
        mock_task.move_to_status = MagicMock()
        
        mock_task_repo.get_by_id.return_value = mock_task
        mock_task_repo.update.return_value = mock_task

        updated_task = task_service.move_task_to_status(1, TaskStatus.IN_PROGRESS)

        assert updated_task is not None
        mock_task.move_to_status.assert_called_once_with(TaskStatus.IN_PROGRESS.value)
        mock_task_repo.get_by_id.assert_called_once_with(1)
        mock_task_repo.update.assert_called_once_with(mock_task)
