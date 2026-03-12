import json
import logging
import os
import re
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Self

import aiofiles
import aiohttp
from aiohttp import ClientResponse
from dotenv import load_dotenv

load_dotenv()

logger = logging.getLogger(__name__)
file_regex = re.compile(r"default-cards-(\d+\+\d{4})\.json")
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


def look_for_data_file() -> str | None:
    return_file = None
    for file in Path().iterdir():
        if match := file_regex.match(file.name):
            date = datetime.strptime(match.group(1), DATE_FORMAT)
            date_cache.date = date
            if date < (datetime.now(tz=timezone.utc) - timedelta(days=6, hours=23)):
                logger.info(f"Deleting stale card data: {file.name}")
                file.unlink()
            else:
                return_file = file

    return return_file


async def load_data_file(data_file: str | Path) -> list[dict]:
    async with aiofiles.open(data_file, encoding="utf-8") as file:
        return json.loads(await file.read())


async def download_scryfall_data() -> list[dict] | None:
    logger.info("Downloading from Scryfall")
    async with aiohttp.ClientSession() as session:
        response = await session.get("https://api.scryfall.com/bulk-data")
        date_cache.extract_datetime(response)
        bulk_data_info = await response.json()

        for category in bulk_data_info["data"]:
            if category["type"] == "default_cards":
                response = await session.get(category["download_uri"])
                data = await response.json()

                if os.getenv("DEV"):
                    async with aiofiles.open(
                        f"default-cards-{date_cache.date.strftime(DATE_FORMAT)}.json",
                        "w",
                        encoding="utf-8",
                    ) as file:
                        await file.write(json.dumps(data))

                return data

        return None


async def load_scryfall_data() -> list[dict] | None:
    if data_file := os.getenv("FILE"):
        logger.info("Found file specified by environment variables.")
        return await load_data_file(data_file)

    if (data_file := look_for_data_file()) and os.getenv("DEV"):
        logger.info("Found cached data from previous download less than a week old.")
        return await load_data_file(data_file)

    return await download_scryfall_data()


date_cache = DateCache()
