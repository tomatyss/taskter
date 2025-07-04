from flask import Flask, render_template, request, redirect, url_for, flash
from flask_sqlalchemy import SQLAlchemy
from datetime import datetime
import os

app = Flask(__name__)
app.config['SECRET_KEY'] = os.environ.get('SECRET_KEY', 'dev-secret-key')
app.config['SQLALCHEMY_DATABASE_URI'] = os.environ.get('DATABASE_URL', 'postgresql://kanban_user:kanban_pass@localhost:5432/kanban_db')
app.config['SQLALCHEMY_TRACK_MODIFICATIONS'] = False

db = SQLAlchemy(app)

# Database Models
class Task(db.Model):
    id = db.Column(db.Integer, primary_key=True)
    title = db.Column(db.String(200), nullable=False)
    description = db.Column(db.Text)
    status = db.Column(db.String(20), nullable=False, default='todo')  # todo, in_progress, done
    created_at = db.Column(db.DateTime, default=datetime.utcnow)
    updated_at = db.Column(db.DateTime, default=datetime.utcnow, onupdate=datetime.utcnow)

    def __repr__(self):
        return f'<Task {self.title}>'

# Routes
@app.route('/')
def index():
    todo_tasks = Task.query.filter_by(status='todo').order_by(Task.created_at.desc()).all()
    in_progress_tasks = Task.query.filter_by(status='in_progress').order_by(Task.created_at.desc()).all()
    done_tasks = Task.query.filter_by(status='done').order_by(Task.created_at.desc()).all()
    
    return render_template('index.html', 
                         todo_tasks=todo_tasks,
                         in_progress_tasks=in_progress_tasks,
                         done_tasks=done_tasks)

@app.route('/add_task', methods=['POST'])
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
def move_task(task_id, status):
    if status not in ['todo', 'in_progress', 'done']:
        flash('Invalid status!', 'error')
        return redirect(url_for('index'))
    
    task = Task.query.get_or_404(task_id)
    task.status = status
    task.updated_at = datetime.utcnow()
    db.session.commit()
    
    flash(f'Task moved to {status.replace("_", " ").title()}!', 'success')
    return redirect(url_for('index'))

@app.route('/delete_task/<int:task_id>')
def delete_task(task_id):
    task = Task.query.get_or_404(task_id)
    db.session.delete(task)
    db.session.commit()
    
    flash('Task deleted successfully!', 'success')
    return redirect(url_for('index'))

@app.route('/edit_task/<int:task_id>', methods=['GET', 'POST'])
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

if __name__ == '__main__':
    with app.app_context():
        db.create_all()
    app.run(host='0.0.0.0', port=5000, debug=True)
