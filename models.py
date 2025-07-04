from datetime import datetime, timezone
from db import db

def utcnow():
    return datetime.now(timezone.utc)

class Task(db.Model):
    id = db.Column(db.Integer, primary_key=True)
    title = db.Column(db.String(200), nullable=False)
    description = db.Column(db.Text)
    status = db.Column(db.String(20), nullable=False, default='todo')  # todo, in_progress, done
    created_at = db.Column(db.DateTime, default=utcnow)
    updated_at = db.Column(db.DateTime, default=utcnow, onupdate=utcnow)
    
    # Agent assignment fields
    assigned_agent_id = db.Column(db.Integer, db.ForeignKey('agent.id'))
    execution_status = db.Column(db.String(20), default='manual')  # manual, assigned, running, completed, failed
    
    # Relationships
    assigned_agent = db.relationship('Agent', backref='assigned_tasks')
    executions = db.relationship('AgentExecution', backref='task', cascade='all, delete-orphan')

    def __repr__(self):
        return f'<Task {self.title}>'

class Agent(db.Model):
    id = db.Column(db.Integer, primary_key=True)
    name = db.Column(db.String(100), nullable=False)
    description = db.Column(db.Text)
    system_instructions = db.Column(db.Text, nullable=False)
    
    # LLM Configuration
    llm_provider = db.Column(db.String(50), nullable=False)  # openai, anthropic, gemini
    llm_model = db.Column(db.String(50), nullable=False)     # gpt-4, claude-3-5-sonnet-20241022, gemini-2.5-flash
    llm_api_key = db.Column(db.String(255))  # Optional - can use env vars instead
    
    # Tool Configuration
    available_tools = db.Column(db.JSON, default=list)  # ["web_search", "send_email", "execute_script"]
    
    # Execution Configuration
    config = db.Column(db.JSON, default=dict)  # {"temperature": 0.7, "max_tokens": 1000, "max_iterations": 10}
    
    is_active = db.Column(db.Boolean, default=True)
    created_at = db.Column(db.DateTime, default=utcnow)
    updated_at = db.Column(db.DateTime, default=utcnow, onupdate=utcnow)
    
    def __repr__(self):
        return f'<Agent {self.name}>'

class AgentExecution(db.Model):
    id = db.Column(db.Integer, primary_key=True)
    task_id = db.Column(db.Integer, db.ForeignKey('task.id'), nullable=False)
    agent_id = db.Column(db.Integer, db.ForeignKey('agent.id'), nullable=False)
    
    status = db.Column(db.String(20), default='pending')  # pending, running, completed, failed, stopped
    conversation_log = db.Column(db.JSON, default=list)  # Full conversation history
    result = db.Column(db.Text)
    error_message = db.Column(db.Text)
    
    # Execution metadata
    iterations_count = db.Column(db.Integer, default=0)
    tokens_used = db.Column(db.Integer, default=0)
    execution_time_seconds = db.Column(db.Float)
    
    started_at = db.Column(db.DateTime)
    completed_at = db.Column(db.DateTime)
    created_at = db.Column(db.DateTime, default=utcnow)
    
    # Relationships
    agent = db.relationship('Agent', backref='executions')
    
    def __repr__(self):
        return f'<AgentExecution {self.id}: Task {self.task_id} by Agent {self.agent_id}>'

class Tool(db.Model):
    id = db.Column(db.Integer, primary_key=True)
    name = db.Column(db.String(50), unique=True, nullable=False)
    display_name = db.Column(db.String(100), nullable=False)
    description = db.Column(db.Text, nullable=False)
    input_schema = db.Column(db.JSON, nullable=False)  # JSON schema for tool inputs
    is_active = db.Column(db.Boolean, default=True)
    created_at = db.Column(db.DateTime, default=utcnow)
    
    def __repr__(self):
        return f'<Tool {self.name}>'
