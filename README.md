# Kanban Board Application

A simple kanban board application built with Flask, PostgreSQL, and server-driven UI. The application is containerized using Docker and Docker Compose.

## Features

- **Three-column Kanban Board**: To Do, In Progress, Done
- **Task Management**: Create, edit, delete, and move tasks between columns
- **Server-driven UI**: All interactions handled server-side with full page refreshes
- **PostgreSQL Database**: Persistent data storage
- **Docker Support**: Easy deployment and development setup
- **Responsive Design**: Works on desktop and mobile devices

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
   cd kanban-app
   ```

3. Start the application:
   ```bash
   docker-compose up --build
   ```

4. Open your browser and go to: http://localhost:5000

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
kanban-app/
├── app.py                 # Main Flask application
├── requirements.txt       # Python dependencies
├── Dockerfile            # Docker configuration
├── docker-compose.yml    # Docker Compose configuration
├── init.sql              # Database initialization script
├── .env                  # Environment variables
├── .dockerignore         # Docker ignore file
├── templates/            # HTML templates
│   ├── base.html         # Base template
│   ├── index.html        # Main kanban board
│   └── edit_task.html    # Task editing form
└── static/               # Static files
    ├── css/
    │   └── style.css     # Custom styles
    └── js/
        └── script.js     # JavaScript functionality
```

## Database Schema

The application uses a simple database schema with one main table:

### Task Table
- `id`: Primary key (auto-increment)
- `title`: Task title (required, max 200 characters)
- `description`: Task description (optional, text)
- `status`: Task status ('todo', 'in_progress', 'done')
- `created_at`: Creation timestamp
- `updated_at`: Last update timestamp

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
