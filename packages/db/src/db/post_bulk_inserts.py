import asyncio
import contextlib
from typing import TYPE_CHECKING

from asyncpg.exceptions import ForeignKeyViolationError
from tqdm import tqdm

from db import queries
from models.post_inserts import token_relations

if TYPE_CHECKING:
    from asyncpg import Pool

    from models.combos import Combo
    from models.related_tokens import RelatedToken


async def insert_relation(related_token: RelatedToken, pbar: tqdm, pool: Pool) -> None:
    with contextlib.suppress(ForeignKeyViolationError):
        await pool.execute(
            queries.tables.related_token.INSERT,
            related_token.id,
            related_token.card_id,
            related_token.token_id,
        )

    pbar.update()


async def insert_token_relations(pool: Pool) -> None:
    with tqdm(total=len(token_relations)) as pbar:
        pbar.set_description("Inserting token relations")
        pbar.refresh()
        await asyncio.gather(*(insert_relation(related_token, pbar, pool) for related_token in token_relations))


async def insert_combo(combo: Combo, pbar: tqdm, pool: Pool) -> None:
    with contextlib.suppress(ForeignKeyViolationError):
        await pool.execute(queries.tables.combo.INSERT, combo.id, combo.card_id, combo.combo_card_id)

    pbar.update()
