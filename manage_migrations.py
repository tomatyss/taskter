#!/usr/bin/env python3
"""
Enhanced database migration management script for Taskter
Provides additional migration utilities beyond Flask-Migrate
"""
import os
import sys
import json
import subprocess
from datetime import datetime
from flask import Flask
from flask_migrate import Migrate, upgrade, downgrade, current, history, init, migrate as create_migration
from db import db
from models import Task, Agent, AgentExecution, Tool

def create_app():
    """Create Flask app for migration operations"""
    app = Flask(__name__)
    app.config['SECRET_KEY'] = os.environ.get('SECRET_KEY', 'dev-secret-key')
    app.config['SQLALCHEMY_DATABASE_URI'] = os.environ.get('DATABASE_URL', 'postgresql://kanban_user:kanban_pass@localhost:5432/kanban_db')
    app.config['SQLALCHEMY_TRACK_MODIFICATIONS'] = False
    
    db.init_app(app)
    migrate = Migrate(app, db)
    return app, migrate

def run_command(command, description):
    """Run a shell command and handle errors"""
    print(f"\n🔄 {description}...")
    try:
        result = subprocess.run(command, shell=True, check=True, capture_output=True, text=True)
        if result.stdout:
            print(result.stdout)
        print(f"✅ {description} completed successfully")
        return True
    except subprocess.CalledProcessError as e:
        print(f"❌ {description} failed:")
        print(f"Error: {e.stderr}")
        return False

def backup_database():
    """Create a database backup before major migrations"""
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    backup_file = f"backup_taskter_{timestamp}.sql"
    
    db_url = os.environ.get('DATABASE_URL', 'postgresql://kanban_user:kanban_pass@localhost:5432/kanban_db')
    
    # Extract connection details from DATABASE_URL
    # Format: postgresql://user:pass@host:port/dbname
    if db_url.startswith('postgresql://'):
        parts = db_url.replace('postgresql://', '').split('/')
        db_name = parts[1] if len(parts) > 1 else 'kanban_db'
        user_host = parts[0].split('@')
        if len(user_host) == 2:
            user_pass = user_host[0].split(':')
            user = user_pass[0] if len(user_pass) > 0 else 'kanban_user'
            host_port = user_host[1].split(':')
            host = host_port[0] if len(host_port) > 0 else 'localhost'
            
            backup_cmd = f"pg_dump -h {host} -U {user} -d {db_name} > {backup_file}"
            if run_command(backup_cmd, f"Creating database backup: {backup_file}"):
                print(f"📁 Backup saved as: {backup_file}")
                return backup_file
    
    print("⚠️  Could not create database backup - proceeding without backup")
    return None

def seed_sample_data():
    """Seed the database with sample data"""
    app, _ = create_app()
    
    with app.app_context():
        try:
            # Check if we already have data
            if Task.query.count() > 0:
                print("📊 Database already contains data, skipping sample data seeding")
                return True
            
            print("🌱 Seeding sample data...")
            
            # Create sample tasks
            sample_tasks = [
                {
                    'title': 'Setup Development Environment',
                    'description': 'Install Docker, Python, and other development tools',
                    'status': 'done'
                },
                {
                    'title': 'Design Database Schema',
                    'description': 'Create the database tables and relationships',
                    'status': 'done'
                },
                {
                    'title': 'Implement User Authentication',
                    'description': 'Add login and registration functionality',
                    'status': 'in_progress'
                },
                {
                    'title': 'Create Kanban Board UI',
                    'description': 'Design and implement the drag-and-drop interface',
                    'status': 'in_progress'
                },
                {
                    'title': 'Add Task Management',
                    'description': 'Implement CRUD operations for tasks',
                    'status': 'todo'
                },
                {
                    'title': 'Write Unit Tests',
                    'description': 'Create comprehensive test suite',
                    'status': 'todo'
                },
                {
                    'title': 'Deploy to Production',
                    'description': 'Setup CI/CD pipeline and deploy',
                    'status': 'todo'
                }
            ]
            
            for task_data in sample_tasks:
                task = Task(**task_data)
                db.session.add(task)
            
            # Create sample agent
            sample_agent = Agent(
                name='General Assistant',
                description='A general-purpose AI assistant for task automation',
                system_instructions='You are a helpful AI assistant that can help with various tasks. Be concise and accurate in your responses.',
                llm_provider='openai',
                llm_model='gpt-4',
                available_tools=['web_search', 'send_email'],
                config={'temperature': 0.7, 'max_tokens': 1000},
                is_active=True
            )
            db.session.add(sample_agent)
            
            # Create sample tools
            sample_tools = [
                {
                    'name': 'web_search',
                    'display_name': 'Web Search',
                    'description': 'Search the web for information',
                    'input_schema': {
                        'type': 'object',
                        'properties': {
                            'query': {'type': 'string', 'description': 'Search query'}
                        },
                        'required': ['query']
                    }
                },
                {
                    'name': 'send_email',
                    'display_name': 'Send Email',
                    'description': 'Send an email message',
                    'input_schema': {
                        'type': 'object',
                        'properties': {
                            'to': {'type': 'string', 'description': 'Recipient email'},
                            'subject': {'type': 'string', 'description': 'Email subject'},
                            'body': {'type': 'string', 'description': 'Email body'}
                        },
                        'required': ['to', 'subject', 'body']
                    }
                }
            ]
            
            for tool_data in sample_tools:
                tool = Tool(**tool_data)
                db.session.add(tool)
            
            db.session.commit()
            print("✅ Sample data seeded successfully")
            return True
            
        except Exception as e:
            print(f"❌ Error seeding sample data: {str(e)}")
            db.session.rollback()
            return False

def validate_migration():
    """Validate the database state after migration"""
    app, _ = create_app()
    
    with app.app_context():
        try:
            print("🔍 Validating database state...")
            
            # Check if all tables exist
            tables = ['task', 'agent', 'agent_execution', 'tool']
            for table in tables:
                try:
                    result = db.session.execute(f"SELECT 1 FROM {table} LIMIT 1")
                    print(f"✅ Table '{table}' exists and is accessible")
                except Exception as e:
                    print(f"❌ Table '{table}' validation failed: {str(e)}")
                    return False
            
            # Check foreign key relationships
            try:
                # Test Task -> Agent relationship
                db.session.execute("SELECT t.id, a.name FROM task t LEFT JOIN agent a ON t.assigned_agent_id = a.id LIMIT 1")
                print("✅ Task -> Agent foreign key relationship working")
                
                # Test AgentExecution relationships
                db.session.execute("SELECT ae.id, t.title, a.name FROM agent_execution ae LEFT JOIN task t ON ae.task_id = t.id LEFT JOIN agent a ON ae.agent_id = a.id LIMIT 1")
                print("✅ AgentExecution foreign key relationships working")
                
            except Exception as e:
                print(f"❌ Foreign key validation failed: {str(e)}")
                return False
            
            print("✅ Database validation completed successfully")
            return True
            
        except Exception as e:
            print(f"❌ Database validation failed: {str(e)}")
            return False

def main():
    """Main migration management function"""
    if len(sys.argv) < 2:
        print("""
🗃️  Taskter Migration Management

Usage: python manage_migrations.py <command> [options]

Commands:
  init                    Initialize migration repository
  migrate <message>       Create new migration from model changes
  upgrade                 Apply all pending migrations
  downgrade [revision]    Rollback to previous migration
  current                 Show current migration revision
  history                 Show migration history
  validate               Validate database state
  seed                   Seed database with sample data
  backup                 Create database backup
  reset                  Reset database (DANGEROUS - removes all data)
  
Examples:
  python manage_migrations.py init
  python manage_migrations.py migrate "Add user authentication"
  python manage_migrations.py upgrade
  python manage_migrations.py seed
        """)
        return
    
    command = sys.argv[1].lower()
    app, migrate_obj = create_app()
    
    with app.app_context():
        if command == 'init':
            print("🚀 Initializing migration repository...")
            try:
                init()
                print("✅ Migration repository initialized")
            except Exception as e:
                print(f"❌ Initialization failed: {str(e)}")
        
        elif command == 'migrate':
            if len(sys.argv) < 3:
                print("❌ Please provide a migration message")
                return
            message = sys.argv[2]
            print(f"📝 Creating migration: {message}")
            try:
                create_migration(message=message)
                print("✅ Migration created successfully")
            except Exception as e:
                print(f"❌ Migration creation failed: {str(e)}")
        
        elif command == 'upgrade':
            print("⬆️  Applying migrations...")
            try:
                upgrade()
                print("✅ Migrations applied successfully")
                validate_migration()
            except Exception as e:
                print(f"❌ Migration upgrade failed: {str(e)}")
        
        elif command == 'downgrade':
            revision = sys.argv[2] if len(sys.argv) > 2 else '-1'
            print(f"⬇️  Rolling back to revision: {revision}")
            try:
                downgrade(revision=revision)
                print("✅ Rollback completed successfully")
                validate_migration()
            except Exception as e:
                print(f"❌ Rollback failed: {str(e)}")
        
        elif command == 'current':
            print("📍 Current migration revision:")
            try:
                current()
            except Exception as e:
                print(f"❌ Could not get current revision: {str(e)}")
        
        elif command == 'history':
            print("📚 Migration history:")
            try:
                history()
            except Exception as e:
                print(f"❌ Could not get migration history: {str(e)}")
        
        elif command == 'validate':
            validate_migration()
        
        elif command == 'seed':
            seed_sample_data()
        
        elif command == 'backup':
            backup_database()
        
        elif command == 'reset':
            print("⚠️  WARNING: This will delete ALL data in the database!")
            confirm = input("Type 'CONFIRM' to proceed: ")
            if confirm == 'CONFIRM':
                try:
                    db.drop_all()
                    db.create_all()
                    print("✅ Database reset completed")
                    seed_sample_data()
                except Exception as e:
                    print(f"❌ Database reset failed: {str(e)}")
            else:
                print("❌ Reset cancelled")
        
        else:
            print(f"❌ Unknown command: {command}")

if __name__ == "__main__":
    main()
