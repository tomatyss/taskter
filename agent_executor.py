"""
Agent execution engine for running AI agents with LLM providers and tools
"""
import json
import time
import logging
from typing import Dict, List, Any
from dotenv import load_dotenv
from models import Agent, Task, AgentExecution, utcnow
from llm_providers import LLMProviderFactory
from tools import tool_registry
from db import db

# Load environment variables from .env file
load_dotenv()

logger = logging.getLogger(__name__)

class AgentExecutor:
    """Handles execution of AI agents"""
    
    def __init__(self):
        self.max_iterations = 20
        self.default_timeout = 300  # 5 minutes
    
    def execute_agent_task(self, task_id: int, agent_id: int) -> Dict[str, Any]:
        """
        Execute an agent task
        
        Args:
            task_id: ID of the task to execute
            agent_id: ID of the agent to use
            
        Returns:
            Dict with execution results
        """
        execution = None
        start_time = time.time()
        
        try:
            # Get task and agent
            task = Task.query.get(task_id)
            agent = Agent.query.get(agent_id)
            
            if not task:
                return {"success": False, "error": f"Task {task_id} not found"}
            
            if not agent:
                return {"success": False, "error": f"Agent {agent_id} not found"}
            
            if not agent.is_active:
                return {"success": False, "error": f"Agent {agent_id} is not active"}
            
            # Create execution record
            execution = AgentExecution(
                task_id=task_id,
                agent_id=agent_id,
                status='running',
                started_at=utcnow(),
                conversation_log=[]
            )
            db.session.add(execution)
            db.session.commit()
            
            # Update task status
            task.execution_status = 'running'
            db.session.commit()
            
            logger.info(f"Starting execution {execution.id}: Task {task_id} with Agent {agent_id}")
            
            # Execute the agent
            result = self._run_agent_loop(task, agent, execution)
            
            # Calculate execution time
            execution_time = time.time() - start_time
            
            # Update execution record
            execution.status = 'completed' if result['success'] else 'failed'
            execution.result = result.get('result', '')
            execution.error_message = result.get('error', '')
            execution.execution_time_seconds = execution_time
            execution.completed_at = utcnow()
            execution.tokens_used = result.get('total_tokens', 0)
            
            # Update task status
            if result['success']:
                task.execution_status = 'completed'
                # Optionally move task to 'done' status
                if result.get('task_completed', False):
                    task.status = 'done'
            else:
                task.execution_status = 'failed'
            
            db.session.commit()
            
            logger.info(f"Completed execution {execution.id} in {execution_time:.2f}s")
            
            return result
            
        except Exception as e:
            logger.error(f"Agent execution error: {str(e)}")
            
            # Update execution record on error
            if execution:
                execution.status = 'failed'
                execution.error_message = str(e)
                execution.execution_time_seconds = time.time() - start_time
                execution.completed_at = utcnow()
                db.session.commit()
            
            # Update task status
            if task_id:
                task = Task.query.get(task_id)
                if task:
                    task.execution_status = 'failed'
                    db.session.commit()
            
            return {"success": False, "error": str(e)}
    
    def _run_agent_loop(self, task: Task, agent: Agent, execution: AgentExecution) -> Dict[str, Any]:
        """
        Run the main agent execution loop
        
        Args:
            task: Task to execute
            agent: Agent to use
            execution: Execution record to update
            
        Returns:
            Dict with execution results
        """
        try:
            # Create LLM provider
            provider = LLMProviderFactory.create_provider(
                agent.llm_provider,
                agent.llm_api_key,
                agent.llm_model
            )
            
            # Get agent configuration
            config = agent.config or {}
            max_iterations = config.get('max_iterations', self.max_iterations)
            temperature = config.get('temperature', 0.7)
            max_tokens = config.get('max_tokens', 1000)
            
            # Initialize conversation
            conversation_history = []
            total_tokens = 0
            
            # Create initial task message
            initial_message = {
                "role": "user",
                "content": self._create_task_prompt(task)
            }
            conversation_history.append(initial_message)
            
            # Get available tools
            available_tools = self._get_tools_for_provider(agent.available_tools, agent.llm_provider)
            
            # Main execution loop
            for iteration in range(max_iterations):
                logger.info(f"Execution {execution.id} - Iteration {iteration + 1}/{max_iterations}")
                
                try:
                    # Get response from LLM
                    response = provider.chat(
                        system=agent.system_instructions,
                        messages=conversation_history,
                        tools=available_tools,
                        temperature=temperature,
                        max_tokens=max_tokens
                    )
                    
                    # Track token usage
                    if response.get('usage'):
                        total_tokens += response['usage'].get('total_tokens', 0)
                    
                    # Add assistant response to conversation
                    if response.get('content'):
                        conversation_history.append({
                            "role": "assistant",
                            "content": response['content']
                        })
                    
                    # Handle tool calls
                    if response.get('tool_calls'):
                        tool_results = []
                        for tool_call in response['tool_calls']:
                            tool_result = self._execute_tool_call(tool_call)
                            tool_results.append(tool_result)
                            
                            # Add tool result to conversation
                            conversation_history.append({
                                "role": "tool",
                                "content": json.dumps(tool_result),
                                "tool_call_id": tool_call.get('id', 'unknown')
                            })
                    
                    # Update execution log
                    execution.conversation_log = conversation_history
                    execution.iterations_count = iteration + 1
                    execution.tokens_used = total_tokens
                    db.session.commit()
                    
                    # Check for completion
                    if self._is_task_completed(response, conversation_history):
                        return {
                            "success": True,
                            "result": "Task completed successfully",
                            "conversation": conversation_history,
                            "iterations": iteration + 1,
                            "total_tokens": total_tokens,
                            "task_completed": True
                        }
                    
                    # Check for explicit stop
                    if response.get('finish_reason') == 'stop' and iteration > 0:
                        # Agent decided to stop without explicit completion
                        return {
                            "success": True,
                            "result": "Agent completed execution",
                            "conversation": conversation_history,
                            "iterations": iteration + 1,
                            "total_tokens": total_tokens,
                            "task_completed": False
                        }
                        
                except Exception as e:
                    logger.error(f"Error in iteration {iteration + 1}: {str(e)}")
                    conversation_history.append({
                        "role": "system",
                        "content": f"Error occurred: {str(e)}"
                    })
                    
                    # Continue to next iteration unless it's a critical error
                    if "API" in str(e) or "quota" in str(e).lower():
                        return {
                            "success": False,
                            "error": f"LLM API error: {str(e)}",
                            "conversation": conversation_history,
                            "iterations": iteration + 1,
                            "total_tokens": total_tokens
                        }
            
            # Max iterations reached
            return {
                "success": False,
                "error": f"Maximum iterations ({max_iterations}) reached without completion",
                "conversation": conversation_history,
                "iterations": max_iterations,
                "total_tokens": total_tokens
            }
            
        except Exception as e:
            logger.error(f"Agent loop error: {str(e)}")
            return {
                "success": False,
                "error": str(e),
                "conversation": conversation_history if 'conversation_history' in locals() else [],
                "iterations": 0,
                "total_tokens": total_tokens if 'total_tokens' in locals() else 0
            }
    
    def _create_task_prompt(self, task: Task) -> str:
        """Create the initial task prompt for the agent"""
        prompt = f"""You have been assigned the following task:

Title: {task.title}
Description: {task.description or 'No description provided'}
Current Status: {task.status}

Your goal is to complete this task using the available tools. When you have successfully completed the task, respond with "TASK_COMPLETED" in your message.

Please analyze the task and create a plan to complete it, then execute that plan step by step."""
        
        return prompt
    
    def _get_tools_for_provider(self, tool_names: List[str], provider_name: str) -> List[Dict]:
        """Get tools formatted for the specific LLM provider"""
        if provider_name == "openai":
            return tool_registry.get_tools_openai_format(tool_names)
        elif provider_name == "anthropic":
            return tool_registry.get_tools_anthropic_format(tool_names)
        else:
            # For Gemini and others, use OpenAI format as fallback
            return tool_registry.get_tools_openai_format(tool_names)
    
    def _execute_tool_call(self, tool_call: Dict) -> Dict[str, Any]:
        """Execute a tool call and return the result"""
        try:
            function = tool_call.get('function', {})
            tool_name = function.get('name', '')
            arguments_str = function.get('arguments', '{}')
            
            # Parse arguments
            try:
                arguments = json.loads(arguments_str) if arguments_str else {}
            except json.JSONDecodeError:
                # Handle simple argument format for Gemini
                arguments = self._parse_simple_arguments(arguments_str)
            
            logger.info(f"Executing tool: {tool_name} with args: {arguments}")
            
            # Execute the tool
            result = tool_registry.execute_tool(tool_name, **arguments)
            
            return {
                "tool_name": tool_name,
                "arguments": arguments,
                "result": result
            }
            
        except Exception as e:
            logger.error(f"Tool execution error: {str(e)}")
            return {
                "tool_name": tool_call.get('function', {}).get('name', 'unknown'),
                "arguments": {},
                "result": {"success": False, "error": str(e)}
            }
    
    def _parse_simple_arguments(self, args_str: str) -> Dict[str, Any]:
        """Parse simple argument format like 'query="test", num_results=5'"""
        args = {}
        if not args_str:
            return args
        
        try:
            # Simple parsing for key=value pairs
            pairs = args_str.split(',')
            for pair in pairs:
                if '=' in pair:
                    key, value = pair.split('=', 1)
                    key = key.strip().strip('"\'')
                    value = value.strip().strip('"\'')
                    
                    # Try to convert to appropriate type
                    if value.isdigit():
                        args[key] = int(value)
                    elif value.lower() in ('true', 'false'):
                        args[key] = value.lower() == 'true'
                    else:
                        args[key] = value
        except Exception as e:
            logger.warning(f"Failed to parse arguments '{args_str}': {str(e)}")
        
        return args
    
    def _is_task_completed(self, response: Dict, conversation: List[Dict]) -> bool:
        """Check if the task has been completed based on the response"""
        content = response.get('content', '')
        if not content:
            return False
        
        # Look for completion indicators
        completion_indicators = [
            'TASK_COMPLETED',
            'task completed',
            'task is completed',
            'successfully completed',
            'finished the task'
        ]
        
        content_lower = content.lower()
        return any(indicator.lower() in content_lower for indicator in completion_indicators)
    
    def stop_execution(self, execution_id: int) -> Dict[str, Any]:
        """Stop a running execution"""
        try:
            execution = AgentExecution.query.get(execution_id)
            if not execution:
                return {"success": False, "error": f"Execution {execution_id} not found"}
            
            if execution.status != 'running':
                return {"success": False, "error": f"Execution {execution_id} is not running"}
            
            # Update execution status
            execution.status = 'stopped'
            execution.completed_at = utcnow()
            execution.error_message = "Execution stopped by user"
            
            # Update task status
            task = execution.task
            if task:
                task.execution_status = 'manual'
            
            db.session.commit()
            
            return {"success": True, "result": "Execution stopped successfully"}
            
        except Exception as e:
            logger.error(f"Error stopping execution: {str(e)}")
            return {"success": False, "error": str(e)}

# Global executor instance
agent_executor = AgentExecutor()
