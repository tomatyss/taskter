"""
Unit tests for Gemini LLM provider
"""
import unittest
from unittest.mock import Mock, patch, MagicMock
import json
import os
import sys
from pathlib import Path

# Setup path for imports
BASE_DIR = Path(__file__).resolve().parents[1] / '..'
sys.path.insert(0, str(BASE_DIR))

class TestGeminiProvider(unittest.TestCase):
    """Unit tests for GeminiProvider class"""
    
    def setUp(self):
        """Set up test fixtures"""
        # Mock the google.genai module to avoid import issues in tests
        self.mock_genai = MagicMock()
        self.mock_types = MagicMock()
        
        # Setup mock types
        self.mock_types.Tool = MagicMock()
        self.mock_types.GenerateContentResponse = MagicMock
        self.mock_types.GenerateContentConfig = MagicMock()
        
        # Patch the imports
        self.genai_patcher = patch.dict('sys.modules', {
            'google': MagicMock(),
            'google.genai': self.mock_genai,
            'google.genai.types': self.mock_types
        })
        self.genai_patcher.start()

        # Patch environment variables
        self.env_patcher = patch.dict(os.environ, {'GEMINI_API_KEY': 'test-key'})
        self.env_patcher.start()
        
        # Now import the provider
        from llm_providers import GeminiProvider
        self.GeminiProvider = GeminiProvider
        
        # Create a provider instance for use in tests
        self.provider = self.GeminiProvider()
        self.mock_genai.Client.assert_called_once_with(api_key='test-key')
        self.mock_genai.reset_mock()

    def tearDown(self):
        """Clean up after tests"""
        self.genai_patcher.stop()
        self.env_patcher.stop()
    
    def test_gemini_provider_initialization_custom_model(self):
        """Test GeminiProvider initialization with custom model"""
        self.mock_genai.reset_mock()
        provider = self.GeminiProvider(model='gemini-pro')
        self.assertEqual(provider.model, 'gemini-pro')
        self.mock_genai.Client.assert_called_once_with(api_key='test-key')
    
    def test_gemini_provider_initialization_no_api_key(self):
        """Test GeminiProvider initialization without API key"""
        self.env_patcher.stop() # Stop the patch to simulate no key
        with patch.dict(os.environ, {}, clear=True):
            with self.assertRaises(ValueError) as context:
                from llm_providers import GeminiProvider
                GeminiProvider()
            self.assertIn("Gemini API key is required", str(context.exception))
        self.env_patcher.start() # Restart for other tests

    def test_convert_tools_to_gemini_format(self):
        """Test tool format conversion from OpenAI to Gemini"""
        openai_tools = [{
            "type": "function",
            "function": {
                "name": "send_email",
                "description": "Send an email",
                "parameters": {
                    "type": "object",
                    "properties": {"to": {"type": "string"}},
                    "required": ["to"]
                }
            }
        }]
        
        gemini_tools = self.provider._convert_tools_to_gemini_format(openai_tools)
        
        self.assertEqual(len(gemini_tools), 1)
        tool = gemini_tools[0]
        self.assertEqual(tool["name"], "send_email")
        self.assertEqual(tool["description"], "Send an email")
        self.assertIn("parameters", tool)

    def test_format_conversation_for_gemini(self):
        """Test conversation formatting for Gemini"""
        system = "You are a helpful assistant."
        messages = [{"role": "user", "content": "Hello"}]
        
        formatted = self.provider._format_conversation_for_gemini(system, messages)
        
        self.assertIn("System Instructions: You are a helpful assistant.", formatted)
        self.assertIn("User: Hello", formatted)

    def test_chat_without_tools(self):
        """Test chat method without tools"""
        mock_response = self.mock_genai.GenerativeModel.return_value.generate_content.return_value
        mock_response.text = "Hello, I'm working!"
        
        response = self.provider.chat(
            system="You are helpful.",
            messages=[{"role": "user", "content": "Say hello"}],
            tools=None
        )
        
        self.assertEqual(response["content"], "Hello, I'm working!")
        self.assertEqual(len(response["tool_calls"]), 0)

    def test_chat_with_tools_function_call(self):
        """Test chat method with tools that triggers function call"""
        mock_response = self.mock_genai.GenerativeModel.return_value.generate_content.return_value
        
        mock_function_call = MagicMock()
        mock_function_call.name = "send_email"
        mock_function_call.args = {"to": "test@example.com"}
        
        # Simulate the structure Gemini returns
        mock_part = MagicMock()
        mock_part.function_call = mock_function_call
        mock_response.candidates[0].content.parts = [mock_part]
        mock_response.text = None # No text part when a tool call is made

        tools = [{"type": "function", "function": {"name": "send_email"}}]
        
        response = self.provider.chat(
            system="You can send emails.",
            messages=[{"role": "user", "content": "Send an email"}],
            tools=tools
        )
        
        self.assertEqual(len(response["tool_calls"]), 1)
        tool_call = response["tool_calls"][0]
        self.assertEqual(tool_call["function"]["name"], "send_email")
        args = json.loads(tool_call["function"]["arguments"])
        self.assertEqual(args["to"], "test@example.com")

    def test_chat_error_handling(self):
        """Test error handling in chat method"""
        self.mock_genai.GenerativeModel.return_value.generate_content.side_effect = Exception("API Error")
        
        with self.assertRaises(Exception) as context:
            self.provider.chat(system="Test", messages=[{"role": "user", "content": "Test"}])
        
        self.assertIn("API Error", str(context.exception))

    def test_get_provider_name(self):
        """Test get_provider_name method"""
        self.assertEqual(self.provider.get_provider_name(), "gemini")

if __name__ == '__main__':
    unittest.main(verbosity=2)
