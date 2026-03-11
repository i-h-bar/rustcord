import asyncio
from typing import TYPE_CHECKING

from tqdm import tqdm

from db.post_bulk_inserts import insert_combo
from models.combos import Combo

if TYPE_CHECKING:
    from asyncpg import Pool


async def insert_combos(data: tuple[dict, ...], pool: Pool) -> None:
    current_combos = {
        (str(record["card_id"]), str(record["combo_card_id"]))
        for record in await pool.fetch("select card_id, combo_card_id from combo")
    }

    combos = []
    for card in data:
        if parts := card.get("all_parts"):
            for part in parts:
                if part["component"] == "combo_piece":
                    card_id, combo_card_id = card["id"], part["id"]
                    if (card_id, combo_card_id) not in current_combos:
                        combos.append(Combo(card_id=card_id, combo_card_id=combo_card_id))

    with tqdm(total=len(combos)) as pbar:
        pbar.set_description("Inserting combos")
        pbar.refresh()
        await asyncio.gather(*(insert_combo(combo, pbar, pool) for combo in combos))
