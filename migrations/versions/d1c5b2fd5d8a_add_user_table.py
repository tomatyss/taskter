"""Add user table

Revision ID: d1c5b2fd5d8a
Revises: 8b82368f9352
Create Date: 2025-07-05 00:00:00.000000
"""
from alembic import op
import sqlalchemy as sa

revision = 'd1c5b2fd5d8a'
down_revision = '8b82368f9352'
branch_labels = None
depends_on = None


def upgrade():
    op.create_table(
        'user',
        sa.Column('id', sa.Integer(), primary_key=True),
        sa.Column('username', sa.String(length=100), nullable=False),
        sa.Column('password_hash', sa.String(length=255), nullable=False),
        sa.Column('created_at', sa.DateTime(), nullable=True),
        sa.Column('updated_at', sa.DateTime(), nullable=True),
        sa.UniqueConstraint('username')
    )


def downgrade():
    op.drop_table('user')
