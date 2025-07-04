# Database Migration Guide

This guide covers the robust database migration system implemented for the Taskter project using Flask-Migrate.

## Overview

The migration system provides:
- ✅ Version-controlled schema changes
- ✅ Automatic migration generation from model changes
- ✅ Safe rollback capabilities
- ✅ Database validation and backup features
- ✅ Docker integration for automated deployments
- ✅ Sample data seeding

## Quick Start

### 1. Initialize Migration Repository (First Time Only)

```bash
# Initialize the migration repository
python manage_migrations.py init

# Or using Flask-Migrate directly
flask db init
```

### 2. Create Your First Migration

```bash
# Generate migration from current models
python manage_migrations.py migrate "Initial database schema"

# Or using Flask-Migrate directly
flask db migrate -m "Initial database schema"
```

### 3. Apply Migrations

```bash
# Apply all pending migrations
python manage_migrations.py upgrade

# Or using Flask-Migrate directly
flask db upgrade
```

### 4. Seed Sample Data

```bash
# Add sample data to the database
python manage_migrations.py seed
```

## Migration Commands

### Core Migration Commands

| Command | Description | Example |
|---------|-------------|---------|
| `init` | Initialize migration repository | `python manage_migrations.py init` |
| `migrate <message>` | Create new migration | `python manage_migrations.py migrate "Add user table"` |
| `upgrade` | Apply pending migrations | `python manage_migrations.py upgrade` |
| `downgrade [revision]` | Rollback migrations | `python manage_migrations.py downgrade` |
| `current` | Show current revision | `python manage_migrations.py current` |
| `history` | Show migration history | `python manage_migrations.py history` |

### Utility Commands

| Command | Description | Example |
|---------|-------------|---------|
| `validate` | Validate database state | `python manage_migrations.py validate` |
| `seed` | Seed sample data | `python manage_migrations.py seed` |
| `backup` | Create database backup | `python manage_migrations.py backup` |
| `reset` | Reset database (DANGEROUS) | `python manage_migrations.py reset` |

## Flask-Migrate Commands

You can also use Flask-Migrate commands directly:

```bash
# Initialize migration repository
flask db init

# Create new migration
flask db migrate -m "Add new column"

# Apply migrations
flask db upgrade

# Rollback one migration
flask db downgrade

# Show current revision
flask db current

# Show migration history
flask db history
```

## Docker Integration

The migration system is fully integrated with Docker Compose:

### Automatic Migrations on Startup

When you run `docker-compose up`, the system will:

1. Start the PostgreSQL database
2. Run the migration service that:
   - Applies all pending migrations
   - Seeds sample data if the database is empty
3. Start the web application and other services

### Manual Migration in Docker

```bash
# Run migrations manually in Docker
docker-compose run --rm migration python manage_migrations.py upgrade

# Create new migration in Docker
docker-compose run --rm migration python manage_migrations.py migrate "Add new feature"

# Seed data in Docker
docker-compose run --rm migration python manage_migrations.py seed
```

## Development Workflow

### 1. Making Model Changes

1. Modify your models in `models.py`
2. Generate a migration:
   ```bash
   python manage_migrations.py migrate "Describe your changes"
   ```
3. Review the generated migration file in `migrations/versions/`
4. Apply the migration:
   ```bash
   python manage_migrations.py upgrade
   ```

### 2. Working with Branches

When working with Git branches that have different migrations:

```bash
# Switch to a branch with different migrations
git checkout feature-branch

# Apply any new migrations
python manage_migrations.py upgrade

# Switch back to main branch
git checkout main

# Rollback to the common migration point if needed
python manage_migrations.py downgrade <revision_id>

# Apply main branch migrations
python manage_migrations.py upgrade
```

### 3. Production Deployment

1. **Always backup before production migrations:**
   ```bash
   python manage_migrations.py backup
   ```

2. **Test migrations in staging first:**
   ```bash
   # In staging environment
   python manage_migrations.py upgrade
   python manage_migrations.py validate
   ```

3. **Deploy to production:**
   ```bash
   # In production environment
   python manage_migrations.py backup
   python manage_migrations.py upgrade
   python manage_migrations.py validate
   ```

## Migration File Structure

```
migrations/
├── alembic.ini          # Alembic configuration
├── env.py              # Migration environment setup
├── script.py.mako      # Migration template
└── versions/           # Migration files
    ├── 001_initial_schema.py
    ├── 002_add_agents.py
    └── 003_add_indexes.py
```

## Best Practices

### 1. Migration Naming

Use descriptive names for your migrations:

```bash
# Good
python manage_migrations.py migrate "Add user authentication tables"
python manage_migrations.py migrate "Add index on task status"
python manage_migrations.py migrate "Remove deprecated columns"

# Bad
python manage_migrations.py migrate "Update"
python manage_migrations.py migrate "Fix"
```

### 2. Review Generated Migrations

Always review the generated migration files before applying them:

```python
# Example migration file
def upgrade():
    # Check these operations make sense
    op.add_column('task', sa.Column('priority', sa.Integer(), nullable=True))
    op.create_index(op.f('ix_task_priority'), 'task', ['priority'], unique=False)

def downgrade():
    # Ensure rollback operations are correct
    op.drop_index(op.f('ix_task_priority'), table_name='task')
    op.drop_column('task', 'priority')
```

### 3. Data Migrations

For complex data transformations, create separate data migration scripts:

```python
# In a migration file
def upgrade():
    # Schema changes first
    op.add_column('task', sa.Column('new_status', sa.String(20)))
    
    # Data transformation
    connection = op.get_bind()
    connection.execute(
        "UPDATE task SET new_status = 'active' WHERE status = 'in_progress'"
    )
    
    # Remove old column
    op.drop_column('task', 'status')
```

### 4. Testing Migrations

Test your migrations thoroughly:

```bash
# Apply migration
python manage_migrations.py upgrade

# Test your application
python app.py

# Test rollback
python manage_migrations.py downgrade

# Test forward migration again
python manage_migrations.py upgrade
```

## Troubleshooting

### Common Issues

1. **Migration conflicts:**
   ```bash
   # If you have conflicting migrations
   python manage_migrations.py history
   python manage_migrations.py downgrade <common_revision>
   # Resolve conflicts and create new migration
   python manage_migrations.py migrate "Resolve migration conflicts"
   ```

2. **Database out of sync:**
   ```bash
   # Mark current database state as up-to-date
   flask db stamp head
   ```

3. **Failed migration:**
   ```bash
   # Rollback failed migration
   python manage_migrations.py downgrade
   # Fix the migration file
   # Try again
   python manage_migrations.py upgrade
   ```

### Recovery Procedures

1. **Restore from backup:**
   ```bash
   # If you have a backup file
   psql -h localhost -U kanban_user -d kanban_db < backup_taskter_20240101_120000.sql
   ```

2. **Reset database (development only):**
   ```bash
   python manage_migrations.py reset
   ```

## Environment Variables

The migration system uses these environment variables:

```bash
# Database connection
DATABASE_URL=postgresql://kanban_user:kanban_pass@localhost:5432/kanban_db

# Flask configuration
SECRET_KEY=your-secret-key
FLASK_ENV=development
```

## Migration vs. Old System

### What Changed

| Old System | New System |
|------------|------------|
| `migrate_db.py` | `manage_migrations.py` + Flask-Migrate |
| `init.sql` | Migration files + seed data |
| Manual SQL | Automatic schema generation |
| No version control | Full version control |
| No rollback | Safe rollback support |
| No validation | Built-in validation |

### Migration Path

The old files have been removed:
- ❌ `migrate_db.py` - Replaced by `manage_migrations.py`
- ❌ `init.sql` - Replaced by migration files and seed data

## Support

For issues with the migration system:

1. Check the migration logs
2. Validate your database state: `python manage_migrations.py validate`
3. Review the migration files in `migrations/versions/`
4. Check the Flask-Migrate documentation: https://flask-migrate.readthedocs.io/
