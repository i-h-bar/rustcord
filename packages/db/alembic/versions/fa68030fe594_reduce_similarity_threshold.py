"""reduce similarity threshold

Revision ID: fa68030fe594
Revises: 6c6c92fc7e9b
Create Date: 2026-01-23 20:43:33.262850

"""

from typing import Sequence, Union

from alembic import op

# revision identifiers, used by Alembic.
revision: str = "fa68030fe594"
down_revision: Union[str, Sequence[str], None] = "6c6c92fc7e9b"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    """Upgrade schema."""
    op.execute("ALTER DATABASE mtg SET pg_trgm.similarity_threshold = 0.3;")


def downgrade() -> None:
    """Downgrade schema."""
    op.execute("ALTER DATABASE mtg RESET pg_trgm.similarity_threshold;")
