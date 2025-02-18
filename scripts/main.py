import asyncio
import os

import aiohttp
import asyncpg
from tqdm import tqdm
from dotenv import load_dotenv

load_dotenv()

async def main():
    db: asyncpg.Connection  = await asyncpg.connect(os.getenv("PSQL_URI"))
    try:
        await db.execute("ALTER TABLE cards ADD normalised_name varchar(150)")
    except asyncpg.exceptions.DuplicateColumnError:
        pass

    names = [record["name"] for record in await db.fetch("SELECT distinct name FROM cards")]

    async with aiohttp.ClientSession() as client:
        for normalised_name in tqdm(names):
            for _ in range(10):
                response = await client.get(f"https://api.scryfall.com/cards/search?q={normalised_name.replace(' ', '%20')}")
                if response.status != 200:
                    await asyncio.sleep(10)
                    continue

                data = await response.json()
                name = data["data"][0]["name"]

                await db.execute("UPDATE cards SET normalised_name=$1 WHERE name=$2", normalised_name, normalised_name)
                await db.execute("UPDATE cards SET name=$1 WHERE normalised_name=$2", name, normalised_name)


if __name__ == '__main__':
    asyncio.run(main())