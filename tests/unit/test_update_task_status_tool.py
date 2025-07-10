import pytest
from app.models.task import Task
from app.core.constants import TaskStatus
from tools import UpdateTaskStatusTool
from db import db


def test_update_task_status_tool(db_session, monkeypatch):
    monkeypatch.setattr(db, "session", db_session)

    task = Task(title="Test Task")
    db_session.add(task)
    db_session.commit()
    db_session.refresh(task)

    tool = UpdateTaskStatusTool()
    result = tool.execute(task_id=task.id, status=TaskStatus.IN_PROGRESS.value)

    assert result["success"] is True
    db_session.refresh(task)
    assert task.status == TaskStatus.IN_PROGRESS.value
