"""
Unit tests for Gemini LLM provider
"""
import unittest
from unittest.mock import Mock, patch, MagicMock
import json
import sys
import types
import importlib.util
from pathlib import Path

# Setup path for imports
BASE_DIR = Path(__file__).resolve().parents[1] / '..'
sys.path.insert(0, str(BASE_DIR))


class TestGeminiProvider(unittest.TestCase):
    """Unit tests for GeminiProvider class"""
    
    def setUp(self):
        """Set up test fixtures"""
        # Mock the google.genai module to avoid import issues in tests
        self.mock_genai = Mock()
        self.mock_types = Mock()
        self.mock_client = Mock()
        
        # Setup mock types
        self.mock_types.Tool = Mock()
        self.mock_types.GenerateContentConfig = Mock()
        
        # Setup mock client
        self.mock_genai.Client = Mock(return_value=self.mock_client)
        
        # Patch the imports
        self.genai_patcher = patch.dict('sys.modules', {
            'google': Mock(),
            'google.genai': self.mock_genai,
            'google.genai.types': self.mock_types
        })
        self.genai_patcher.start()
        
        # Now import the provider
        from llm_providers import GeminiProvider
        self.GeminiProvider = GeminiProvider
    
    def tearDown(self):
        """Clean up after tests"""
        self.genai_patcher.stop()
    
    def test_gemini_provider_initialization(self):
        """Test GeminiProvider initialization"""
        with patch.dict('os.environ', {'GEMINI_API_KEY': 'test-key'}):
            provider = self.GeminiProvider()
            
            # Should have initialized client with API key
            self.mock_genai.Client.assert_called_once_with(api_key='test-key')
            self.assertEqual(provider.model, 'gemini-2.5-flash')
            self.assertEqual(provider.api_key, 'test-key')
    
    def test_gemini_provider_initialization_custom_model(self):
        """Test GeminiProvider initialization with custom model"""
        with patch.dict('os.environ', {'GEMINI_API_KEY': 'test-key'}):
            provider = self.GeminiProvider(model='gemini-pro')
            
            self.assertEqual(provider.model, 'gemini-pro')
    
    def test_gemini_provider_initialization_no_api_key(self):
        """Test GeminiProvider initialization without API key"""
        with patch.dict('os.environ', {}, clear=True):
            with self.assertRaises(ValueError) as context:
                self.GeminiProvider()
            
            self.assertIn("Gemini API key is required", str(context.exception))
    
    def test_convert_tools_to_gemini_format(self):
        """Test tool format conversion from OpenAI to Gemini"""
        with patch.dict('os.environ', {'GEMINI_API_KEY': 'test-key'}):
            provider = self.GeminiProvider()
            
            openai_tools = [{
                "type": "function",
                "function": {
                    "name": "send_email",
                    "description": "Send an email to recipients",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "to": {"type": "array", "items": {"type": "string"}},
                            "subject": {"type": "string"},
                            "body": {"type": "string"}
                        },
                        "required": ["to", "subject", "body"]
                    }
                }
            }]
            
            gemini_tools = provider._convert_tools_to_gemini_format(openai_tools)
            
            self.assertEqual(len(gemini_tools), 1)
            tool = gemini_tools[0]
            
            self.assertEqual(tool["name"], "send_email")
            self.assertEqual(tool["description"], "Send an email to recipients")
            self.assertEqual(tool["parameters"]["type"], "object")
            self.assertIn("properties", tool["parameters"])
            self.assertIn("required", tool["parameters"])
    
    def test_convert_tools_empty_list(self):
        """Test tool conversion with empty list"""
        with patch.dict('os.environ', {'GEMINI_API_KEY': 'test-key'}):
            provider = self.GeminiProvider()
            
            result = provider._convert_tools_to_gemini_format([])
            self.assertEqual(result, [])
    
    def test_convert_tools_invalid_format(self):
        """Test tool conversion with invalid format"""
        with patch.dict('os.environ', {'GEMINI_API_KEY': 'test-key'}):
            provider = self.GeminiProvider()
            
            invalid_tools = [{"type": "invalid", "data": "test"}]
            result = provider._convert_tools_to_gemini_format(invalid_tools)
            
            # Should return empty list for invalid tools
            self.assertEqual(len(result), 0)
    
    def test_format_conversation_for_gemini(self):
        """Test conversation formatting for Gemini"""
        with patch.dict('os.environ', {'GEMINI_API_KEY': 'test-key'}):
            provider = self.GeminiProvider()
            
            system = "You are a helpful assistant."
            messages = [
                {"role": "user", "content": "Hello"},
                {"role": "assistant", "content": "Hi there!"},
                {"role": "user", "content": "How are you?"}
            ]
            
            formatted = provider._format_conversation_for_gemini(system, messages)
            
            self.assertIn("System Instructions: You are a helpful assistant.", formatted)
            self.assertIn("User: Hello", formatted)
            self.assertIn("Assistant: Hi there!", formatted)
            self.assertIn("User: How are you?", formatted)
    
    def test_format_conversation_with_tool_results(self):
        """Test conversation formatting with tool results"""
        with patch.dict('os.environ', {'GEMINI_API_KEY': 'test-key'}):
            provider = self.GeminiProvider()
            
            system = "You are a helpful assistant."
            messages = [
                {"role": "user", "content": "Send an email"},
                {"role": "tool", "content": '{"success": true, "message": "Email sent"}'}
            ]
            
            formatted = provider._format_conversation_for_gemini(system, messages)
            
            self.assertIn("System Instructions:", formatted)
            self.assertIn("User: Send an email", formatted)
            self.assertIn("Tool Result:", formatted)
            self.assertIn("Email sent", formatted)
    
    def test_chat_without_tools(self):
        """Test chat method without tools"""
        with patch.dict('os.environ', {'GEMINI_API_KEY': 'test-key'}):
            provider = self.GeminiProvider()
            
            # Mock response
            mock_response = Mock()
            mock_response.candidates = [Mock()]
            mock_response.candidates[0].content = Mock()
            mock_response.candidates[0].content.parts = [Mock()]
            mock_response.candidates[0].content.parts[0].text = "Hello, I'm working!"
            
            # Mock the generate_content method
            self.mock_client.models.generate_content.return_value = mock_response
            
            response = provider.chat(
                system="You are helpful.",
                messages=[{"role": "user", "content": "Say hello"}],
                tools=None
            )
            
            # Verify API call was made
            self.mock_client.models.generate_content.assert_called_once()
            
            # Verify response structure
            self.assertIsInstance(response, dict)
            self.assertIn("content", response)
            self.assertIn("tool_calls", response)
            self.assertEqual(response["content"], "Hello, I'm working!")
            self.assertEqual(len(response["tool_calls"]), 0)
    
    def test_chat_with_tools_function_call(self):
        """Test chat method with tools that triggers function call"""
        with patch.dict('os.environ', {'GEMINI_API_KEY': 'test-key'}):
            provider = self.GeminiProvider()
            
            # Mock response with function call
            mock_response = Mock()
            mock_response.candidates = [Mock()]
            mock_response.candidates[0].content = Mock()
            mock_response.candidates[0].content.parts = [Mock()]
            
            # Mock function call
            mock_function_call = Mock()
            mock_function_call.name = "send_email"
            mock_function_call.args = {"to": ["test@example.com"], "subject": "Test", "body": "Test message"}
            
            mock_response.candidates[0].content.parts[0].function_call = mock_function_call
            
            # Mock the generate_content method
            self.mock_client.models.generate_content.return_value = mock_response
            
            tools = [{
                "type": "function",
                "function": {
                    "name": "send_email",
                    "description": "Send email",
                    "parameters": {"type": "object"}
                }
            }]
            
            response = provider.chat(
                system="You can send emails.",
                messages=[{"role": "user", "content": "Send an email"}],
                tools=tools
            )
            
            # Verify response structure
            self.assertIsInstance(response, dict)
            self.assertIn("tool_calls", response)
            self.assertEqual(len(response["tool_calls"]), 1)
            
            tool_call = response["tool_calls"][0]
            self.assertEqual(tool_call["function"]["name"], "send_email")
            
            # Verify arguments are JSON string
            args = json.loads(tool_call["function"]["arguments"])
            self.assertEqual(args["subject"], "Test")
    
    def test_chat_with_tools_configuration(self):
        """Test that tools are properly configured in API call"""
        with patch.dict('os.environ', {'GEMINI_API_KEY': 'test-key'}):
            provider = self.GeminiProvider()
            
            # Mock response
            mock_response = Mock()
            mock_response.candidates = [Mock()]
            mock_response.candidates[0].content = Mock()
            mock_response.candidates[0].content.parts = [Mock()]
            mock_response.candidates[0].content.parts[0].text = "I'll send the email."
            
            self.mock_client.models.generate_content.return_value = mock_response
            
            tools = [{
                "type": "function",
                "function": {
                    "name": "send_email",
                    "description": "Send email",
                    "parameters": {"type": "object"}
                }
            }]
            
            provider.chat(
                system="You can send emails.",
                messages=[{"role": "user", "content": "Send an email"}],
                tools=tools
            )
            
            # Verify that Tool and GenerateContentConfig were called
            self.mock_types.Tool.assert_called_once()
            self.mock_types.GenerateContentConfig.assert_called_once()
            
            # Verify generate_content was called with config
            call_args = self.mock_client.models.generate_content.call_args
            self.assertIn("config", call_args.kwargs)
    
    def test_chat_error_handling(self):
        """Test error handling in chat method"""
        with patch.dict('os.environ', {'GEMINI_API_KEY': 'test-key'}):
            provider = self.GeminiProvider()
            
            # Mock API error
            self.mock_client.models.generate_content.side_effect = Exception("API Error")
            
            with self.assertRaises(Exception) as context:
                provider.chat(
                    system="Test",
                    messages=[{"role": "user", "content": "Test"}],
                    tools=None
                )
            
            self.assertIn("API Error", str(context.exception))
    
    def test_get_provider_name(self):
        """Test get_provider_name method"""
        with patch.dict('os.environ', {'GEMINI_API_KEY': 'test-key'}):
            provider = self.GeminiProvider()
            self.assertEqual(provider.get_provider_name(), "gemini")
    
    def test_chat_response_fallback_to_text(self):
        """Test fallback to response.text when parts are not available"""
        with patch.dict('os.environ', {'GEMINI_API_KEY': 'test-key'}):
            provider = self.GeminiProvider()
            
            # Mock response with text attribute but no candidates/parts
            mock_response = Mock()
            mock_response.candidates = []
            mock_response.text = "Fallback response text"
            
            self.mock_client.models.generate_content.return_value = mock_response
            
            response = provider.chat(
                system="Test",
                messages=[{"role": "user", "content": "Test"}],
                tools=None
            )
            
            self.assertEqual(response["content"], "Fallback response text")
    
    def test_chat_empty_response_handling(self):
        """Test handling of empty responses"""
        with patch.dict('os.environ', {'GEMINI_API_KEY': 'test-key'}):
            provider = self.GeminiProvider()
            
            # Mock empty response
            mock_response = Mock()
            mock_response.candidates = []
            # No text attribute either
            
            self.mock_client.models.generate_content.return_value = mock_response
            
            response = provider.chat(
                system="Test",
                messages=[{"role": "user", "content": "Test"}],
                tools=None
            )
            
            self.assertIsNone(response["content"])
            self.assertEqual(len(response["tool_calls"]), 0)


class TestGeminiProviderIntegration(unittest.TestCase):
    """Integration-style tests for GeminiProvider with mocked dependencies"""
    
    def setUp(self):
        """Set up test fixtures"""
        # Mock the google.genai module
        self.mock_genai = Mock()
        self.mock_types = Mock()
        self.mock_client = Mock()
        
        self.mock_types.Tool = Mock()
        self.mock_types.GenerateContentConfig = Mock()
        self.mock_genai.Client = Mock(return_value=self.mock_client)
        
        self.genai_patcher = patch.dict('sys.modules', {
            'google': Mock(),
            'google.genai': self.mock_genai,
            'google.genai.types': self.mock_types
        })
        self.genai_patcher.start()
    
    def tearDown(self):
        """Clean up after tests"""
        self.genai_patcher.stop()
    
    def test_gemini_tool_calling_workflow(self):
        """Test complete tool calling workflow"""
        from llm_providers import GeminiProvider
        
        with patch.dict('os.environ', {'GEMINI_API_KEY': 'test-key'}):
            provider = GeminiProvider()
            
            # Mock a complete workflow: user request -> function call -> tool result -> final response
            
            # First call: user request, should return function call
            mock_response1 = Mock()
            mock_response1.candidates = [Mock()]
            mock_response1.candidates[0].content = Mock()
            mock_response1.candidates[0].content.parts = [Mock()]
            
            mock_function_call = Mock()
            mock_function_call.name = "send_email"
            mock_function_call.args = {
                "to": ["user@example.com"],
                "subject": "Test Email",
                "body": "This is a test email."
            }
            mock_response1.candidates[0].content.parts[0].function_call = mock_function_call
            
            self.mock_client.models.generate_content.return_value = mock_response1
            
            tools = [{
                "type": "function",
                "function": {
                    "name": "send_email",
                    "description": "Send an email",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "to": {"type": "array"},
                            "subject": {"type": "string"},
                            "body": {"type": "string"}
                        },
                        "required": ["to", "subject", "body"]
                    }
                }
            }]
            
            # Make the first call
            response1 = provider.chat(
                system="You are an email assistant.",
                messages=[{"role": "user", "content": "Send an email to user@example.com saying hello"}],
                tools=tools
            )
            
            # Verify function call was detected
            self.assertEqual(len(response1["tool_calls"]), 1)
            tool_call = response1["tool_calls"][0]
            self.assertEqual(tool_call["function"]["name"], "send_email")
            
            # Parse and verify arguments
            args = json.loads(tool_call["function"]["arguments"])
            self.assertEqual(args["subject"], "Test Email")
            self.assertIn("user@example.com", args["to"])
            
            # Simulate second call with tool result
            mock_response2 = Mock()
            mock_response2.candidates = [Mock()]
            mock_response2.candidates[0].content = Mock()
            mock_response2.candidates[0].content.parts = [Mock()]
            mock_response2.candidates[0].content.parts[0].text = "Email sent successfully!"
            
            self.mock_client.models.generate_content.return_value = mock_response2
            
            # Second call with tool result
            conversation_with_result = [
                {"role": "user", "content": "Send an email to user@example.com saying hello"},
                {"role": "tool", "content": '{"success": true, "message": "Email sent successfully"}'}
            ]
            
            response2 = provider.chat(
                system="You are an email assistant.",
                messages=conversation_with_result,
                tools=tools
            )
            
            # Verify final response
            self.assertEqual(response2["content"], "Email sent successfully!")
            self.assertEqual(len(response2["tool_calls"]), 0)


if __name__ == '__main__':
    unittest.main(verbosity=2)
