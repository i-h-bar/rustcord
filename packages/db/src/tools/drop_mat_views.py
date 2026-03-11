import asyncio
import os

import asyncpg
from asyncpg import OutOfMemoryError
from dotenv import load_dotenv

from db.materialized_view import slow_drop_all_mv
from db.queries.materialised_views.drop_all import DROP_MAT_VIEWS

load_dotenv()


async def main() -> None:
    async with asyncpg.create_pool(dsn=os.getenv("PSQL_URI")) as pool:
        try:
            await pool.execute(DROP_MAT_VIEWS)
        except OutOfMemoryError:
            await slow_drop_all_mv(pool)


if __name__ == "__main__":
    asyncio.run(main())
