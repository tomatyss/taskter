"""Add comments column to task

Revision ID: add_task_comments
Revises: 5a6c245589e6
Create Date: 2025-07-10 02:00:00.000000
"""

from alembic import op
import sqlalchemy as sa
from sqlalchemy.dialects import postgresql

# revision identifiers, used by Alembic.
revision = 'add_task_comments'
down_revision = '5a6c245589e6'
branch_labels = None
depends_on = None


def upgrade():
    """Add comments JSON column"""
    op.add_column('task', sa.Column('comments', postgresql.JSON(), nullable=True))
    op.execute("UPDATE task SET comments = '[]' WHERE comments IS NULL")


def downgrade():
    """Remove comments column"""
    op.drop_column('task', 'comments')

