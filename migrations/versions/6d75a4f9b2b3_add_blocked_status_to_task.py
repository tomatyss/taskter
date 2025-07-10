"""Add blocked status to task

Revision ID: 6d75a4f9b2b3
Revises: 8b82368f9352
Create Date: 2025-07-06 20:52:33.000000
"""
from alembic import op
import sqlalchemy as sa

# revision identifiers, used by Alembic.
revision = '6d75a4f9b2b3'
down_revision = '8b82368f9352'
branch_labels = None
depends_on = None


def upgrade():
    """Add check constraint for new blocked status"""
    op.execute(
        """
        ALTER TABLE task
        ADD CONSTRAINT task_status_check
        CHECK (status IN ('todo','in_progress','blocked','done'))
        """
    )


def downgrade():
    """Remove blocked status constraint"""
    op.execute("ALTER TABLE task DROP CONSTRAINT IF EXISTS task_status_check")

