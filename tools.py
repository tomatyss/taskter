"""
Tool framework for AI agents with initial tool implementations
"""
import os
import subprocess
import smtplib
import requests
from abc import ABC, abstractmethod
from typing import Dict, Any, List, Optional
from email.mime.text import MIMEText
from email.mime.multipart import MIMEMultipart
import logging

logger = logging.getLogger(__name__)

class Tool(ABC):
    """Abstract base class for all tools"""
    
    @property
    @abstractmethod
    def name(self) -> str:
        """Tool name"""
        pass
    
    @property
    @abstractmethod
    def description(self) -> str:
        """Tool description"""
        pass
    
    @property
    @abstractmethod
    def input_schema(self) -> Dict:
        """JSON schema for tool inputs"""
        pass
    
    @abstractmethod
    def execute(self, **kwargs) -> Dict[str, Any]:
        """
        Execute the tool with given parameters
        
        Returns:
            Dict with keys: success (bool), result (any), error (str, optional)
        """
        pass
    
    def to_openai_format(self) -> Dict:
        """Convert tool to OpenAI function calling format"""
        return {
            "type": "function",
            "function": {
                "name": self.name,
                "description": self.description,
                "parameters": self.input_schema
            }
        }
    
    def to_anthropic_format(self) -> Dict:
        """Convert tool to Anthropic tool format"""
        return {
            "name": self.name,
            "description": self.description,
            "input_schema": self.input_schema
        }

class WebSearchTool(Tool):
    """Tool for performing web searches"""
    
    def __init__(self):
        self.google_api_key = os.getenv('GOOGLE_SEARCH_API_KEY')
        self.search_engine_id = os.getenv('GOOGLE_SEARCH_ENGINE_ID')
        
        # Fallback to DuckDuckGo if Google not configured
        self.use_google = bool(self.google_api_key and self.search_engine_id)
    
    @property
    def name(self) -> str:
        return "web_search"
    
    @property
    def description(self) -> str:
        return "Search the web for information using search queries. Returns relevant search results with titles, snippets, and URLs."
    
    @property
    def input_schema(self) -> Dict:
        return {
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The search query to execute"
                },
                "num_results": {
                    "type": "integer",
                    "description": "Number of results to return (default: 5, max: 10)",
                    "minimum": 1,
                    "maximum": 10,
                    "default": 5
                }
            },
            "required": ["query"]
        }
    
    def execute(self, **kwargs) -> Dict[str, Any]:
        try:
            query = kwargs.get('query')
            num_results = kwargs.get('num_results', 5)
            
            if not query:
                return {"success": False, "error": "Query parameter is required"}
            
            if self.use_google:
                results = self._google_search(query, num_results)
            else:
                results = self._duckduckgo_search(query, num_results)
            
            return {"success": True, "result": results}
            
        except Exception as e:
            logger.error(f"Web search error: {str(e)}")
            return {"success": False, "error": str(e)}
    
    def _google_search(self, query: str, num_results: int) -> List[Dict]:
        """Perform Google Custom Search"""
        url = "https://www.googleapis.com/customsearch/v1"
        params = {
            'key': self.google_api_key,
            'cx': self.search_engine_id,
            'q': query,
            'num': min(num_results, 10)
        }
        
        response = requests.get(url, params=params)
        response.raise_for_status()
        
        data = response.json()
        results = []
        
        for item in data.get('items', []):
            results.append({
                'title': item.get('title', ''),
                'snippet': item.get('snippet', ''),
                'url': item.get('link', ''),
                'display_url': item.get('displayLink', '')
            })
        
        return results
    
    def _duckduckgo_search(self, query: str, num_results: int) -> List[Dict]:
        """Perform DuckDuckGo search (fallback method)"""
        # Simple DuckDuckGo instant answer API
        url = "https://api.duckduckgo.com/"
        params = {
            'q': query,
            'format': 'json',
            'no_html': '1',
            'skip_disambig': '1'
        }
        
        response = requests.get(url, params=params)
        response.raise_for_status()
        
        data = response.json()
        results = []
        
        # Add abstract if available
        if data.get('Abstract'):
            results.append({
                'title': data.get('Heading', 'DuckDuckGo Result'),
                'snippet': data.get('Abstract', ''),
                'url': data.get('AbstractURL', ''),
                'display_url': data.get('AbstractSource', '')
            })
        
        # Add related topics
        for topic in data.get('RelatedTopics', [])[:num_results-1]:
            if isinstance(topic, dict) and 'Text' in topic:
                results.append({
                    'title': topic.get('Text', '').split(' - ')[0] if ' - ' in topic.get('Text', '') else 'Related',
                    'snippet': topic.get('Text', ''),
                    'url': topic.get('FirstURL', ''),
                    'display_url': 'DuckDuckGo'
                })
        
        return results[:num_results]

class SendEmailTool(Tool):
    """Tool for sending emails"""
    
    def __init__(self):
        self.smtp_server = os.getenv('SMTP_SERVER', 'smtp.gmail.com')
        self.smtp_port = int(os.getenv('SMTP_PORT', '587'))
        self.smtp_username = os.getenv('SMTP_USERNAME')
        self.smtp_password = os.getenv('SMTP_PASSWORD')
        
        if not (self.smtp_username and self.smtp_password):
            logger.warning("SMTP credentials not configured. Email tool will not work.")
    
    @property
    def name(self) -> str:
        return "send_email"
    
    @property
    def description(self) -> str:
        return "Send an email to specified recipients with subject and body content."
    
    @property
    def input_schema(self) -> Dict:
        return {
            "type": "object",
            "properties": {
                "to": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "List of recipient email addresses"
                },
                "subject": {
                    "type": "string",
                    "description": "Email subject line"
                },
                "body": {
                    "type": "string",
                    "description": "Email body content"
                },
                "cc": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "List of CC email addresses (optional)"
                },
                "bcc": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "List of BCC email addresses (optional)"
                }
            },
            "required": ["to", "subject", "body"]
        }
    
    def execute(self, **kwargs) -> Dict[str, Any]:
        try:
            if not (self.smtp_username and self.smtp_password):
                return {"success": False, "error": "SMTP credentials not configured"}
            
            to_emails = kwargs.get('to', [])
            subject = kwargs.get('subject', '')
            body = kwargs.get('body', '')
            cc_emails = kwargs.get('cc', [])
            bcc_emails = kwargs.get('bcc', [])
            
            if not to_emails:
                return {"success": False, "error": "At least one recipient is required"}
            
            # Create message
            msg = MIMEMultipart()
            msg['From'] = self.smtp_username
            msg['To'] = ', '.join(to_emails)
            msg['Subject'] = subject
            
            if cc_emails:
                msg['Cc'] = ', '.join(cc_emails)
            
            # Add body
            msg.attach(MIMEText(body, 'plain'))
            
            # Connect to server and send email
            server = smtplib.SMTP(self.smtp_server, self.smtp_port)
            server.starttls()
            server.login(self.smtp_username, self.smtp_password)
            
            # Send to all recipients
            all_recipients = to_emails + cc_emails + bcc_emails
            text = msg.as_string()
            server.sendmail(self.smtp_username, all_recipients, text)
            server.quit()
            
            return {
                "success": True,
                "result": f"Email sent successfully to {len(all_recipients)} recipients"
            }
            
        except Exception as e:
            logger.error(f"Email sending error: {str(e)}")
            return {"success": False, "error": str(e)}

class ExecuteScriptTool(Tool):
    """Tool for executing Python scripts safely"""
    
    def __init__(self):
        self.timeout = 30  # 30 second timeout
        self.allowed_imports = {
            'json', 'csv', 'datetime', 'time', 'math', 'random', 'os', 'sys',
            'requests', 'urllib', 'base64', 'hashlib', 'uuid', 're',
            'collections', 'itertools', 'functools', 'operator'
        }
    
    @property
    def name(self) -> str:
        return "execute_script"
    
    @property
    def description(self) -> str:
        return "Execute a Python script safely with limited imports and timeout. Returns the script output."
    
    @property
    def input_schema(self) -> Dict:
        return {
            "type": "object",
            "properties": {
                "code": {
                    "type": "string",
                    "description": "Python code to execute"
                },
                "timeout": {
                    "type": "integer",
                    "description": "Execution timeout in seconds (default: 30, max: 60)",
                    "minimum": 1,
                    "maximum": 60,
                    "default": 30
                }
            },
            "required": ["code"]
        }
    
    def execute(self, **kwargs) -> Dict[str, Any]:
        try:
            code = kwargs.get('code', '')
            timeout = kwargs.get('timeout', self.timeout)
            
            if not code:
                return {"success": False, "error": "Code parameter is required"}
            
            # Basic security check - prevent dangerous imports
            dangerous_patterns = [
                'import subprocess', 'import os.system', 'import shutil',
                'import socket', 'import threading', 'import multiprocessing',
                'exec(', 'eval(', '__import__', 'open('
            ]
            
            for pattern in dangerous_patterns:
                if pattern in code:
                    return {
                        "success": False,
                        "error": f"Potentially dangerous code detected: {pattern}"
                    }
            
            # Create a temporary Python file
            import tempfile
            with tempfile.NamedTemporaryFile(mode='w', suffix='.py', delete=False) as f:
                f.write(code)
                temp_file = f.name
            
            try:
                # Execute the script with timeout
                result = subprocess.run(
                    ['python', temp_file],
                    capture_output=True,
                    text=True,
                    timeout=timeout
                )
                
                output = {
                    "stdout": result.stdout,
                    "stderr": result.stderr,
                    "return_code": result.returncode
                }
                
                if result.returncode == 0:
                    return {"success": True, "result": output}
                else:
                    return {
                        "success": False,
                        "error": f"Script execution failed: {result.stderr}",
                        "result": output
                    }
                    
            finally:
                # Clean up temporary file
                os.unlink(temp_file)
                
        except subprocess.TimeoutExpired:
            return {"success": False, "error": f"Script execution timed out after {timeout} seconds"}
        except Exception as e:
            logger.error(f"Script execution error: {str(e)}")
            return {"success": False, "error": str(e)}

class ToolRegistry:
    """Registry for managing available tools"""
    
    def __init__(self):
        self._tools = {}
        self._register_default_tools()
    
    def _register_default_tools(self):
        """Register the default tools"""
        self.register_tool(WebSearchTool())
        self.register_tool(SendEmailTool())
        self.register_tool(ExecuteScriptTool())
    
    def register_tool(self, tool: Tool):
        """Register a new tool"""
        self._tools[tool.name] = tool
        logger.info(f"Registered tool: {tool.name}")
    
    def get_tool(self, name: str) -> Optional[Tool]:
        """Get a tool by name"""
        return self._tools.get(name)
    
    def get_available_tools(self) -> List[str]:
        """Get list of available tool names"""
        return list(self._tools.keys())
    
    def get_tools_for_agent(self, tool_names: List[str]) -> List[Tool]:
        """Get tool instances for an agent"""
        tools = []
        for name in tool_names:
            tool = self.get_tool(name)
            if tool:
                tools.append(tool)
            else:
                logger.warning(f"Tool not found: {name}")
        return tools
    
    def execute_tool(self, tool_name: str, **kwargs) -> Dict[str, Any]:
        """Execute a tool by name"""
        tool = self.get_tool(tool_name)
        if not tool:
            return {"success": False, "error": f"Tool not found: {tool_name}"}
        
        return tool.execute(**kwargs)
    
    def get_tools_openai_format(self, tool_names: List[str]) -> List[Dict]:
        """Get tools in OpenAI function calling format"""
        tools = []
        for name in tool_names:
            tool = self.get_tool(name)
            if tool:
                tools.append(tool.to_openai_format())
        return tools
    
    def get_tools_anthropic_format(self, tool_names: List[str]) -> List[Dict]:
        """Get tools in Anthropic tool format"""
        tools = []
        for name in tool_names:
            tool = self.get_tool(name)
            if tool:
                tools.append(tool.to_anthropic_format())
        return tools

# Global tool registry instance
tool_registry = ToolRegistry()
