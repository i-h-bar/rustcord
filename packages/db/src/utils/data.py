import asyncio
import json
import logging
import os
import re
from datetime import datetime, timedelta, timezone
from typing import Any, Self

import aiohttp
from aiohttp import ClientResponse
from anyio import Path
from dotenv import load_dotenv

load_dotenv()

logger = logging.getLogger(__name__)
file_regex = re.compile(r"default-cards-(\d+\+\d{4})\.json")
set_file_regex = re.compile(r"sets-(\d+\+\d{4})\.json")
DATE_FORMAT = "%Y%m%d%H%M%S%z"


class DateCache:
    def __init__(self: Self) -> None:
        self._date = None

    @property
    def date(self: Self) -> datetime:
        if self._date is None:
            self._date = datetime.now(tz=timezone.utc)

        return self._date

    @date.setter
    def date(self: Self, value: datetime) -> None:
        self._date = value

    def extract_datetime(self: Self, response: ClientResponse) -> None:
        self._date = datetime.strptime(response.headers.get("date"), "%a, %d %b %Y %H:%M:%S %Z").replace(
            tzinfo=timezone.utc
        )


async def look_for_card_data_file(rx: re.Pattern) -> Path | None:
    return_file = None
    async for file in Path().iterdir():
        if match := rx.match(file.name):
            date = datetime.strptime(match.group(1), DATE_FORMAT)
            date_cache.date = date
            if date < (datetime.now(tz=timezone.utc) - timedelta(days=6, hours=23)):
                logger.info(f"Deleting stale card data: {file.name}")
                await file.unlink()
            else:
                return_file = file

    return return_file


async def load_data_file(data_file: Path) -> list[dict]:
    return json.loads(await data_file.read_text())


async def download_scryfall_set_data() -> list[dict[str, Any]] | None:
    logger.info("Downloading scryfall set data.")
    async with aiohttp.ClientSession() as session:
        sets_response = await session.get("https://api.scryfall.com/sets")
        date_cache.extract_datetime(sets_response)
        data = await sets_response.json()

    if data := data.get("data"):
        await Path(f"sets-{date_cache.date.strftime(DATE_FORMAT)}.json").write_text(json.dumps(data))

        return data

    return None


async def download_scryfall_card_data() -> list[dict] | None:
    logger.info("Downloading cards from Scryfall")
    async with aiohttp.ClientSession() as session:
        cards_response = await session.get("https://api.scryfall.com/bulk-data")
        date_cache.extract_datetime(cards_response)
        bulk_data_info = await cards_response.json()

        for category in bulk_data_info["data"]:
            if category["type"] == "default_cards":
                cards_response = await session.get(category["download_uri"])
                data = await cards_response.json()

                if os.getenv("DEV"):
                    cards_path = Path(f"default-cards-{date_cache.date.strftime(DATE_FORMAT)}.json")
                    await cards_path.write_text(json.dumps(data))

                return data

        return None


async def load_scryfall_data() -> tuple[list[dict] | None, list[dict[str, Any]] | None]:
    card_loader = None
    if data_file := os.getenv("FILE"):
        logger.info("Found file specified by environment variables.")
        card_loader = load_data_file(Path(data_file))

    if (data_file := await look_for_card_data_file(file_regex)) and os.getenv("DEV"):
        logger.info("Found cached card data from previous download less than a week old.")
        card_loader = load_data_file(data_file)

    if not card_loader:
        card_loader = download_scryfall_card_data()

    if (data_file := await look_for_card_data_file(set_file_regex)) and os.getenv("DEV"):
        logger.info("Found cached set data from previous download less than a week old.")
        sets_loader = load_data_file(data_file)
    else:
        sets_loader = download_scryfall_set_data()

    return await asyncio.gather(card_loader, sets_loader)


date_cache = DateCache()
