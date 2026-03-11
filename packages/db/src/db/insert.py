import asyncio
from typing import TYPE_CHECKING

from tqdm import tqdm

from db import queries
from db.delete import truncate_changeable_tables
from db.post_bulk_inserts import insert_token_relations
from models.card_info import CardInfo
from models.post_inserts import token_relations
from utils.card_cache import artist_cache, illustration_cache
from utils.combo_updates import insert_combos

if TYPE_CHECKING:
    from asyncpg import Pool

    from utils.custom_types import JSONType


async def _insert_card(card_info: CardInfo, pool: Pool) -> None:
    artist = card_info.artist
    if artist.id not in artist_cache:
        await pool.execute(queries.tables.artist.INSERT, artist.id, artist.name, artist.normalised_name)
        artist_cache.add(artist.id)

    illustration = card_info.illustration
    if illustration and illustration.id not in illustration_cache:
        await pool.execute(queries.tables.illustration.INSERT, illustration.id, illustration.scryfall_url)
        illustration_cache.add(illustration.id)

    image = card_info.image
    await pool.execute(queries.tables.image.UPSERT, image.id, image.scryfall_url)

    legality = card_info.legality
    await pool.execute(
        queries.tables.legality.UPSERT,
        legality.id,
        legality.alchemy,
        legality.brawl,
        legality.commander,
        legality.duel,
        legality.future,
        legality.gladiator,
        legality.historic,
        legality.legacy,
        legality.modern,
        legality.oathbreaker,
        legality.oldschool,
        legality.pauper,
        legality.paupercommander,
        legality.penny,
        legality.pioneer,
        legality.predh,
        legality.premodern,
        legality.standard,
        legality.standardbrawl,
        legality.timeless,
        legality.vintage,
        legality.game_changer,
    )

    rule = card_info.rule
    await pool.execute(
        queries.tables.rule.UPSERT,
        rule.id,
        rule.colour_identity,
        rule.mana_cost,
        rule.cmc,
        rule.power,
        rule.toughness,
        rule.loyalty,
        rule.defence,
        rule.type_line,
        rule.oracle_text,
        rule.colours,
        rule.keywords,
        rule.produced_mana,
        rule.rulings_url,
    )

    set_ = card_info.set
    await pool.execute(queries.tables.sets.INSERT, set_.id, set_.name, set_.normalised_name, set_.abbreviation)

    card = card_info.card
    await pool.execute(
        queries.tables.card.UPSERT,
        card.id,
        card.oracle_id,
        card.name,
        card.normalised_name,
        card.scryfall_url,
        card.flavour_text,
        card.release_date,
        card.reserved,
        card.rarity,
        card.artist_id,
        card.image_id,
        card.illustration_id,
        card.set_id,
        card.backside_id,
    )

    price = card_info.price
    await pool.execute(
        queries.tables.price.UPSERT,
        price.id,
        price.usd,
        price.usd_foil,
        price.usd_etched,
        price.euro,
        price.euro_foil,
        price.tix,
        price.updated_time,
    )

    for related_token in card_info.related_tokens:
        token_relations.append(related_token)


async def insert_card(card: dict, pbar: tqdm, pool: Pool) -> None:
    card_infos = CardInfo.parse_card(card)
    if not card_infos:
        pbar.update()
        return

    for card_info in card_infos:
        await _insert_card(card_info, pool)

    pbar.update()


async def insert_data(data: tuple[dict[str, JSONType], ...], pool: Pool) -> None:
    await truncate_changeable_tables(pool)

    with tqdm(total=len(data)) as pbar:
        pbar.set_description("Upserting Cards")
        pbar.refresh()
        await asyncio.gather(*(insert_card(card, pbar, pool) for card in data))

    await insert_token_relations(pool)
    await insert_combos(data, pool)
