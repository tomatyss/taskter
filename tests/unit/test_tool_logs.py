"""
Unit tests for tool logs functionality on the AgentExecution model.
"""
import pytest
from datetime import datetime, timezone
import json

from app.models.execution import AgentExecution

def utcnow():
    return datetime.now(timezone.utc)

@pytest.fixture
def sample_execution(db_session):
    """Fixture to create a sample AgentExecution instance."""
    execution = AgentExecution(task_id=1, agent_id=1, tool_logs=[])
    db_session.add(execution)
    db_session.commit()
    db_session.refresh(execution)
    return execution

class TestToolLogs:
    """Test suite for tool logging functionality."""

    def test_add_tool_log_entry_basic(self, db_session, sample_execution):
        """Test adding a basic tool log entry."""
        tool_name = "send_email"
        arguments = {"to": "test@example.com", "subject": "Test"}
        status = "started"
        
        sample_execution.add_tool_log_entry(tool_name, arguments, status)
        db_session.commit()

        logs = sample_execution.get_tool_logs()
        assert len(logs) == 1
        log = logs[0]
        assert log['tool_name'] == tool_name
        assert log['arguments'] == arguments
        assert log['status'] == status
        assert 'timestamp' in log
        assert log['result'] is None
        assert log['execution_time'] is None

    def test_add_tool_log_entry_with_result(self, db_session, sample_execution):
        """Test adding a tool log entry with result and execution time."""
        tool_name = "send_email"
        arguments = {"to": "test@example.com"}
        status = "completed"
        result = {"success": True, "message": "Email sent"}
        execution_time = 2.5
        
        sample_execution.add_tool_log_entry(
            tool_name, arguments, status, result=result, execution_time=execution_time
        )
        db_session.commit()

        logs = sample_execution.get_tool_logs()
        assert len(logs) == 1
        log = logs[0]
        assert log['result'] == result
        assert log['execution_time'] == execution_time

    def test_add_multiple_tool_log_entries(self, db_session, sample_execution):
        """Test adding multiple tool log entries."""
        sample_execution.add_tool_log_entry("tool1", {}, "started")
        sample_execution.add_tool_log_entry("tool1", {}, "completed", result={"success": True})
        sample_execution.add_tool_log_entry("tool2", {}, "failed", result={"success": False})
        db_session.commit()

        logs = sample_execution.get_tool_logs()
        assert len(logs) == 3
        assert logs[0]['status'] == 'started'
        assert logs[1]['status'] == 'completed'
        assert logs[2]['status'] == 'failed'

    def test_get_tool_logs_by_status(self, db_session, sample_execution):
        """Test filtering tool logs by status."""
        sample_execution.add_tool_log_entry("tool1", {}, "started")
        sample_execution.add_tool_log_entry("tool2", {}, "completed")
        sample_execution.add_tool_log_entry("tool3", {}, "failed")
        sample_execution.add_tool_log_entry("tool4", {}, "completed")
        db_session.commit()

        completed_logs = sample_execution.get_tool_logs_by_status("completed")
        assert len(completed_logs) == 2
        assert completed_logs[0]['tool_name'] == "tool2"
        assert completed_logs[1]['tool_name'] == "tool4"

    def test_get_failed_tool_logs(self, db_session, sample_execution):
        """Test getting all failed tool logs."""
        sample_execution.add_tool_log_entry("tool1", {}, "completed")
        sample_execution.add_tool_log_entry("tool2", {}, "failed")
        sample_execution.add_tool_log_entry("tool3", {}, "error")
        db_session.commit()

        failed_logs = sample_execution.get_failed_tool_logs()
        assert len(failed_logs) == 2
        statuses = {log['status'] for log in failed_logs}
        assert statuses == {'failed', 'error'}

    def test_get_successful_tool_logs(self, db_session, sample_execution):
        """Test getting all successful tool logs."""
        sample_execution.add_tool_log_entry("tool1", {}, "started")
        sample_execution.add_tool_log_entry("tool2", {}, "completed")
        sample_execution.add_tool_log_entry("tool3", {}, "failed")
        db_session.commit()

        successful_logs = sample_execution.get_successful_tool_logs()
        assert len(successful_logs) == 1
        assert successful_logs[0]['status'] == 'completed'

    def test_empty_tool_logs(self, sample_execution):
        """Test behavior with empty tool logs."""
        assert sample_execution.get_tool_logs() == []
        assert sample_execution.get_tool_logs_by_status("completed") == []
        assert sample_execution.get_failed_tool_logs() == []
        assert sample_execution.get_successful_tool_logs() == []
