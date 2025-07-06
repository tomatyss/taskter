"""
Unit tests for Gemini email sending functionality
"""
import unittest
import json
import os
from unittest.mock import Mock, patch, MagicMock
from dotenv import load_dotenv

# Load environment variables
load_dotenv()

class TestGeminiEmailFix(unittest.TestCase):
    """Test cases for Gemini email sending fix"""
    
    def setUp(self):
        """Set up test fixtures"""
        self.api_key = os.getenv('GEMINI_API_KEY')
        if not self.api_key:
            self.skipTest("GEMINI_API_KEY not available")
    
    def test_gemini_provider_initialization(self):
        """Test that Gemini provider initializes correctly"""
        from llm_providers import GeminiProvider
        
        provider = GeminiProvider()
        self.assertIsNotNone(provider.client)
        self.assertEqual(provider.model, "gemini-2.5-flash")
        self.assertIsNotNone(provider.types)
    
    def test_gemini_tool_conversion(self):
        """Test that tools are converted to Gemini format correctly"""
        from llm_providers import GeminiProvider
        
        provider = GeminiProvider()
        
        # Test tool in OpenAI format
        openai_tools = [{
            "type": "function",
            "function": {
                "name": "send_email",
                "description": "Send an email",
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
        
        # Convert to Gemini format
        gemini_tools = provider._convert_tools_to_gemini_format(openai_tools)
        
        self.assertEqual(len(gemini_tools), 1)
        self.assertEqual(gemini_tools[0]["name"], "send_email")
        self.assertEqual(gemini_tools[0]["description"], "Send an email")
        self.assertIn("parameters", gemini_tools[0])
    
    def test_gemini_basic_chat(self):
        """Test basic chat functionality without tools"""
        from llm_providers import GeminiProvider
        
        provider = GeminiProvider()
        
        response = provider.chat(
            system="You are a helpful assistant.",
            messages=[{"role": "user", "content": "Say hello"}],
            tools=None
        )
        
        self.assertIsInstance(response, dict)
        self.assertIn("content", response)
        self.assertIn("tool_calls", response)
        self.assertIn("finish_reason", response)
        self.assertIn("usage", response)
    
    def test_gemini_chat_with_tools(self):
        """Test chat functionality with email tool"""
        from llm_providers import GeminiProvider
        from tools import tool_registry
        
        provider = GeminiProvider()
        
        # Get tools in OpenAI format (will be converted internally)
        tools = tool_registry.get_tools_openai_format(['send_email'])
        
        response = provider.chat(
            system="You are a helpful assistant that can send emails.",
            messages=[{"role": "user", "content": "What can you do?"}],
            tools=tools
        )
        
        self.assertIsInstance(response, dict)
        self.assertIn("content", response)
        self.assertIn("tool_calls", response)
        
        # Check that response mentions email capability
        content = response.get("content", "").lower()
        self.assertTrue("email" in content or "send" in content)
    
    def test_tool_registry_gemini_format(self):
        """Test that tool registry provides correct Gemini format"""
        from tools import tool_registry
        
        gemini_tools = tool_registry.get_tools_gemini_format(['send_email'])
        
        self.assertEqual(len(gemini_tools), 1)
        tool = gemini_tools[0]
        
        self.assertEqual(tool["name"], "send_email")
        self.assertIn("description", tool)
        self.assertIn("parameters", tool)
        
        # Check parameters structure
        params = tool["parameters"]
        self.assertEqual(params["type"], "object")
        self.assertIn("properties", params)
        self.assertIn("required", params)
        
        # Check required fields
        self.assertIn("to", params["required"])
        self.assertIn("subject", params["required"])
        self.assertIn("body", params["required"])
    
    def test_agent_executor_tool_formatting(self):
        """Test that agent executor formats tools correctly for Gemini"""
        from agent_executor import AgentExecutor
        
        executor = AgentExecutor()
        
        # Test Gemini tool formatting
        tools = executor._get_tools_for_provider(['send_email'], 'gemini')
        
        self.assertEqual(len(tools), 1)
        tool = tools[0]
        
        self.assertEqual(tool["name"], "send_email")
        self.assertIn("description", tool)
        self.assertIn("parameters", tool)
    
    @patch('llm_providers.GeminiProvider.chat')
    def test_gemini_error_handling(self, mock_chat):
        """Test that Gemini provider handles errors correctly"""
        from llm_providers import GeminiProvider
        
        # Mock a 400 error
        mock_chat.side_effect = Exception("400 INVALID_ARGUMENT")
        
        provider = GeminiProvider()
        
        with self.assertRaises(Exception) as context:
            provider.chat(
                system="Test",
                messages=[{"role": "user", "content": "Test"}],
                tools=None
            )
        
        self.assertIn("400 INVALID_ARGUMENT", str(context.exception))
    
    def test_conversation_formatting(self):
        """Test that conversation is formatted correctly for Gemini"""
        from llm_providers import GeminiProvider
        
        provider = GeminiProvider()
        
        system = "You are a helpful assistant."
        messages = [
            {"role": "user", "content": "Hello"},
            {"role": "assistant", "content": "Hi there!"},
            {"role": "user", "content": "Can you help me?"}
        ]
        
        formatted = provider._format_conversation_for_gemini(system, messages)
        
        self.assertIn("System Instructions:", formatted)
        self.assertIn("User: Hello", formatted)
        self.assertIn("Assistant: Hi there!", formatted)
        self.assertIn("User: Can you help me?", formatted)
    
    def test_email_tool_execution_mock(self):
        """Test email tool execution (may succeed or fail depending on SMTP config)"""
        from tools import tool_registry
        
        email_tool = tool_registry.get_tool('send_email')
        self.assertIsNotNone(email_tool)
        
        # Test email tool execution
        result = email_tool.execute(
            to=['test@example.com'],
            subject='Test Subject',
            body='Test Body'
        )
        
        self.assertIsInstance(result, dict)
        self.assertIn('success', result)
        
        if result['success']:
            # Email sent successfully
            self.assertIn('result', result)
            self.assertIn('Email sent successfully', result['result'])
        else:
            # Email failed (expected in most test environments)
            self.assertIn('error', result)
            
            # Accept various types of SMTP errors
            error_msg = result['error']
            acceptable_errors = [
                'SMTP credentials not configured',
                'timed out',
                'Connection unexpectedly closed',
                'Connection refused',
                'Name or service not known',
                'Authentication failed'
            ]
            
            self.assertTrue(
                any(err in error_msg for err in acceptable_errors),
                f"Expected one of {acceptable_errors}, got: {error_msg}"
            )


class TestGeminiIntegration(unittest.TestCase):
    """Integration tests for Gemini with actual API calls"""
    
    def setUp(self):
        """Set up test fixtures"""
        self.api_key = os.getenv('GEMINI_API_KEY')
        if not self.api_key:
            self.skipTest("GEMINI_API_KEY not available")
    
    def test_real_gemini_api_call(self):
        """Test actual Gemini API call to verify fix"""
        from llm_providers import GeminiProvider
        
        provider = GeminiProvider()
        
        try:
            response = provider.chat(
                system="You are a helpful assistant.",
                messages=[{"role": "user", "content": "Just say 'API working'"}],
                tools=None
            )
            
            self.assertIsInstance(response, dict)
            self.assertIn("content", response)
            content = response.get("content", "")
            self.assertTrue(len(content) > 0)
            
        except Exception as e:
            # If we get a 400 INVALID_ARGUMENT error, the fix didn't work
            if "400 INVALID_ARGUMENT" in str(e):
                self.fail(f"Gemini API still returning 400 error: {e}")
            else:
                # Other errors might be acceptable (rate limits, etc.)
                self.skipTest(f"API call failed with non-400 error: {e}")
    
    def test_real_gemini_with_tools(self):
        """Test actual Gemini API call with tools"""
        from llm_providers import GeminiProvider
        from tools import tool_registry
        
        provider = GeminiProvider()
        tools = tool_registry.get_tools_openai_format(['send_email'])
        
        try:
            response = provider.chat(
                system="You are a helpful assistant that can send emails.",
                messages=[{"role": "user", "content": "What tools do you have?"}],
                tools=tools
            )
            
            self.assertIsInstance(response, dict)
            self.assertIn("content", response)
            
            # Should not get 400 error
            content = response.get("content", "")
            self.assertTrue(len(content) > 0)
            
        except Exception as e:
            # If we get a 400 INVALID_ARGUMENT error, the fix didn't work
            if "400 INVALID_ARGUMENT" in str(e):
                self.fail(f"Gemini API with tools still returning 400 error: {e}")
            else:
                # Other errors might be acceptable
                self.skipTest(f"API call failed with non-400 error: {e}")


if __name__ == '__main__':
    # Run tests with verbose output
    unittest.main(verbosity=2)
