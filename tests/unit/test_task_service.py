import sys
import types
import importlib.util
from pathlib import Path
import pytest

# Load constants and exceptions directly from files to avoid heavy imports
BASE_DIR = Path(__file__).resolve().parents[1] / '..'
constants_path = BASE_DIR / 'app' / 'core' / 'constants.py'
exceptions_path = BASE_DIR / 'app' / 'core' / 'exceptions.py'

const_spec = importlib.util.spec_from_file_location('constants', constants_path)
constants = importlib.util.module_from_spec(const_spec)
const_spec.loader.exec_module(constants)

exc_spec = importlib.util.spec_from_file_location('exceptions', exceptions_path)
exceptions = importlib.util.module_from_spec(exc_spec)
exc_spec.loader.exec_module(exceptions)

TaskStatus = constants.TaskStatus
TaskValidationError = exceptions.TaskValidationError

# --- Create stub modules to satisfy imports ---
class DummyTask:
    def __init__(self, title, description=None, status=None):
        self.title = title
        self.description = description
        self.status = status
        self.id = None

    @classmethod
    def from_dict(cls, data):
        return cls(
            title=data.get('title'),
            description=data.get('description'),
            status=data.get('status')
        )

    def update_from_dict(self, data):
        self.__dict__.update(data)


class DummyRepository:
    def __init__(self):
        self.created = []

    def create(self, task):
        task.id = len(self.created) + 1
        self.created.append(task)
        return task


class DummyLogger:
    def info(self, *args, **kwargs):
        pass

    def error(self, *args, **kwargs):
        pass

log_module = types.ModuleType('app.core.logging')
log_module.get_logger = lambda name=None: DummyLogger()
log_module.log_database_operation = lambda *a, **k: None
sys.modules['app.core.logging'] = log_module

repo_module = types.ModuleType('app.repositories.task_repository')
repo_module.TaskRepository = DummyRepository
sys.modules['app.repositories.task_repository'] = repo_module

model_module = types.ModuleType('app.models.task')
model_module.Task = DummyTask
sys.modules['app.models.task'] = model_module

# Provide constants and exceptions to service import
sys.modules['app.core.constants'] = constants
sys.modules['app.core.exceptions'] = exceptions

# Dynamically load the service module without importing the package
service_path = BASE_DIR / 'app' / 'services' / 'task_service.py'
service_spec = importlib.util.spec_from_file_location('task_service', service_path)
service_module = importlib.util.module_from_spec(service_spec)
service_spec.loader.exec_module(service_module)
TaskService = service_module.TaskService


def test_create_task_success():
    service = TaskService()
    task = service.create_task('Test', 'Desc')

    assert task.id == 1
    assert task.title == 'Test'
    assert task.description == 'Desc'
    assert task.status == TaskStatus.TODO.value


def test_create_task_validation_error():
    service = TaskService()
    with pytest.raises(TaskValidationError):
        service.create_task('   ')


