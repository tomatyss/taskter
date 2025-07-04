# Taskter - AI-Powered Kanban Board Application

An intelligent kanban board application built with Flask, PostgreSQL, and AI agents. The application combines traditional task management with AI automation, allowing you to assign tasks to AI agents that can execute them using various tools and LLM providers.

## Features

### Core Kanban Features
- **Three-column Kanban Board**: To Do, In Progress, Done
- **Task Management**: Create, edit, delete, and move tasks between columns
- **Server-driven UI**: All interactions handled server-side with full page refreshes
- **PostgreSQL Database**: Persistent data storage
- **Docker Support**: Easy deployment and development setup
- **Responsive Design**: Works on desktop and mobile devices

### AI Agent Features
- **Multi-LLM Support**: OpenAI GPT, Anthropic Claude, and Google Gemini
- **Agent Management**: Create, configure, and manage AI agents with custom instructions
- **Task Assignment**: Assign tasks to AI agents for automated execution
- **Tool Integration**: Agents can use web search, email, and script execution tools
- **Background Processing**: Asynchronous task execution with Celery and Redis
- **Execution Monitoring**: Real-time tracking of agent task execution
- **Conversation Logs**: Detailed logs of agent decision-making processes

## Technology Stack

- **Backend**: Flask (Python)
- **Database**: PostgreSQL
- **Frontend**: HTML, CSS, Bootstrap 5, JavaScript
- **Containerization**: Docker & Docker Compose

## Quick Start

### Using Docker Compose (Recommended)

1. Clone or download the project files
2. Navigate to the project directory:
   ```bash
   cd taskter
   ```

3. Start the application:
   ```bash
   docker-compose up --build
   ```
   
   The system will automatically:
   - Start PostgreSQL and Redis
   - Run database migrations
   - Seed sample data
   - Start the web application and background services

4. Open your browser and go to: http://localhost:5001

5. To stop the application:
   ```bash
   docker-compose down
   ```

### Manual Setup (Development)

1. Install Python 3.11+ and PostgreSQL
2. Create a virtual environment:
   ```bash
   python -m venv venv
   source venv/bin/activate  # On Windows: venv\Scripts\activate
   ```

3. Install dependencies:
   ```bash
   pip install -r requirements.txt
   ```

4. Setup PostgreSQL database:
   - Create a database named `kanban_db`
   - Create a user `kanban_user` with password `kanban_pass`
   - Grant all privileges to the user

5. Run the application:
   ```bash
   python app.py
   ```

## Usage

### Adding Tasks
- Click the "Add New Task" button
- Fill in the task title (required) and description (optional)
- Click "Add Task"

### Moving Tasks
- Use the arrow buttons on each task card to move between columns:
  - From "To Do" → "In Progress"
  - From "In Progress" → "To Do" or "Done"
  - From "Done" → "In Progress"

### Editing Tasks
- Click the edit icon (pencil) on any task card
- Modify the title and/or description
- Click "Save Changes"

### Deleting Tasks
- Click the delete icon (trash) on any task card
- Confirm the deletion in the popup dialog

## Project Structure

```
taskter/
├── app.py                    # Main Flask application
├── manage_migrations.py      # Database migration management
├── models.py                 # Database models
├── db.py                     # Database configuration
├── requirements.txt          # Python dependencies
├── Dockerfile               # Docker configuration
├── docker-compose.yml       # Docker Compose configuration
├── MIGRATION_GUIDE.md       # Migration system documentation
├── .env                     # Environment variables
├── .dockerignore            # Docker ignore file
├── migrations/              # Database migration files (auto-generated)
├── templates/               # HTML templates
│   ├── base.html            # Base template
│   ├── index.html           # Main kanban board
│   └── edit_task.html       # Task editing form
└── static/                  # Static files
    ├── css/
    │   └── style.css        # Custom styles
    └── js/
        └── script.js        # JavaScript functionality
```

## Database Migration System

Taskter uses a robust database migration system built on Flask-Migrate (Alembic) that provides:

- ✅ **Version-controlled schema changes**
- ✅ **Automatic migration generation from model changes**
- ✅ **Safe rollback capabilities**
- ✅ **Database validation and backup features**
- ✅ **Docker integration for automated deployments**

### Quick Migration Commands

```bash
# Initialize migration repository (first time only)
python manage_migrations.py init

# Create new migration from model changes
python manage_migrations.py migrate "Add new feature"

# Apply all pending migrations
python manage_migrations.py upgrade

# Seed sample data
python manage_migrations.py seed

# Validate database state
python manage_migrations.py validate

# Create database backup
python manage_migrations.py backup
```

### Docker Integration

Migrations run automatically when using Docker Compose:

```bash
# Start with automatic migrations
docker-compose up --build

# Manual migration in Docker
docker-compose run --rm migration python manage_migrations.py upgrade
```

For detailed migration documentation, see [MIGRATION_GUIDE.md](MIGRATION_GUIDE.md).

## Database Schema

The application uses the following database schema:

### Task Table
- `id`: Primary key (auto-increment)
- `title`: Task title (required, max 200 characters)
- `description`: Task description (optional, text)
- `status`: Task status ('todo', 'in_progress', 'done')
- `created_at`: Creation timestamp
- `updated_at`: Last update timestamp
- `assigned_agent_id`: Foreign key to Agent table (optional)
- `execution_status`: Execution status ('manual', 'assigned', 'running', 'completed', 'failed')

### Agent Table
- `id`: Primary key (auto-increment)
- `name`: Agent name (required, max 100 characters)
- `description`: Agent description (optional, text)
- `system_instructions`: System instructions for the agent (required, text)
- `llm_provider`: LLM provider ('openai', 'anthropic', 'gemini')
- `llm_model`: LLM model name
- `llm_api_key`: API key for the LLM provider (optional)
- `available_tools`: JSON array of available tools
- `config`: JSON configuration object
- `is_active`: Boolean flag for agent status
- `created_at`: Creation timestamp
- `updated_at`: Last update timestamp

### AgentExecution Table
- `id`: Primary key (auto-increment)
- `task_id`: Foreign key to Task table
- `agent_id`: Foreign key to Agent table
- `status`: Execution status ('pending', 'running', 'completed', 'failed', 'stopped')
- `conversation_log`: JSON array of conversation history
- `result`: Execution result (text)
- `error_message`: Error message if execution failed
- `iterations_count`: Number of iterations performed
- `tokens_used`: Number of tokens consumed
- `execution_time_seconds`: Execution time in seconds
- `started_at`: Execution start timestamp
- `completed_at`: Execution completion timestamp
- `created_at`: Creation timestamp

### Tool Table
- `id`: Primary key (auto-increment)
- `name`: Tool name (unique, required)
- `display_name`: Human-readable tool name
- `description`: Tool description
- `input_schema`: JSON schema for tool inputs
- `is_active`: Boolean flag for tool status
- `created_at`: Creation timestamp

## Environment Variables

The application uses the following environment variables:

- `DATABASE_URL`: PostgreSQL connection string
- `SECRET_KEY`: Flask secret key for sessions
- `FLASK_ENV`: Flask environment (development/production)
- `FLASK_DEBUG`: Enable/disable debug mode

## Docker Configuration

### Services

1. **Database (db)**:
   - PostgreSQL 15 Alpine
   - Port: 5432
   - Volume: `postgres_data` for data persistence
   - Health check included

2. **Web Application (web)**:
   - Built from local Dockerfile
   - Port: 5000
   - Depends on database service
   - Volume mount for development

### Volumes

- `postgres_data`: Persistent storage for PostgreSQL data

## Development

### Running in Development Mode

1. Start the services:
   ```bash
   docker-compose up
   ```

2. The application will automatically reload when you make changes to the code

3. Access the application at http://localhost:5000

### Database Access

To access the PostgreSQL database directly:

```bash
docker exec -it kanban_postgres psql -U kanban_user -d kanban_db
```

### Logs

View application logs:
```bash
docker-compose logs web
```

View database logs:
```bash
docker-compose logs db
```

## Production Deployment

For production deployment:

1. Change the `SECRET_KEY` in the environment variables
2. Set `FLASK_ENV=production`
3. Consider using a reverse proxy (nginx)
4. Set up proper SSL certificates
5. Configure database backups
6. Monitor application logs

## AI Agent System

### Overview

The AI agent system allows you to create intelligent agents that can automatically execute tasks using various tools and LLM providers. Each agent has:

- **Custom System Instructions**: Define the agent's role and behavior
- **LLM Provider**: Choose from OpenAI, Anthropic, or Google Gemini
- **Available Tools**: Select which tools the agent can use
- **Configuration**: Set temperature, max tokens, and iteration limits

### Available Tools

1. **Web Search Tool**: Search the web for information using Google Custom Search or DuckDuckGo
2. **Send Email Tool**: Send emails via SMTP with customizable content
3. **Execute Script Tool**: Run Python scripts safely with timeout and security restrictions

### LLM Providers

- **OpenAI**: GPT-4, GPT-3.5-turbo models
- **Anthropic**: Claude-3.5-sonnet and other Claude models
- **Google Gemini**: Gemini-2.5-flash and other Gemini models

### Setting Up AI Agents

#### 1. Configure API Keys

Copy `.env.example` to `.env` and add your API keys:

```bash
cp .env.example .env
```

Edit `.env` and add your keys:
```bash
OPENAI_API_KEY=your-openai-api-key-here
ANTHROPIC_API_KEY=your-anthropic-api-key-here
GEMINI_API_KEY=your-gemini-api-key-here

# Optional: Email configuration for send_email tool
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=your-app-password

# Optional: Google Search configuration for web_search tool
GOOGLE_SEARCH_API_KEY=your-google-search-api-key
GOOGLE_SEARCH_ENGINE_ID=your-search-engine-id
```

#### 2. Initialize Sample Agents

Run the initialization script to create sample agents:

```bash
# If using Docker
docker-compose exec web python init_agents.py

# If running locally
python init_agents.py
```

This creates three sample agents:
- **Research Assistant**: Specialized in web research using OpenAI GPT-4
- **Data Analyst**: Focused on data processing using Anthropic Claude
- **General Assistant**: Versatile agent with all tools using Google Gemini

### Using AI Agents

#### Via API

**List all agents:**
```bash
curl http://localhost:5001/api/agents
```

**Create a new agent:**
```bash
curl -X POST http://localhost:5001/api/agents \
  -H "Content-Type: application/json" \
  -d '{
    "name": "My Custom Agent",
    "description": "A custom agent for specific tasks",
    "system_instructions": "You are a helpful assistant...",
    "llm_provider": "openai",
    "llm_model": "gpt-4",
    "available_tools": ["web_search", "send_email"],
    "config": {
      "temperature": 0.7,
      "max_tokens": 1000,
      "max_iterations": 10
    }
  }'
```

**Assign a task to an agent:**
```bash
curl -X POST http://localhost:5001/api/tasks/1/assign/1
```

**Monitor execution:**
```bash
curl http://localhost:5001/api/executions
```

#### Background Processing

The system uses Celery with Redis for background task processing:

- **Celery Worker**: Executes agent tasks asynchronously
- **Celery Beat**: Scheduler that checks for pending tasks every 30 seconds
- **Redis**: Message broker and result backend

### Agent Execution Flow

1. **Task Assignment**: Assign a task to an agent via API or UI
2. **Scheduler Check**: Celery Beat finds pending tasks every 30 seconds
3. **Agent Execution**: Celery Worker starts agent execution
4. **LLM Loop**: Agent makes decisions and uses tools iteratively
5. **Completion**: Task marked as completed or failed with detailed logs

### Monitoring and Debugging

**View execution logs:**
```bash
# Celery worker logs
docker-compose logs celery_worker

# Celery beat logs
docker-compose logs celery_beat

# Web application logs
docker-compose logs web
```

**Check execution details:**
```bash
curl http://localhost:5001/api/executions/1
```

### Security Considerations

- **Script Execution**: Python scripts run with timeouts and restricted imports
- **API Keys**: Store securely in environment variables
- **Tool Access**: Agents only have access to explicitly assigned tools
- **Sandboxing**: Script execution is isolated with security checks

### Extending the System

#### Adding New Tools

1. Create a new tool class in `tools.py`:
```python
class MyCustomTool(Tool):
    @property
    def name(self) -> str:
        return "my_custom_tool"
    
    # Implement other required methods...
```

2. Register the tool in `ToolRegistry`
3. Update agent configurations to include the new tool

#### Adding New LLM Providers

1. Create a new provider class in `llm_providers.py`:
```python
class MyLLMProvider(LLMProvider):
    def chat(self, system, messages, tools=None, **kwargs):
        # Implement LLM integration
        pass
```

2. Update `LLMProviderFactory` to include the new provider

## Sample Data

The application includes sample tasks that are automatically created when the database is initialized:

- Setup Development Environment (Done)
- Design Database Schema (Done)
- Implement User Authentication (In Progress)
- Create Kanban Board UI (In Progress)
- Add Task Management (To Do)
- Write Unit Tests (To Do)
- Deploy to Production (To Do)

## Troubleshooting

### Common Issues

1. **Port already in use**: Change the port mapping in `docker-compose.yml`
2. **Database connection failed**: Ensure PostgreSQL is running and credentials are correct
3. **Permission denied**: Check file permissions and Docker daemon status

### Reset Database

To reset the database and start fresh:

```bash
docker-compose down -v
docker-compose up --build
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test thoroughly
5. Submit a pull request

## License

This project is open source and available under the MIT License.
