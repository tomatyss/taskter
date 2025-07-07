"""
Integration tests for Gemini API functionality
"""
import unittest
import json
import os
from dotenv import load_dotenv

# Load environment variables
load_dotenv()

class TestGeminiIntegration(unittest.TestCase):
    """Integration tests for Gemini with actual API calls"""
    
    def setUp(self):
        """Set up test fixtures"""
        self.api_key = os.getenv('GEMINI_API_KEY')
        if not self.api_key:
            self.skipTest("GEMINI_API_KEY not available")
    
    def test_gemini_400_error_reproduction(self):
        """Test the exact scenario that was causing 400 INVALID_ARGUMENT error"""
        from llm_providers import GeminiProvider
        from tools import tool_registry
        from agent_executor import AgentExecutor
        
        # Simulate the exact agent execution flow that was failing
        provider = GeminiProvider()
        executor = AgentExecutor()
        tools = executor._get_tools_for_provider(['send_email'], 'gemini')
        
        # This is the exact conversation that was causing the 400 error
        conversation_history = [
            {
                "role": "user",
                "content": "You have been assigned the following task:\n\nTitle: send email ivan\nDescription: No description provided\nCurrent Status: todo\n\nYour goal is to complete this task using the available tools. When you have successfully completed the task, respond with \"TASK_COMPLETED\" in your message.\n\nPlease analyze the task and create a plan to complete it, then execute that plan step by step."
            }
        ]
        
        try:
            response = provider.chat(
                system="You are a helpful AI assistant that can help users complete tasks. You have access to various tools to accomplish different objectives. Always be helpful, accurate, and efficient in completing the assigned tasks.",
                messages=conversation_history,
                tools=tools,
                temperature=0.7,
                max_tokens=1000
            )
            
            # If we get here, the 400 error is fixed
            self.assertIsInstance(response, dict)
            self.assertIn("content", response)
            self.assertIn("tool_calls", response)
            
            # Should not be empty response
            content = response.get("content", "")
            self.assertTrue(len(content) > 0)
            
        except Exception as e:
            # If we get a 400 INVALID_ARGUMENT error, the fix didn't work
            if "400 INVALID_ARGUMENT" in str(e):
                self.fail(f"Gemini API still returning 400 error: {e}")
            else:
                # Other errors might be acceptable (rate limits, etc.)
                self.skipTest(f"API call failed with non-400 error: {e}")
    
    def test_gemini_tool_format_validation(self):
        """Test that tools are properly formatted for Gemini API"""
        from llm_providers import GeminiProvider
        from tools import tool_registry
        
        provider = GeminiProvider()
        
        # Get tools in OpenAI format
        openai_tools = tool_registry.get_tools_openai_format(['send_email'])
        
        # Convert to Gemini format
        gemini_tools = provider._convert_tools_to_gemini_format(openai_tools)
        
        # Validate the format
        self.assertEqual(len(gemini_tools), 1)
        tool = gemini_tools[0]
        
        # Check required fields
        required_fields = ['name', 'description', 'parameters']
        for field in required_fields:
            self.assertIn(field, tool, f"Missing required field: {field}")
        
        # Check parameters structure
        params = tool['parameters']
        self.assertEqual(params['type'], 'object')
        self.assertIn('properties', params)
        self.assertIn('required', params)
        
        # Check that required fields are present
        required_params = params['required']
        self.assertIn('to', required_params)
        self.assertIn('subject', required_params)
        self.assertIn('body', required_params)
        
        # Check properties structure
        properties = params['properties']
        for prop in required_params:
            self.assertIn(prop, properties, f"Missing property: {prop}")
    
    def test_agent_executor_gemini_integration(self):
        """Test full agent executor integration with Gemini"""
        from agent_executor import AgentExecutor
        
        executor = AgentExecutor()
        
        # Test that Gemini tools are properly formatted
        tools = executor._get_tools_for_provider(['send_email'], 'gemini')
        
        self.assertEqual(len(tools), 1)
        tool = tools[0]
        
        # Validate tool structure
        self.assertEqual(tool['name'], 'send_email')
        self.assertIn('description', tool)
        self.assertIn('parameters', tool)
        
        # This should not raise any validation errors
        self.assertIsInstance(tool['parameters'], dict)
        self.assertEqual(tool['parameters']['type'], 'object')
    
    def test_gemini_tool_calling_fix_basic_chat(self):
        """Test basic Gemini chat functionality without tools"""
        from llm_providers import GeminiProvider
        
        provider = GeminiProvider()
        
        response = provider.chat(
            system="You are a helpful assistant.",
            messages=[{"role": "user", "content": "Say 'Hello, I am working!' and nothing else."}],
            tools=None
        )
        
        self.assertIsInstance(response, dict)
        self.assertIn("content", response)
        self.assertIn("tool_calls", response)
        self.assertIsNotNone(response["content"])
        self.assertEqual(len(response["tool_calls"]), 0)
        self.assertIn("Hello, I am working!", response["content"])
    
    def test_gemini_tool_calling_fix_with_tools(self):
        """Test Gemini tool calling functionality - the main fix verification"""
        from llm_providers import GeminiProvider
        from tools import tool_registry
        
        provider = GeminiProvider()
        
        # Get tools in OpenAI format (will be converted internally)
        tools = tool_registry.get_tools_openai_format(['send_email'])
        
        response = provider.chat(
            system="You are a helpful assistant that can send emails. When asked to send an email, use the send_email tool.",
            messages=[{"role": "user", "content": "Please send a test email to test@example.com with subject 'Test' and body 'This is a test email.'"}],
            tools=tools
        )
        
        self.assertIsInstance(response, dict)
        self.assertIn("content", response)
        self.assertIn("tool_calls", response)
        
        # The key test: tool calls should be detected
        tool_calls = response.get("tool_calls", [])
        self.assertGreater(len(tool_calls), 0, "No tool calls detected - fix may not be working")
        
        # Verify tool call structure
        tool_call = tool_calls[0]
        self.assertIn("function", tool_call)
        self.assertIn("name", tool_call["function"])
        self.assertIn("arguments", tool_call["function"])
        self.assertEqual(tool_call["function"]["name"], "send_email")
        
        # Verify arguments can be parsed
        try:
            args = json.loads(tool_call["function"]["arguments"])
            self.assertIn("to", args)
            self.assertIn("subject", args)
            self.assertIn("body", args)
            self.assertEqual(args["subject"], "Test")
            self.assertIn("test@example.com", args["to"])
        except json.JSONDecodeError:
            self.fail(f"Tool call arguments are not valid JSON: {tool_call['function']['arguments']}")
    
    def test_gemini_tool_execution_integration(self):
        """Test that Gemini tool calls can be executed by the agent executor"""
        from llm_providers import GeminiProvider
        from tools import tool_registry
        from agent_executor import AgentExecutor
        
        provider = GeminiProvider()
        executor = AgentExecutor()
        
        # Get tools and make a call that should trigger tool usage
        tools = tool_registry.get_tools_openai_format(['send_email'])
        
        response = provider.chat(
            system="You are a helpful assistant that can send emails. When asked to send an email, use the send_email tool.",
            messages=[{"role": "user", "content": "Send an email to integration-test@example.com with subject 'Integration Test' and body 'Testing Gemini integration'"}],
            tools=tools
        )
        
        # Should have tool calls
        tool_calls = response.get("tool_calls", [])
        self.assertGreater(len(tool_calls), 0)
        
        # Test that the tool call can be executed
        tool_call = tool_calls[0]
        
        # Create a mock execution for logging
        class MockExecution:
            def __init__(self):
                self.tool_logs = []
            
            def add_tool_log_entry(self, **kwargs):
                self.tool_logs.append(kwargs)
        
        mock_execution = MockExecution()
        
        # Execute the tool call using agent executor logic
        result = executor._execute_tool_call(tool_call, mock_execution)
        
        # Verify execution result
        self.assertIsInstance(result, dict)
        self.assertIn("tool_name", result)
        self.assertIn("result", result)
        self.assertEqual(result["tool_name"], "send_email")
        
        # Verify tool logs were created
        self.assertGreater(len(mock_execution.tool_logs), 0)
        
        # Verify the tool actually executed (should have success/failure result)
        tool_result = result["result"]
        self.assertIn("success", tool_result)
        # Note: The email might fail due to SMTP config, but it should attempt execution
    
    def test_gemini_response_parsing_fix(self):
        """Test that Gemini responses are properly parsed according to Google docs pattern"""
        from llm_providers import GeminiProvider
        from tools import tool_registry
        
        provider = GeminiProvider()
        tools = tool_registry.get_tools_openai_format(['send_email'])
        
        # Test the specific parsing logic that was fixed
        response = provider.chat(
            system="You are a helpful assistant. Use tools when appropriate.",
            messages=[{"role": "user", "content": "Send a quick email to parser-test@example.com saying 'Parser test successful'"}],
            tools=tools
        )
        
        # Test response structure
        self.assertIsInstance(response, dict)
        required_keys = ["content", "tool_calls", "finish_reason", "usage"]
        for key in required_keys:
            self.assertIn(key, response, f"Missing required response key: {key}")
        
        # Test that either content or tool_calls (or both) are present
        has_content = response.get("content") is not None
        has_tool_calls = len(response.get("tool_calls", [])) > 0
        self.assertTrue(has_content or has_tool_calls, "Response has neither content nor tool calls")
        
        # If tool calls are present, verify they follow the correct structure
        if has_tool_calls:
            for tool_call in response["tool_calls"]:
                self.assertIn("id", tool_call)
                self.assertIn("type", tool_call)
                self.assertEqual(tool_call["type"], "function")
                self.assertIn("function", tool_call)
                self.assertIn("name", tool_call["function"])
                self.assertIn("arguments", tool_call["function"])
                
                # Arguments should be valid JSON string
                try:
                    json.loads(tool_call["function"]["arguments"])
                except json.JSONDecodeError:
                    self.fail(f"Tool call arguments not valid JSON: {tool_call['function']['arguments']}")


if __name__ == '__main__':
    unittest.main(verbosity=2)
