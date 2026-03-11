import asyncio
from typing import TYPE_CHECKING

from db.queries.tables import combo, related_token

if TYPE_CHECKING:
    from asyncpg import Pool


async def truncate_combos(pool: Pool) -> None:
    await pool.execute(combo.TRUNCATE)


async def truncate_tokens(pool: Pool) -> None:
    await pool.execute(related_token.TRUNCATE)


async def truncate_changeable_tables(pool: Pool) -> None:
    await asyncio.gather(truncate_tokens(pool), truncate_combos(pool))
