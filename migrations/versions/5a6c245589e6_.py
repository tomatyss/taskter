"""empty message

Revision ID: 5a6c245589e6
Revises: 6d75a4f9b2b3, add_tool_logs_column
Create Date: 2025-07-10 01:16:16.501628

"""
from alembic import op
import sqlalchemy as sa


# revision identifiers, used by Alembic.
revision = '5a6c245589e6'
down_revision = ('6d75a4f9b2b3', 'add_tool_logs_column')
branch_labels = None
depends_on = None


def upgrade():
    pass


def downgrade():
    pass
