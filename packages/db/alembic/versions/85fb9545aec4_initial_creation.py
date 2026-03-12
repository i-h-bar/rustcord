"""initial_creation

Revision ID: 85fb9545aec4
Revises:
Create Date: 2025-08-21 19:06:49.316070

"""

from typing import Sequence, Union

import sqlalchemy as sa
from sqlalchemy.dialects import postgresql

from alembic import op

# revision identifiers, used by Alembic.
revision: str = "85fb9545aec4"
down_revision: Union[str, Sequence[str], None] = None
branch_labels: Union[str, Sequence[str], None] = None
depends_on: Union[str, Sequence[str], None] = None


def upgrade() -> None:
    op.execute("CREATE EXTENSION IF NOT EXISTS pg_trgm;")

    op.create_table(
        "artist",
        sa.Column("id", postgresql.UUID(as_uuid=True), primary_key=True),
        sa.Column("name", sa.Text(), nullable=True),
        sa.Column("normalised_name", sa.Text(), nullable=True),
    )
    op.create_table(
        "image",
        sa.Column("id", postgresql.UUID(as_uuid=True), primary_key=True),
        sa.Column("scryfall_url", sa.Text(), nullable=True),
    )
    op.create_table(
        "illustration",
        sa.Column("id", postgresql.UUID(as_uuid=True), primary_key=True),
        sa.Column("scryfall_url", sa.Text(), nullable=True),
    )
    op.create_table(
        "legality",
        sa.Column("id", postgresql.UUID(as_uuid=True), primary_key=True),
        sa.Column("alchemy", sa.Text(), nullable=True),
        sa.Column("brawl", sa.Text(), nullable=True),
        sa.Column("commander", sa.Text(), nullable=True),
        sa.Column("duel", sa.Text(), nullable=True),
        sa.Column("future", sa.Text(), nullable=True),
        sa.Column("gladiator", sa.Text(), nullable=True),
        sa.Column("historic", sa.Text(), nullable=True),
        sa.Column("legacy", sa.Text(), nullable=True),
        sa.Column("modern", sa.Text(), nullable=True),
        sa.Column("oathbreaker", sa.Text(), nullable=True),
        sa.Column("oldschool", sa.Text(), nullable=True),
        sa.Column("pauper", sa.Text(), nullable=True),
        sa.Column("paupercommander", sa.Text(), nullable=True),
        sa.Column("penny", sa.Text(), nullable=True),
        sa.Column("pioneer", sa.Text(), nullable=True),
        sa.Column("predh", sa.Text(), nullable=True),
        sa.Column("premodern", sa.Text(), nullable=True),
        sa.Column("standard", sa.Text(), nullable=True),
        sa.Column("standardbrawl", sa.Text(), nullable=True),
        sa.Column("timeless", sa.Text(), nullable=True),
        sa.Column("vintage", sa.Text(), nullable=True),
        sa.Column("game_changer", sa.Boolean(), nullable=True),
    )
    op.create_table(
        "rule",
        sa.Column("id", postgresql.UUID(as_uuid=True), primary_key=True),
        sa.Column("colour_identity", sa.ARRAY(sa.CHAR(1)), nullable=True),
        sa.Column("mana_cost", sa.Text(), nullable=True),
        sa.Column("cmc", sa.Integer(), nullable=True),
        sa.Column("power", sa.Text(), nullable=True),
        sa.Column("toughness", sa.Text(), nullable=True),
        sa.Column("loyalty", sa.Text(), nullable=True),
        sa.Column("defence", sa.Text(), nullable=True),
        sa.Column("type_line", sa.Text(), nullable=True),
        sa.Column("oracle_text", sa.Text(), nullable=True),
        sa.Column("colours", sa.ARRAY(sa.CHAR(1)), nullable=True),
        sa.Column("keywords", sa.ARRAY(sa.Text()), nullable=True),
        sa.Column("produced_mana", sa.ARRAY(sa.CHAR(1)), nullable=True),
        sa.Column("rulings_url", sa.Text(), nullable=True),
    )
    op.create_table(
        "set",
        sa.Column("id", postgresql.UUID(as_uuid=True), primary_key=True),
        sa.Column("name", sa.Text(), nullable=True),
        sa.Column("normalised_name", sa.Text(), nullable=True),
        sa.Column("abbreviation", sa.Text(), nullable=True),
    )

    # Create the 'card' table which depends on many other tables
    op.create_table(
        "card",
        sa.Column("id", postgresql.UUID(as_uuid=True), primary_key=True),
        sa.Column("oracle_id", postgresql.UUID(as_uuid=True), nullable=True),
        sa.Column("name", sa.Text(), nullable=True),
        sa.Column("normalised_name", sa.Text(), nullable=True),
        sa.Column("scryfall_url", sa.Text(), nullable=True),
        sa.Column("flavour_text", sa.Text(), nullable=True),
        sa.Column("release_date", sa.Date(), nullable=True),
        sa.Column("reserved", sa.Boolean(), nullable=True),
        sa.Column("rarity", sa.Text(), nullable=True),
        sa.Column("artist_id", postgresql.UUID(as_uuid=True), nullable=True),
        sa.Column("image_id", postgresql.UUID(as_uuid=True), nullable=True),
        sa.Column("illustration_id", postgresql.UUID(as_uuid=True), nullable=True),
        sa.Column("set_id", postgresql.UUID(as_uuid=True), nullable=True),
        sa.Column("backside_id", postgresql.UUID(as_uuid=True), nullable=True),
        sa.ForeignKeyConstraint(["artist_id"], ["artist.id"]),
        sa.ForeignKeyConstraint(["illustration_id"], ["illustration.id"]),
        sa.ForeignKeyConstraint(["image_id"], ["image.id"]),
        sa.ForeignKeyConstraint(["oracle_id"], ["legality.id"]),
        sa.ForeignKeyConstraint(["oracle_id"], ["rule.id"]),
        sa.ForeignKeyConstraint(["set_id"], ["set.id"]),
    )

    op.create_table(
        "combo",
        sa.Column("id", postgresql.UUID(as_uuid=True), primary_key=True),
        sa.Column("card_id", postgresql.UUID(as_uuid=True), nullable=True),
        sa.Column("combo_card_id", postgresql.UUID(as_uuid=True), nullable=True),
        sa.ForeignKeyConstraint(["card_id"], ["card.id"]),
        sa.ForeignKeyConstraint(["combo_card_id"], ["card.id"]),
    )
    op.create_table(
        "price",
        sa.Column("id", postgresql.UUID(as_uuid=True), primary_key=True),
        sa.Column("usd", sa.Numeric(precision=10, scale=2), nullable=True),
        sa.Column("usd_foil", sa.Numeric(precision=10, scale=2), nullable=True),
        sa.Column("usd_etched", sa.Numeric(precision=10, scale=2), nullable=True),
        sa.Column("euro", sa.Numeric(precision=10, scale=2), nullable=True),
        sa.Column("euro_foil", sa.Numeric(precision=10, scale=2), nullable=True),
        sa.Column("tix", sa.Numeric(precision=10, scale=2), nullable=True),
        sa.Column("updated_time", sa.Time(timezone=True), nullable=True),
        sa.ForeignKeyConstraint(["id"], ["card.id"]),
    )
    op.create_table(
        "related_token",
        sa.Column("id", postgresql.UUID(as_uuid=True), primary_key=True),
        sa.Column("card_id", postgresql.UUID(as_uuid=True), nullable=True),
        sa.Column("token_id", postgresql.UUID(as_uuid=True), nullable=True),
        sa.ForeignKeyConstraint(["card_id"], ["card.id"]),
        sa.ForeignKeyConstraint(["token_id"], ["card.id"]),
    )


def downgrade() -> None:
    op.drop_table("related_token")
    op.drop_table("price")
    op.drop_table("combo")
    op.drop_table("card")
    op.drop_table("set")
    op.drop_table("rule")
    op.drop_table("legality")
    op.drop_table("illustration")
    op.drop_table("image")
    op.drop_table("artist")

    op.execute("DROP EXTENSION IF EXISTS pg_trgm;")
