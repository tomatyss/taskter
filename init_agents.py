"""
Script to initialize sample AI agents for demonstration
"""
import os
import sys
from flask import Flask
from db import db
from models import Agent, Tool

def create_app():
    """Create Flask app for database operations"""
    app = Flask(__name__)
    app.config['SECRET_KEY'] = os.environ.get('SECRET_KEY', 'dev-secret-key')
    app.config['SQLALCHEMY_DATABASE_URI'] = os.environ.get('DATABASE_URL', 'postgresql://kanban_user:kanban_pass@localhost:5432/kanban_db')
    app.config['SQLALCHEMY_TRACK_MODIFICATIONS'] = False
    
    db.init_app(app)
    return app

def init_sample_agents():
    """Initialize sample agents"""
    
    # Research Agent
    research_agent = Agent(
        name="Research Assistant",
        description="An AI agent specialized in web research and information gathering",
        system_instructions="""You are a research assistant AI agent. Your role is to help users gather information from the web and provide comprehensive research on various topics.

When assigned a task:
1. Analyze the task to understand what information is needed
2. Use the web_search tool to find relevant information
3. Synthesize the findings into a clear, well-organized response
4. If the task requires sending results via email, use the send_email tool
5. Always cite your sources and provide URLs when possible

Be thorough, accurate, and helpful in your research. When you have completed the task successfully, respond with "TASK_COMPLETED".""",
        llm_provider="openai",
        llm_model="gpt-4",
        available_tools=["web_search", "send_email"],
        config={
            "temperature": 0.3,
            "max_tokens": 1500,
            "max_iterations": 10
        },
        is_active=True
    )
    
    # Data Analysis Agent
    data_agent = Agent(
        name="Data Analyst",
        description="An AI agent specialized in data processing and analysis using Python scripts",
        system_instructions="""You are a data analyst AI agent. Your role is to help users process, analyze, and manipulate data using Python scripts.

When assigned a task:
1. Understand the data processing requirements
2. Write Python code to analyze, transform, or process data
3. Use the execute_script tool to run your analysis
4. Interpret the results and provide insights
5. If needed, send results via email using the send_email tool

You can work with various data formats (CSV, JSON, etc.) and perform statistical analysis, data cleaning, visualization preparation, and more. Always explain your approach and findings clearly.

When you have completed the task successfully, respond with "TASK_COMPLETED".""",
        llm_provider="anthropic",
        llm_model="claude-3-5-sonnet-20241022",
        available_tools=["execute_script", "send_email"],
        config={
            "temperature": 0.2,
            "max_tokens": 2000,
            "max_iterations": 15
        },
        is_active=True
    )
    
    # General Assistant Agent
    general_agent = Agent(
        name="General Assistant",
        description="A versatile AI agent with access to all tools for general task completion",
        system_instructions="""You are a general-purpose AI assistant agent. You have access to multiple tools and can help with a wide variety of tasks including research, data processing, and communication.

When assigned a task:
1. Analyze the task requirements carefully
2. Determine which tools would be most helpful
3. Create a step-by-step plan to complete the task
4. Execute your plan using the available tools
5. Provide clear updates on your progress
6. Ensure the task is completed thoroughly

Available tools:
- web_search: For finding information online
- send_email: For sending email communications
- execute_script: For running Python scripts and data processing

Be proactive, thorough, and helpful. Adapt your approach based on the specific requirements of each task.

When you have completed the task successfully, respond with "TASK_COMPLETED".""",
        llm_provider="gemini",
        llm_model="gemini-2.5-flash",
        available_tools=["web_search", "send_email", "execute_script"],
        config={
            "temperature": 0.5,
            "max_tokens": 1200,
            "max_iterations": 12
        },
        is_active=True
    )
    
    return [research_agent, data_agent, general_agent]

def main():
    """Main function to initialize agents"""
    app = create_app()
    
    with app.app_context():
        try:
            # Create tables if they don't exist
            db.create_all()
            
            # Check if agents already exist
            existing_agents = Agent.query.count()
            if existing_agents > 0:
                print(f"Found {existing_agents} existing agents. Skipping initialization.")
                return
            
            # Create sample agents
            agents = init_sample_agents()
            
            for agent in agents:
                db.session.add(agent)
                print(f"Created agent: {agent.name}")
            
            db.session.commit()
            print(f"\nSuccessfully initialized {len(agents)} sample agents!")
            
            # Print agent details
            print("\nAgent Details:")
            print("-" * 50)
            for agent in agents:
                print(f"ID: {agent.id}")
                print(f"Name: {agent.name}")
                print(f"Provider: {agent.llm_provider}")
                print(f"Model: {agent.llm_model}")
                print(f"Tools: {', '.join(agent.available_tools)}")
                print("-" * 50)
                
        except Exception as e:
            print(f"Error initializing agents: {str(e)}")
            sys.exit(1)

if __name__ == "__main__":
    main()
