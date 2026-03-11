import asyncio
import os

import asyncpg
from dotenv import load_dotenv

from utils.images import download_missing_images

load_dotenv()


async def main() -> None:
    async with asyncpg.create_pool(dsn=os.getenv("PSQL_URI")) as pool:
        await download_missing_images(pool)


if __name__ == "__main__":
    asyncio.run(main())
