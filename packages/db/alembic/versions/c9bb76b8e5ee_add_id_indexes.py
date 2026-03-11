"""add_id_indexes

Revision ID: c9bb76b8e5ee
Revises: fa68030fe594
Create Date: 2026-02-27 19:36:52.251434

"""

from typing import Sequence, Union

from alembic import op  # ty:ignore[unresolved-import]

# revision identifiers, used by Alembic.
revision: str = "c9bb76b8e5ee"
down_revision: Union[str, Sequence[str], None] = "fa68030fe594"
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    """Upgrade schema."""
    op.execute("create index ix_card_set_id on card (set_id);")
    op.execute("create index ix_card_oracle_id on card (oracle_id);")


def downgrade() -> None:
    """Downgrade schema."""
    op.execute("drop index ix_card_oracle_id;")
    op.execute("drop index ix_card_set_id;")
