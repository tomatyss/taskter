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
- **Validation**: Pydantic for request/response schemas
- **Background Tasks**: Celery with Redis
- **Containerization**: Docker & Docker Compose

## Quick Start

### Using Docker Compose (Recommended)

1. Clone or download the project files
2. Navigate to the project directory:
   ```bash
   cd taskter
   ```

3. Copy the environment file and configure your API keys:
   ```bash
   cp .env.example .env
   ```
   
   Edit `.env` and add your API keys:
   ```bash
   # LLM Provider API Keys (at least one required for AI features)
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

4. Start the application:
   ```bash
   docker-compose up --build
   ```
   
   The system will automatically:
   - Start PostgreSQL and Redis
   - Run database migrations
   - Seed sample data
   - Start the web application and background services

5. Open your browser and go to: http://localhost:5001

6. To stop the application:
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

5. Configure environment variables:
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

6. Run database migrations:
   ```bash
   python manage_migrations.py upgrade
   python manage_migrations.py seed
   ```

7. Run the application:
   ```bash
   python main.py
   ```

## Project Structure

```
taskter/
├── main.py                      # Main Flask application entry point
├── manage_migrations.py         # Database migration management
├── celery_app.py               # Celery configuration
├── requirements.txt            # Python dependencies
├── Dockerfile                  # Docker configuration
├── docker-compose.yml          # Docker Compose configuration
├── .env                        # Environment variables
├── app/                        # Main application package
│   ├── __init__.py
│   ├── api/                    # API layer
│   │   ├── __init__.py
│   │   ├── response.py         # API response utilities
│   │   ├── middleware/         # API middleware
│   │   └── v1/                 # API version 1
│   │       ├── __init__.py
│   │       ├── tasks.py        # Task endpoints
│   │       ├── agents.py       # Agent endpoints
│   │       └── executions.py   # Execution endpoints
│   ├── core/                   # Core application components
│   │   ├── __init__.py
│   │   ├── config.py           # Centralized configuration
│   │   ├── constants.py        # Application constants
│   │   ├── exceptions.py       # Custom exceptions
│   │   └── logging.py          # Logging configuration
│   ├── models/                 # Database models
│   │   ├── __init__.py
│   │   ├── task.py             # Task model
│   │   ├── agent.py            # Agent model
│   │   └── execution.py        # Execution model
│   ├── schemas/                # Pydantic schemas
│   │   ├── __init__.py
│   │   ├── task_schemas.py     # Task request/response schemas
│   │   ├── agent_schemas.py    # Agent request/response schemas
│   │   └── execution_schemas.py # Execution request/response schemas
│   ├── services/               # Business logic layer
│   │   ├── __init__.py
│   │   ├── task_service.py     # Task business logic
│   │   ├── agent_service.py    # Agent business logic
│   │   └── execution_service.py # Execution business logic
│   ├── repositories/           # Data access layer
│   │   ├── __init__.py
│   │   ├── base.py             # Base repository
│   │   ├── task_repository.py  # Task data access
│   │   ├── agent_repository.py # Agent data access
│   │   └── execution_repository.py # Execution data access
│   ├── agents/                 # AI agent system
│   │   ├── __init__.py
│   │   ├── providers/          # LLM providers
│   │   └── tools/              # Agent tools
│   └── utils/                  # Utility functions
│       └── __init__.py
├── migrations/                 # Database migration files
├── templates/                  # HTML templates
│   ├── base.html
│   ├── index.html              # Main kanban board
│   ├── agents.html             # Agent management
│   ├── executions.html         # Execution monitoring
│   └── edit_task.html          # Task editing form
├── static/                     # Static files
│   ├── css/
│   │   └── style.css
│   └── js/
│       └── script.js
├── tests/                      # Test files
│   ├── __init__.py
│   ├── unit/
│   ├── integration/
│   └── fixtures/
└── scripts/                    # Utility scripts
```

## API Endpoints

The application provides a RESTful API with the following endpoints:

### Tasks (`/api/v1/tasks`)
- `GET /api/v1/tasks` - List tasks with filtering and pagination
- `POST /api/v1/tasks` - Create new task
- `GET /api/v1/tasks/{id}` - Get specific task
- `PUT /api/v1/tasks/{id}` - Update task
- `DELETE /api/v1/tasks/{id}` - Delete task
- `PUT /api/v1/tasks/{id}/status` - Update task status
- `POST /api/v1/tasks/{id}/assign` - Assign task to agent
- `POST /api/v1/tasks/{id}/unassign` - Unassign task
- `GET /api/v1/tasks/stats` - Get task statistics

### Agents (`/api/v1/agents`)
- `GET /api/v1/agents` - List agents with filtering
- `POST /api/v1/agents` - Create new agent
- `GET /api/v1/agents/{id}` - Get specific agent
- `PUT /api/v1/agents/{id}` - Update agent
- `DELETE /api/v1/agents/{id}` - Delete agent
- `POST /api/v1/agents/{id}/toggle` - Toggle agent active status
- `GET /api/v1/agents/{id}/tasks` - Get agent's assigned tasks
- `GET /api/v1/agents/stats` - Get agent statistics

### Executions (`/api/v1/executions`)
- `GET /api/v1/executions` - List executions with filtering
- `GET /api/v1/executions/{id}` - Get specific execution
- `POST /api/v1/executions/{id}/cancel` - Cancel execution
- `GET /api/v1/executions/stats` - Get execution statistics
- `GET /api/v1/executions/running` - Get currently running executions

## Usage

### Basic Task Management

#### Adding Tasks
- Click the "Add New Task" button
- Fill in the task title (required) and description (optional)
- Click "Add Task"

#### Moving Tasks
- Use the arrow buttons on each task card to move between columns:
  - From "To Do" → "In Progress"
  - From "In Progress" → "To Do" or "Done"
  - From "Done" → "In Progress"

#### Editing Tasks
- Click the edit icon (pencil) on any task card
- Modify the title and/or description
- Click "Save Changes"

#### Deleting Tasks
- Click the delete icon (trash) on any task card
- Confirm the deletion in the popup dialog

### AI Agent System

#### Setting Up AI Agents

1. **Configure API Keys**: Add your LLM provider API keys to the `.env` file
2. **Initialize Sample Agents**: Run the initialization script to create sample agents:
   ```bash
   # If using Docker
   docker-compose exec web python init_agents.py
   
   # If running locally
   python init_agents.py
   ```

#### Using AI Agents

**Via Web Interface:**
1. Navigate to `/agents` to manage agents
2. Navigate to `/executions` to monitor task executions
3. Assign tasks to agents from the main kanban board

**Via API:**

**Create a new agent:**
```bash
curl -X POST http://localhost:5001/api/v1/agents \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Research Assistant",
    "description": "Specialized in web research and data gathering",
    "system_instructions": "You are a research assistant. Help users find information and summarize findings.",
    "llm_provider": "openai",
    "llm_model": "gpt-4",
    "available_tools": ["web_search"],
    "config": {
      "temperature": 0.7,
      "max_tokens": 1000,
      "max_iterations": 10
    }
  }'
```

**Assign a task to an agent:**
```bash
curl -X POST http://localhost:5001/api/v1/tasks/1/assign \
  -H "Content-Type: application/json" \
  -d '{"agent_id": 1}'
```

**Monitor execution:**
```bash
curl http://localhost:5001/api/v1/executions
```

#### Available Tools

1. **Web Search Tool**: Search the web for information using Google Custom Search or DuckDuckGo
2. **Send Email Tool**: Send emails via SMTP with customizable content
3. **Execute Script Tool**: Run Python scripts safely with timeout and security restrictions

#### LLM Providers

- **OpenAI**: GPT-4, GPT-3.5-turbo models
- **Anthropic**: Claude-3.5-sonnet and other Claude models
- **Google Gemini**: Gemini-2.5-flash and other Gemini models

## Configuration

The application uses a centralized configuration system with environment variables:

### Required Configuration
```bash
# Database
DATABASE_URL=postgresql://kanban_user:kanban_pass@localhost:5432/kanban_db

# Flask
SECRET_KEY=your-secret-key-change-in-production

# Redis (for background tasks)
REDIS_URL=redis://localhost:6379/0
```

### Optional Configuration
```bash
# LLM Provider API Keys (at least one required for AI features)
OPENAI_API_KEY=your-openai-api-key-here
ANTHROPIC_API_KEY=your-anthropic-api-key-here
GEMINI_API_KEY=your-gemini-api-key-here

# Email Tool Configuration
SMTP_SERVER=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=your-app-password

# Web Search Tool Configuration
GOOGLE_SEARCH_API_KEY=your-google-search-api-key
GOOGLE_SEARCH_ENGINE_ID=your-search-engine-id

# Logging
LOG_LEVEL=INFO

# Agent Configuration
AGENT_MAX_ITERATIONS=20
AGENT_DEFAULT_TIMEOUT=300
AGENT_MAX_TOKENS=1000
AGENT_TEMPERATURE=0.7
```

## Database Migration System

Taskter uses a robust database migration system built on Flask-Migrate (Alembic):

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

## Database Schema

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

## Docker Configuration

### Services

1. **Database (db)**: PostgreSQL 15 Alpine with health checks
2. **Redis (redis)**: Redis 7 Alpine for background task queue
3. **Migration (migration)**: Runs database migrations and seeding
4. **Web Application (web)**: Main Flask application
5. **Celery Worker (celery_worker)**: Background task processor
6. **Celery Beat (celery_beat)**: Task scheduler

### Volumes

- `postgres_data`: Persistent storage for PostgreSQL data
- `redis_data`: Persistent storage for Redis data

## Development

### Running in Development Mode

1. Start the services:
   ```bash
   docker-compose up
   ```

2. The application will automatically reload when you make changes to the code

3. Access the application at http://localhost:5001

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

View background task logs:
```bash
docker-compose logs celery_worker
docker-compose logs celery_beat
```

### Adding New Features

The application follows a layered architecture:

1. **API Layer** (`app/api/v1/`): Handle HTTP requests and responses
2. **Service Layer** (`app/services/`): Business logic and validation
3. **Repository Layer** (`app/repositories/`): Data access and database operations
4. **Model Layer** (`app/models/`): Database models and relationships

When adding new features:
1. Define database models in `app/models/`
2. Create Pydantic schemas in `app/schemas/`
3. Implement data access in `app/repositories/`
4. Add business logic in `app/services/`
5. Create API endpoints in `app/api/v1/`

## Production Deployment

For production deployment:

1. Change the `SECRET_KEY` in the environment variables
2. Set `FLASK_ENV=production`
3. Configure proper database credentials
4. Set up SSL certificates
5. Use a reverse proxy (nginx)
6. Configure database backups
7. Set up monitoring and logging

## Sample Data

The application includes sample tasks and agents that are automatically created when the database is initialized:

**Sample Tasks:**
- Setup Development Environment (Done)
- Design Database Schema (Done)
- Implement User Authentication (In Progress)
- Create Kanban Board UI (In Progress)
- Add Task Management (To Do)
- Write Unit Tests (To Do)
- Deploy to Production (To Do)

**Sample Agents:**
- Research Assistant (OpenAI GPT-4)
- Data Analyst (Anthropic Claude)
- General Assistant (Google Gemini)

## Troubleshooting

### Common Issues

1. **Port already in use**: Change the port mapping in `docker-compose.yml`
2. **Database connection failed**: Ensure PostgreSQL is running and credentials are correct
3. **Permission denied**: Check file permissions and Docker daemon status
4. **API key errors**: Verify your LLM provider API keys are correctly set in `.env`

### Reset Database

To reset the database and start fresh:

```bash
docker-compose down -v
docker-compose up --build
```

### Debugging Agent Executions

Check Celery logs for agent execution details:
```bash
docker-compose logs celery_worker
```

Monitor executions via API:
```bash
curl http://localhost:5001/api/v1/executions
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes following the established architecture
4. Test thoroughly
5. Submit a pull request

## License

This project is open source and available under the MIT License.
