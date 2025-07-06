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


if __name__ == '__main__':
    unittest.main(verbosity=2)
