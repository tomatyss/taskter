"""
LLM Provider abstraction layer supporting OpenAI, Anthropic, and Google Gemini
"""
import os
import json
from abc import ABC, abstractmethod
from typing import Dict, List
import logging
from dotenv import load_dotenv

# Load environment variables from .env file
load_dotenv()

logger = logging.getLogger(__name__)

class LLMProvider(ABC):
    """Abstract base class for LLM providers"""
    
    @abstractmethod
    def chat(self, system: str, messages: List[Dict], tools: List[Dict] = None, **kwargs) -> Dict:
        """
        Send a chat completion request to the LLM
        
        Args:
            system: System instructions
            messages: List of conversation messages
            tools: Optional list of available tools
            **kwargs: Additional provider-specific parameters
            
        Returns:
            Dict with keys: content, tool_calls, finish_reason, usage
        """
        pass
    
    @abstractmethod
    def get_provider_name(self) -> str:
        """Return the provider name"""
        pass

class OpenAIProvider(LLMProvider):
    """OpenAI GPT provider"""
    
    def __init__(self, api_key: str = None, model: str = "gpt-4"):
        try:
            import openai
            from openai import OpenAI
        except ImportError:
            raise ImportError("openai package is required for OpenAI provider")
        
        self.api_key = api_key or os.getenv('OPENAI_API_KEY')
        if not self.api_key:
            raise ValueError("OpenAI API key is required")
        
        self.client = OpenAI(api_key=self.api_key)
        self.model = model
    
    def chat(self, system: str, messages: List[Dict], tools: List[Dict] = None, **kwargs) -> Dict:
        try:
            # Format messages for OpenAI
            formatted_messages = [{"role": "system", "content": system}]
            formatted_messages.extend(messages)
            
            # Prepare request parameters
            request_params = {
                "model": self.model,
                "messages": formatted_messages,
                "temperature": kwargs.get('temperature', 0.7),
                "max_tokens": kwargs.get('max_tokens', 1000)
            }
            
            # Add tools if provided
            if tools:
                request_params["tools"] = tools
                request_params["tool_choice"] = "auto"
            
            # Make API call
            response = self.client.chat.completions.create(**request_params)
            
            # Extract response data
            choice = response.choices[0]
            message = choice.message
            
            return {
                "content": message.content,
                "tool_calls": [
                    {
                        "id": tc.id,
                        "type": tc.type,
                        "function": {
                            "name": tc.function.name,
                            "arguments": tc.function.arguments
                        }
                    } for tc in (message.tool_calls or [])
                ],
                "finish_reason": choice.finish_reason,
                "usage": {
                    "prompt_tokens": response.usage.prompt_tokens,
                    "completion_tokens": response.usage.completion_tokens,
                    "total_tokens": response.usage.total_tokens
                }
            }
            
        except Exception as e:
            logger.error(f"OpenAI API error: {str(e)}")
            raise
    
    def get_provider_name(self) -> str:
        return "openai"

class AnthropicProvider(LLMProvider):
    """Anthropic Claude provider"""
    
    def __init__(self, api_key: str = None, model: str = "claude-3-5-sonnet-20241022"):
        try:
            import anthropic
        except ImportError:
            raise ImportError("anthropic package is required for Anthropic provider")
        
        self.api_key = api_key or os.getenv('ANTHROPIC_API_KEY')
        if not self.api_key:
            raise ValueError("Anthropic API key is required")
        
        self.client = anthropic.Anthropic(api_key=self.api_key)
        self.model = model
    
    def chat(self, system: str, messages: List[Dict], tools: List[Dict] = None, **kwargs) -> Dict:
        try:
            # Prepare request parameters
            request_params = {
                "model": self.model,
                "system": system,
                "messages": messages,
                "max_tokens": kwargs.get('max_tokens', 1000),
                "temperature": kwargs.get('temperature', 0.7)
            }
            
            # Add tools if provided
            if tools:
                request_params["tools"] = tools
            
            # Make API call
            response = self.client.messages.create(**request_params)
            
            # Extract tool calls if any
            tool_calls = []
            for content_block in response.content:
                if hasattr(content_block, 'type') and content_block.type == 'tool_use':
                    tool_calls.append({
                        "id": content_block.id,
                        "type": "function",
                        "function": {
                            "name": content_block.name,
                            "arguments": json.dumps(content_block.input)
                        }
                    })
            
            # Get text content
            text_content = ""
            for content_block in response.content:
                if hasattr(content_block, 'type') and content_block.type == 'text':
                    text_content += content_block.text
            
            return {
                "content": text_content if text_content else None,
                "tool_calls": tool_calls,
                "finish_reason": response.stop_reason,
                "usage": {
                    "prompt_tokens": response.usage.input_tokens,
                    "completion_tokens": response.usage.output_tokens,
                    "total_tokens": response.usage.input_tokens + response.usage.output_tokens
                }
            }
            
        except Exception as e:
            logger.error(f"Anthropic API error: {str(e)}")
            raise
    
    def get_provider_name(self) -> str:
        return "anthropic"

class GeminiProvider(LLMProvider):
    """Google Gemini provider"""
    
    def __init__(self, api_key: str = None, model: str = "gemini-2.5-flash"):
        try:
            from google import genai
            from google.genai import types
        except ImportError:
            raise ImportError("google-genai package is required for Gemini provider")
        
        # Store the types module for later use
        self.types = types
        
        # Get API key from parameter or environment
        self.api_key = api_key or os.getenv('GEMINI_API_KEY')
        
        # Validate API key is available
        if not self.api_key:
            raise ValueError("Gemini API key is required. Please set GEMINI_API_KEY environment variable or provide api_key parameter.")
        
        # Log the API key status for debugging (without exposing the actual key)
        logger.info(f"Gemini API key found: {self.api_key[:10]}...")
        
        # Initialize the client with explicit API key
        try:
            self.client = genai.Client(api_key=self.api_key)
            logger.info("Gemini client initialized successfully with explicit API key")
        except Exception as e:
            logger.error(f"Failed to initialize Gemini client: {str(e)}")
            raise ValueError(f"Failed to initialize Gemini client. Please verify your GEMINI_API_KEY is valid. Error: {str(e)}")
        
        self.model = model
    
    def _convert_tools_to_gemini_format(self, tools: List[Dict]) -> List[Dict]:
        """Convert OpenAI-style tools to Gemini function declarations"""
        function_declarations = []
        
        for tool in tools:
            if tool.get('type') == 'function':
                func = tool.get('function', {})
                function_declarations.append({
                    "name": func.get('name', ''),
                    "description": func.get('description', ''),
                    "parameters": func.get('parameters', {})
                })
        
        return function_declarations
    
    def _format_conversation_for_gemini(self, system: str, messages: List[Dict]) -> str:
        """Format conversation history for Gemini"""
        formatted_content = f"System Instructions: {system}\n\n"
        
        # Add conversation history
        for msg in messages:
            role = msg.get('role', 'user')
            content = msg.get('content', '')
            if content:  # Skip empty messages
                if role == 'tool':
                    # Format tool results
                    formatted_content += f"Tool Result: {content}\n"
                else:
                    formatted_content += f"{role.title()}: {content}\n"
        
        return formatted_content
    
    def chat(self, system: str, messages: List[Dict], tools: List[Dict] = None, **kwargs) -> Dict:
        try:
            # Format content for Gemini
            formatted_content = self._format_conversation_for_gemini(system, messages)
            
            # Prepare request parameters
            request_params = {
                "model": self.model,
                "contents": formatted_content
            }
            
            # Add tools if provided
            if tools:
                function_declarations = self._convert_tools_to_gemini_format(tools)
                if function_declarations:
                    gemini_tools = self.types.Tool(function_declarations=function_declarations)
                    config = self.types.GenerateContentConfig(tools=[gemini_tools])
                    request_params["config"] = config
            
            # Make API call
            response = self.client.models.generate_content(**request_params)
            
            # Extract response data
            tool_calls = []
            content = ""
            
            # Check if response has candidates
            if response.candidates and len(response.candidates) > 0:
                candidate = response.candidates[0]
                
                if candidate.content and candidate.content.parts:
                    for part in candidate.content.parts:
                        # Check for function calls
                        if hasattr(part, 'function_call') and part.function_call:
                            function_call = part.function_call
                            tool_calls.append({
                                "id": f"call_{len(tool_calls)}",
                                "type": "function",
                                "function": {
                                    "name": function_call.name,
                                    "arguments": json.dumps(dict(function_call.args))
                                }
                            })
                        
                        # Check for text content
                        elif hasattr(part, 'text') and part.text:
                            content += part.text
            
            # Fallback to response.text if no parts found
            if not content and not tool_calls:
                content = getattr(response, 'text', '')
            
            return {
                "content": content if content else None,
                "tool_calls": tool_calls,
                "finish_reason": "stop",
                "usage": {
                    "prompt_tokens": 0,  # Gemini doesn't provide detailed token counts
                    "completion_tokens": 0,
                    "total_tokens": 0
                }
            }
            
        except Exception as e:
            logger.error(f"Gemini API error: {str(e)}")
            raise
    
    def get_provider_name(self) -> str:
        return "gemini"

class LLMProviderFactory:
    """Factory for creating LLM provider instances"""
    
    @staticmethod
    def create_provider(provider_name: str, api_key: str = None, model: str = None) -> LLMProvider:
        """
        Create an LLM provider instance
        
        Args:
            provider_name: Name of the provider (openai, anthropic, gemini)
            api_key: Optional API key (will use env vars if not provided)
            model: Optional model name (will use defaults if not provided)
            
        Returns:
            LLMProvider instance
        """
        provider_name = provider_name.lower()
        
        if provider_name == "openai":
            default_model = model or "gpt-4"
            return OpenAIProvider(api_key, default_model)
        elif provider_name == "anthropic":
            default_model = model or "claude-3-5-sonnet-20241022"
            return AnthropicProvider(api_key, default_model)
        elif provider_name == "gemini":
            default_model = model or "gemini-2.5-flash"
            return GeminiProvider(api_key, default_model)
        else:
            raise ValueError(f"Unsupported provider: {provider_name}")
    
    @staticmethod
    def get_available_providers() -> List[str]:
        """Get list of available provider names"""
        return ["openai", "anthropic", "gemini"]
    
    @staticmethod
    def get_default_models() -> Dict[str, str]:
        """Get default models for each provider"""
        return {
            "openai": "gpt-4",
            "anthropic": "claude-3-5-sonnet-20241022",
            "gemini": "gemini-2.5-flash"
        }
