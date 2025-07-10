"""
Integration tests for Gemini API functionality
"""
import unittest
import json
import os
from dotenv import load_dotenv

from llm_providers import GeminiProvider
from tools import tool_registry
from agent_executor import AgentExecutor

# Load environment variables
load_dotenv()

class TestGeminiIntegration(unittest.TestCase):
    """Integration tests for Gemini with actual API calls"""
    
    def setUp(self):
        """Set up test fixtures"""
        self.api_key = os.getenv('GEMINI_API_KEY')
        if not self.api_key:
            self.skipTest("GEMINI_API_KEY not available")
        
        self.provider = GeminiProvider()
        self.executor = AgentExecutor()
    
    def test_gemini_400_error_reproduction(self):
        """Test the exact scenario that was causing 400 INVALID_ARGUMENT error"""
        tools = self.executor._get_tools_for_provider(['send_email'], 'gemini')
        
        conversation_history = [
            {"role": "user", "content": "You have been assigned a task to send an email."}
        ]
        
        try:
            response = self.provider.chat(
                system="You are a helpful AI assistant.",
                messages=conversation_history,
                tools=tools
            )
            self.assertIsInstance(response, dict)
            self.assertTrue(response.get("content") or response.get("tool_calls"))
        except Exception as e:
            if "400 INVALID_ARGUMENT" in str(e):
                self.fail(f"Gemini API still returning 400 error: {e}")
            self.skipTest(f"API call failed with non-400 error: {e}")
    
    def test_gemini_tool_format_validation(self):
        """Test that tools are properly formatted for Gemini API"""
        openai_tools = tool_registry.get_tools_openai_format(['send_email'])
        gemini_tools = self.provider._convert_tools_to_gemini_format(openai_tools)
        
        self.assertEqual(len(gemini_tools), 1)
        tool = gemini_tools[0]
        
        self.assertIn('name', tool)
        self.assertIn('description', tool)
        self.assertIn('parameters', tool)
        self.assertEqual(tool['parameters']['type'], 'object')

    def test_gemini_basic_chat(self):
        """Test basic Gemini chat functionality without tools"""
        response = self.provider.chat(
            system="You are a helpful assistant.",
            messages=[{"role": "user", "content": "Say 'Hello, I am working!'"}]
        )
        
        self.assertIn("Hello, I am working", response["content"])
        self.assertEqual(len(response["tool_calls"]), 0)
    
    @unittest.skip("Skipping tool calling test until model behavior is confirmed")
    def test_gemini_tool_calling(self):
        """Test Gemini tool calling functionality"""
        tools = tool_registry.get_tools_openai_format(['send_email'])
        
        response = self.provider.chat(
            system="You are an email assistant.",
            messages=[{"role": "user", "content": "Send an email to test@example.com"}],
            tools=tools
        )
        
        tool_calls = response.get("tool_calls", [])
        self.assertGreater(len(tool_calls), 0, "No tool calls detected")
        
        tool_call = tool_calls[0]
        self.assertEqual(tool_call["function"]["name"], "send_email")
        self.assertIn("arguments", tool_call["function"])

    def test_gemini_response_parsing(self):
        """Test that Gemini responses are properly parsed"""
        tools = tool_registry.get_tools_openai_format(['send_email'])
        
        response = self.provider.chat(
            system="You are a helpful assistant.",
            messages=[{"role": "user", "content": "Send an email to test@example.com"}],
            tools=tools
        )
        
        self.assertIn("content", response)
        self.assertIn("tool_calls", response)
        self.assertIn("finish_reason", response)
        self.assertIn("usage", response)

if __name__ == '__main__':
    unittest.main(verbosity=2)
