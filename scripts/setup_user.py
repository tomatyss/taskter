#!/usr/bin/env python3
"""Create an initial user for the Taskter application."""
import os
import sys
import getpass
from flask import Flask

# Add the parent directory to the Python path so we can import from the root
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from db import db
from app.models.user import User

app = Flask(__name__)
app.config['SQLALCHEMY_DATABASE_URI'] = os.environ.get('DATABASE_URL', 'postgresql://taskter_user:taskter_pass@db:5432/taskter_db')
app.config['SQLALCHEMY_TRACK_MODIFICATIONS'] = False

# Initialize the database with the Flask app
db.init_app(app)


def main():
    with app.app_context():
        db.create_all()
        username = input("Username: ").strip()
        if not username:
            print("Username is required")
            return
        if User.query.filter_by(username=username).first():
            print("User already exists")
            return
        password = getpass.getpass("Password: ")
        if not password:
            print("Password is required")
            return
        user = User(username=username)
        user.set_password(password)
        db.session.add(user)
        db.session.commit()
        print(f"User '{username}' created successfully")


if __name__ == '__main__':
    main()
