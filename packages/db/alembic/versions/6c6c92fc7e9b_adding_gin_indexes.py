"""adding_gin_indexes

Revision ID: 6c6c92fc7e9b
Revises: 85fb9545aec4
Create Date: 2025-08-23 17:16:57.765300

"""

from typing import Sequence, Union

from alembic import op

# revision identifiers, used by Alembic.
revision: str = "6c6c92fc7e9b"
down_revision: Union[str, Sequence[str], None] = "85fb9545aec4"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    """Upgrade schema."""
    op.execute("ALTER DATABASE mtg SET pg_trgm.similarity_threshold = 0.5;")
    op.execute("CREATE INDEX idx_gin_card_normalised_name ON card USING gin (normalised_name gin_trgm_ops);")
    op.execute("CREATE INDEX idx_gin_set_normalised_name ON set USING gin (normalised_name gin_trgm_ops);")
    op.execute("CREATE INDEX idx_gin_artist_normalised_name ON artist USING gin (normalised_name gin_trgm_ops);")


def downgrade() -> None:
    """Downgrade schema."""
    op.execute("ALTER DATABASE mtg RESET pg_trgm.similarity_threshold;")
    op.execute("DROP INDEX idx_gin_card_normalised_name;")
    op.execute("DROP INDEX idx_gin_set_normalised_name;")
    op.execute("DROP INDEX idx_gin_artist_normalised_name;")
