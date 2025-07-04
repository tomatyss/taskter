"""
LLM Provider abstraction layer supporting OpenAI, Anthropic, and Google Gemini
"""
import os
import json
from abc import ABC, abstractmethod
from typing import Dict, List
import logging

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
        except ImportError:
            raise ImportError("google-genai package is required for Gemini provider")
        
        self.api_key = api_key or os.getenv('GEMINI_API_KEY')
        if self.api_key:
            self.client = genai.Client(api_key=self.api_key)
        else:
            # Try to use environment variable
            self.client = genai.Client()
        
        self.model = model
    
    def chat(self, system: str, messages: List[Dict], tools: List[Dict] = None, **kwargs) -> Dict:
        try:
            # Format content for Gemini
            formatted_content = f"System Instructions: {system}\n\n"
            
            # Add conversation history
            for msg in messages:
                role = msg.get('role', 'user')
                content = msg.get('content', '')
                if content:  # Skip empty messages
                    formatted_content += f"{role.title()}: {content}\n"
            
            # Add tool information if available
            if tools:
                tool_descriptions = []
                for tool in tools:
                    func = tool.get('function', {})
                    name = func.get('name', 'unknown')
                    desc = func.get('description', 'No description')
                    tool_descriptions.append(f"- {name}: {desc}")
                
                formatted_content += f"\nAvailable tools:\n" + "\n".join(tool_descriptions)
                formatted_content += "\n\nIf you need to use a tool, respond with: TOOL_CALL: tool_name(arguments)"
            
            # Make API call
            response = self.client.models.generate_content(
                model=self.model,
                contents=formatted_content
            )
            
            # Parse tool calls from response text (simple parsing)
            tool_calls = []
            content = response.text
            
            if content and "TOOL_CALL:" in content:
                # Simple tool call parsing - in production, you'd want more robust parsing
                lines = content.split('\n')
                for line in lines:
                    if line.strip().startswith('TOOL_CALL:'):
                        tool_part = line.replace('TOOL_CALL:', '').strip()
                        if '(' in tool_part and ')' in tool_part:
                            tool_name = tool_part.split('(')[0].strip()
                            args_part = tool_part.split('(', 1)[1].rsplit(')', 1)[0]
                            tool_calls.append({
                                "id": f"call_{len(tool_calls)}",
                                "type": "function",
                                "function": {
                                    "name": tool_name,
                                    "arguments": args_part
                                }
                            })
            
            return {
                "content": content,
                "tool_calls": tool_calls,
                "finish_reason": "stop",
                "usage": {
                    "prompt_tokens": 0,  # Gemini doesn't provide token counts in the same way
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
