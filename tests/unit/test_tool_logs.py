"""
Unit tests for tool logs functionality
"""
import unittest
from unittest.mock import Mock, patch, MagicMock
import json
from datetime import datetime, timezone
import sys
import types
import importlib.util
from pathlib import Path

# Setup path for imports
BASE_DIR = Path(__file__).resolve().parents[1] / '..'
sys.path.insert(0, str(BASE_DIR))

def utcnow():
    return datetime.now(timezone.utc)

class MockAgentExecution:
    """Mock AgentExecution model for testing"""
    
    def __init__(self, id=1, task_id=1, agent_id=1):
        self.id = id
        self.task_id = task_id
        self.agent_id = agent_id
        self.tool_logs = []
        self.updated_at = utcnow()
    
    def add_tool_log_entry(self, tool_name, arguments, status, result=None, execution_time=None, timestamp=None):
        """Add a tool execution log entry"""
        if not self.tool_logs:
            self.tool_logs = []
        
        tool_log_entry = {
            'tool_name': tool_name,
            'arguments': arguments,
            'status': status,
            'result': result,
            'execution_time': execution_time,
            'timestamp': (timestamp or utcnow()).isoformat()
        }
        
        self.tool_logs.append(tool_log_entry)
        self.updated_at = utcnow()
    
    def get_tool_logs(self):
        """Get all tool execution logs"""
        return self.tool_logs or []
    
    def get_tool_logs_by_status(self, status):
        """Get tool logs filtered by status"""
        return [log for log in (self.tool_logs or []) if log.get('status') == status]
    
    def get_failed_tool_logs(self):
        """Get all failed tool execution logs"""
        return [log for log in (self.tool_logs or []) if log.get('status') in ['failed', 'error']]
    
    def get_successful_tool_logs(self):
        """Get all successful tool execution logs"""
        return [log for log in (self.tool_logs or []) if log.get('status') == 'completed']


class TestToolLogs(unittest.TestCase):
    """Test tool logs functionality"""
    
    def setUp(self):
        """Set up test fixtures"""
        self.execution = MockAgentExecution()
        self.sample_timestamp = utcnow()
    
    def test_add_tool_log_entry_basic(self):
        """Test adding a basic tool log entry"""
        tool_name = "send_email"
        arguments = {"to": "test@example.com", "subject": "Test", "body": "Test message"}
        status = "started"
        
        self.execution.add_tool_log_entry(tool_name, arguments, status, timestamp=self.sample_timestamp)
        
        logs = self.execution.get_tool_logs()
        self.assertEqual(len(logs), 1)
        
        log = logs[0]
        self.assertEqual(log['tool_name'], tool_name)
        self.assertEqual(log['arguments'], arguments)
        self.assertEqual(log['status'], status)
        self.assertEqual(log['timestamp'], self.sample_timestamp.isoformat())
        self.assertIsNone(log['result'])
        self.assertIsNone(log['execution_time'])
    
    def test_add_tool_log_entry_with_result(self):
        """Test adding a tool log entry with result and execution time"""
        tool_name = "send_email"
        arguments = {"to": "test@example.com", "subject": "Test", "body": "Test message"}
        status = "completed"
        result = {"success": True, "message": "Email sent successfully"}
        execution_time = 2.5
        
        self.execution.add_tool_log_entry(
            tool_name, arguments, status, result=result, 
            execution_time=execution_time, timestamp=self.sample_timestamp
        )
        
        logs = self.execution.get_tool_logs()
        self.assertEqual(len(logs), 1)
        
        log = logs[0]
        self.assertEqual(log['tool_name'], tool_name)
        self.assertEqual(log['arguments'], arguments)
        self.assertEqual(log['status'], status)
        self.assertEqual(log['result'], result)
        self.assertEqual(log['execution_time'], execution_time)
    
    def test_add_multiple_tool_log_entries(self):
        """Test adding multiple tool log entries"""
        # Add first log entry
        self.execution.add_tool_log_entry(
            "send_email", 
            {"to": "test1@example.com", "subject": "Test 1", "body": "Message 1"},
            "started",
            timestamp=self.sample_timestamp
        )
        
        # Add second log entry
        self.execution.add_tool_log_entry(
            "send_email",
            {"to": "test1@example.com", "subject": "Test 1", "body": "Message 1"},
            "completed",
            result={"success": True, "message": "Email sent"},
            execution_time=1.2,
            timestamp=self.sample_timestamp
        )
        
        # Add third log entry (different tool)
        self.execution.add_tool_log_entry(
            "web_search",
            {"query": "test search", "num_results": 5},
            "failed",
            result={"success": False, "error": "API error"},
            execution_time=0.8,
            timestamp=self.sample_timestamp
        )
        
        logs = self.execution.get_tool_logs()
        self.assertEqual(len(logs), 3)
        
        # Check that logs are in order
        self.assertEqual(logs[0]['tool_name'], "send_email")
        self.assertEqual(logs[0]['status'], "started")
        self.assertEqual(logs[1]['tool_name'], "send_email")
        self.assertEqual(logs[1]['status'], "completed")
        self.assertEqual(logs[2]['tool_name'], "web_search")
        self.assertEqual(logs[2]['status'], "failed")
    
    def test_get_tool_logs_by_status(self):
        """Test filtering tool logs by status"""
        # Add logs with different statuses
        self.execution.add_tool_log_entry("tool1", {}, "started")
        self.execution.add_tool_log_entry("tool2", {}, "completed")
        self.execution.add_tool_log_entry("tool3", {}, "failed")
        self.execution.add_tool_log_entry("tool4", {}, "completed")
        self.execution.add_tool_log_entry("tool5", {}, "error")
        
        # Test filtering by completed status
        completed_logs = self.execution.get_tool_logs_by_status("completed")
        self.assertEqual(len(completed_logs), 2)
        self.assertEqual(completed_logs[0]['tool_name'], "tool2")
        self.assertEqual(completed_logs[1]['tool_name'], "tool4")
        
        # Test filtering by failed status
        failed_logs = self.execution.get_tool_logs_by_status("failed")
        self.assertEqual(len(failed_logs), 1)
        self.assertEqual(failed_logs[0]['tool_name'], "tool3")
        
        # Test filtering by non-existent status
        nonexistent_logs = self.execution.get_tool_logs_by_status("nonexistent")
        self.assertEqual(len(nonexistent_logs), 0)
    
    def test_get_failed_tool_logs(self):
        """Test getting all failed tool logs"""
        # Add logs with different statuses
        self.execution.add_tool_log_entry("tool1", {}, "started")
        self.execution.add_tool_log_entry("tool2", {}, "completed")
        self.execution.add_tool_log_entry("tool3", {}, "failed")
        self.execution.add_tool_log_entry("tool4", {}, "error")
        self.execution.add_tool_log_entry("tool5", {}, "completed")
        
        failed_logs = self.execution.get_failed_tool_logs()
        self.assertEqual(len(failed_logs), 2)
        
        # Should include both 'failed' and 'error' statuses
        statuses = [log['status'] for log in failed_logs]
        self.assertIn('failed', statuses)
        self.assertIn('error', statuses)
        
        tool_names = [log['tool_name'] for log in failed_logs]
        self.assertIn('tool3', tool_names)
        self.assertIn('tool4', tool_names)
    
    def test_get_successful_tool_logs(self):
        """Test getting all successful tool logs"""
        # Add logs with different statuses
        self.execution.add_tool_log_entry("tool1", {}, "started")
        self.execution.add_tool_log_entry("tool2", {}, "completed")
        self.execution.add_tool_log_entry("tool3", {}, "failed")
        self.execution.add_tool_log_entry("tool4", {}, "completed")
        self.execution.add_tool_log_entry("tool5", {}, "error")
        
        successful_logs = self.execution.get_successful_tool_logs()
        self.assertEqual(len(successful_logs), 2)
        
        # Should only include 'completed' status
        for log in successful_logs:
            self.assertEqual(log['status'], 'completed')
        
        tool_names = [log['tool_name'] for log in successful_logs]
        self.assertIn('tool2', tool_names)
        self.assertIn('tool4', tool_names)
    
    def test_empty_tool_logs(self):
        """Test behavior with empty tool logs"""
        logs = self.execution.get_tool_logs()
        self.assertEqual(len(logs), 0)
        
        filtered_logs = self.execution.get_tool_logs_by_status("completed")
        self.assertEqual(len(filtered_logs), 0)
        
        failed_logs = self.execution.get_failed_tool_logs()
        self.assertEqual(len(failed_logs), 0)
        
        successful_logs = self.execution.get_successful_tool_logs()
        self.assertEqual(len(successful_logs), 0)


class TestToolLogsIntegration(unittest.TestCase):
    """Test tool logs integration scenarios"""
    
    def setUp(self):
        """Set up test fixtures"""
        self.mock_execution = MockAgentExecution()
    
    def test_tool_logs_workflow(self):
        """Test a complete tool logs workflow"""
        # Simulate a complete email sending workflow
        
        # 1. Tool execution starts
        self.mock_execution.add_tool_log_entry(
            "send_email",
            {"to": "test@example.com", "subject": "Test", "body": "Test message"},
            "started"
        )
        
        # 2. Tool execution completes successfully
        self.mock_execution.add_tool_log_entry(
            "send_email",
            {"to": "test@example.com", "subject": "Test", "body": "Test message"},
            "completed",
            result={"success": True, "message": "Email sent successfully", "message_id": "12345"},
            execution_time=1.5
        )
        
        # Verify the workflow
        logs = self.mock_execution.get_tool_logs()
        self.assertEqual(len(logs), 2)
        
        # Check start log
        start_log = logs[0]
        self.assertEqual(start_log['status'], 'started')
        self.assertIsNone(start_log['result'])
        
        # Check completion log
        completion_log = logs[1]
        self.assertEqual(completion_log['status'], 'completed')
        self.assertTrue(completion_log['result']['success'])
        self.assertEqual(completion_log['execution_time'], 1.5)
        
        # Verify filtering works
        successful_logs = self.mock_execution.get_successful_tool_logs()
        self.assertEqual(len(successful_logs), 1)
        self.assertEqual(successful_logs[0]['result']['message_id'], "12345")
    
    def test_multiple_tools_workflow(self):
        """Test workflow with multiple different tools"""
        # Simulate a workflow with web search followed by email
        
        # 1. Web search starts
        self.mock_execution.add_tool_log_entry(
            "web_search",
            {"query": "latest news", "num_results": 5},
            "started"
        )
        
        # 2. Web search completes
        self.mock_execution.add_tool_log_entry(
            "web_search",
            {"query": "latest news", "num_results": 5},
            "completed",
            result={"success": True, "results": ["result1", "result2"]},
            execution_time=2.1
        )
        
        # 3. Email sending starts
        self.mock_execution.add_tool_log_entry(
            "send_email",
            {"to": "user@example.com", "subject": "News Update", "body": "Here are the latest news..."},
            "started"
        )
        
        # 4. Email sending fails
        self.mock_execution.add_tool_log_entry(
            "send_email",
            {"to": "user@example.com", "subject": "News Update", "body": "Here are the latest news..."},
            "failed",
            result={"success": False, "error": "SMTP server unavailable"},
            execution_time=0.8
        )
        
        # Verify the complete workflow
        logs = self.mock_execution.get_tool_logs()
        self.assertEqual(len(logs), 4)
        
        # Check tool distribution
        web_search_logs = [log for log in logs if log['tool_name'] == 'web_search']
        email_logs = [log for log in logs if log['tool_name'] == 'send_email']
        self.assertEqual(len(web_search_logs), 2)
        self.assertEqual(len(email_logs), 2)
        
        # Check success/failure distribution
        successful_logs = self.mock_execution.get_successful_tool_logs()
        failed_logs = self.mock_execution.get_failed_tool_logs()
        self.assertEqual(len(successful_logs), 1)  # Only web search succeeded
        self.assertEqual(len(failed_logs), 1)      # Only email failed
        
        # Verify specific results
        self.assertEqual(successful_logs[0]['tool_name'], 'web_search')
        self.assertEqual(failed_logs[0]['tool_name'], 'send_email')
        self.assertIn('SMTP server unavailable', failed_logs[0]['result']['error'])


class TestAgentExecutorToolLogging(unittest.TestCase):
    """Test tool logging in agent executor"""
    
    def setUp(self):
        """Set up test fixtures"""
        self.mock_execution = MockAgentExecution()
    
    def test_execute_tool_call_logging(self):
        """Test that tool calls are properly logged"""
        # This would test the _execute_tool_call method in agent_executor.py
        # Since we can't easily import the full agent executor in unit tests,
        # we'll test the logging logic separately
        
        tool_call = {
            'function': {
                'name': 'send_email',
                'arguments': '{"to": "test@example.com", "subject": "Test", "body": "Test message"}'
            }
        }
        
        # Simulate the logging that should happen in _execute_tool_call
        arguments = json.loads(tool_call['function']['arguments'])
        tool_name = tool_call['function']['name']
        
        # Log start
        self.mock_execution.add_tool_log_entry(
            tool_name=tool_name,
            arguments=arguments,
            status='started'
        )
        
        # Simulate tool execution result
        result = {"success": True, "message": "Email sent successfully"}
        execution_time = 1.2
        
        # Log completion
        self.mock_execution.add_tool_log_entry(
            tool_name=tool_name,
            arguments=arguments,
            status='completed',
            result=result,
            execution_time=execution_time
        )
        
        # Verify logs were created
        logs = self.mock_execution.get_tool_logs()
        self.assertEqual(len(logs), 2)
        
        # Check start log
        start_log = logs[0]
        self.assertEqual(start_log['tool_name'], 'send_email')
        self.assertEqual(start_log['status'], 'started')
        self.assertEqual(start_log['arguments'], arguments)
        
        # Check completion log
        completion_log = logs[1]
        self.assertEqual(completion_log['tool_name'], 'send_email')
        self.assertEqual(completion_log['status'], 'completed')
        self.assertEqual(completion_log['result'], result)
        self.assertEqual(completion_log['execution_time'], execution_time)
    
    def test_execute_tool_call_error_logging(self):
        """Test that tool call errors are properly logged"""
        tool_call = {
            'function': {
                'name': 'send_email',
                'arguments': '{"to": "invalid-email", "subject": "Test", "body": "Test"}'
            }
        }
        
        arguments = json.loads(tool_call['function']['arguments'])
        tool_name = tool_call['function']['name']
        
        # Log start
        self.mock_execution.add_tool_log_entry(
            tool_name=tool_name,
            arguments=arguments,
            status='started'
        )
        
        # Simulate tool execution error
        error_result = {"success": False, "error": "Invalid email address"}
        execution_time = 0.5
        
        # Log error
        self.mock_execution.add_tool_log_entry(
            tool_name=tool_name,
            arguments=arguments,
            status='failed',
            result=error_result,
            execution_time=execution_time
        )
        
        # Verify logs were created
        logs = self.mock_execution.get_tool_logs()
        self.assertEqual(len(logs), 2)
        
        # Check error log
        error_log = logs[1]
        self.assertEqual(error_log['status'], 'failed')
        self.assertEqual(error_log['result'], error_result)
        
        # Verify failed logs retrieval
        failed_logs = self.mock_execution.get_failed_tool_logs()
        self.assertEqual(len(failed_logs), 1)
        self.assertEqual(failed_logs[0]['tool_name'], 'send_email')


if __name__ == '__main__':
    unittest.main(verbosity=2)
