import asyncio
import logging
import os
import sys

import asyncpg
from db.insert import insert_data
from dotenv import load_dotenv
from utils.data import load_scryfall_data
from utils.emojis import sync_set_symbol_emojis
from utils.images import download_missing_images

load_dotenv()
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


async def main() -> None:
    card_data, set_data = await load_scryfall_data()
    if not card_data:
        logger.error("Scryfall data could not be loaded.")
        sys.exit(1)

    if not set_data:
        logger.error("Setting data could not be loaded.")
        sys.exit(1)

    user = os.getenv("POSTGRES_USER")
    password = os.getenv("POSTGRES_PW")
    host = os.getenv("POSTGRES_HOST", "localhost:5432")
    db = os.getenv("POSTGRES_DB")
    uri = f"postgresql://{user}:{password}@{host}/{db}"
    async with asyncpg.create_pool(dsn=uri) as pool:
        card_data = tuple(
            card
            for card in card_data
            if card.get("set_type") != "memorabilia"
            and card.get("image_uris", {}).get("png") != "https://errors.scryfall.com/soon.jpg"
        )

        await insert_data(card_data, pool)

        await download_missing_images(pool, set_data)
        await sync_set_symbol_emojis(replace=True)


if __name__ == "__main__":
    asyncio.run(main())
