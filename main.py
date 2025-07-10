from app.api.v1 import register_blueprints
import sys
from celery_app import execute_agent_task_async
from app.models import Task, Agent, AgentExecution
from flask import Flask, render_template, request, redirect, url_for, flash
from flask_login import LoginManager, login_user, logout_user, login_required, current_user
from flask_migrate import Migrate
from datetime import datetime
import os
import logging
from dotenv import load_dotenv
from db import db

# Setup Flask-Login
login_manager = LoginManager()
# Load environment variables from .env file
load_dotenv()

app = Flask(__name__)
app.config['SECRET_KEY'] = os.environ.get('SECRET_KEY', 'dev-secret-key')
app.config['SQLALCHEMY_DATABASE_URI'] = os.environ.get(
    'DATABASE_URL', 'postgresql://taskter_user:taskter_pass@db:5432/taskter_db')
app.config['SQLALCHEMY_TRACK_MODIFICATIONS'] = False

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

db.init_app(app)
migrate = Migrate(app, db)
login_manager.init_app(app)
login_manager.login_view = 'login'

from app.models import Task, Agent, AgentExecution, User
from celery_app import execute_agent_task_async

# Register API blueprints
sys.path.append(os.path.dirname(os.path.abspath(__file__)))

register_blueprints(app)


@login_manager.user_loader
def load_user(user_id):
    return User.query.get(int(user_id))


@app.route('/login', methods=['GET', 'POST'])
def login():
    if request.method == 'POST':
        username = request.form.get('username')
        password = request.form.get('password')
        user = User.query.filter_by(username=username).first()
        if user and user.check_password(password):
            login_user(user)
            flash('Logged in successfully!', 'success')
            return redirect(url_for('index'))
        flash('Invalid username or password', 'error')
    return render_template('login.html')


@app.route('/logout')
@login_required
def logout():
    logout_user()
    flash('Logged out', 'success')
    return redirect(url_for('login'))


@login_manager.user_loader
def load_user(user_id):
    return User.query.get(int(user_id))

# Routes


@app.route('/')
@login_required
def index():
    todo_tasks = Task.query.filter_by(
        status='todo').order_by(Task.created_at.desc()).all()
    in_progress_tasks = Task.query.filter_by(
        status='in_progress').order_by(Task.created_at.desc()).all()
    blocked_tasks = Task.query.filter_by(
        status='blocked').order_by(Task.created_at.desc()).all()
    done_tasks = Task.query.filter_by(
        status='done').order_by(Task.created_at.desc()).all()

    return render_template('index.html',
                           todo_tasks=todo_tasks,
                           in_progress_tasks=in_progress_tasks,
                           blocked_tasks=blocked_tasks,
                           done_tasks=done_tasks)


@app.route('/add_task', methods=['POST'])
@login_required
def add_task():
    title = request.form.get('title')
    description = request.form.get('description')

    if not title:
        flash('Task title is required!', 'error')
        return redirect(url_for('index'))

    task = Task(title=title, description=description, status='todo')
    db.session.add(task)
    db.session.commit()

    flash('Task added successfully!', 'success')
    return redirect(url_for('index'))


@app.route('/move_task/<int:task_id>/<status>')
@login_required
def move_task(task_id, status):
    if status not in ['todo', 'in_progress', 'blocked', 'done']:
        flash('Invalid status!', 'error')
        return redirect(url_for('index'))

    task = Task.query.get_or_404(task_id)
    task.status = status
    task.updated_at = datetime.utcnow()
    db.session.commit()

    flash(f'Task moved to {status.replace("_", " ").title()}!', 'success')
    return redirect(url_for('index'))


@app.route('/delete_task/<int:task_id>')
@login_required
def delete_task(task_id):
    task = Task.query.get_or_404(task_id)
    db.session.delete(task)
    db.session.commit()

    flash('Task deleted successfully!', 'success')
    return redirect(url_for('index'))


@app.route('/edit_task/<int:task_id>', methods=['GET', 'POST'])
@login_required
def edit_task(task_id):
    task = Task.query.get_or_404(task_id)

    if request.method == 'POST':
        task.title = request.form.get('title')
        task.description = request.form.get('description')
        task.updated_at = datetime.utcnow()
        db.session.commit()

        flash('Task updated successfully!', 'success')
        return redirect(url_for('index'))

    return render_template('edit_task.html', task=task)

# UI Routes for Agent Management


@app.route('/agents')
@login_required
def agents():
    """Agents management page"""
    return render_template('agents.html')


@app.route('/executions')
@login_required
def executions():
    """Agent executions monitoring page"""
    return render_template('executions.html')

# All API routes have been moved to app/api/v1/ controllers
# - Tasks: app/api/v1/tasks.py
# - Agents: app/api/v1/agents.py
# - Executions: app/api/v1/executions.py


if __name__ == '__main__':
    with app.app_context():
        db.create_all()
    app.run(host='0.0.0.0', port=5000, debug=True)
