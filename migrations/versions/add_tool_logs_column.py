"""Add tool_logs column to agent_execution table

Revision ID: add_tool_logs_column
Revises: 8b82368f9352
Create Date: 2025-07-06 17:50:30.000000

"""
from alembic import op
import sqlalchemy as sa
from sqlalchemy.dialects import postgresql

# revision identifiers, used by Alembic.
revision = 'add_tool_logs_column'
down_revision = '8b82368f9352'
branch_labels = None
depends_on = None


def upgrade():
    # Add tool_logs column to agent_execution table
    op.add_column('agent_execution', sa.Column('tool_logs', sa.JSON(), nullable=True))
    
    # Set default value for existing records
    op.execute("UPDATE agent_execution SET tool_logs = '[]' WHERE tool_logs IS NULL")


def downgrade():
    # Remove tool_logs column
    op.drop_column('agent_execution', 'tool_logs')
