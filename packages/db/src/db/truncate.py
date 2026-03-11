import logging
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from asyncpg import Pool

logger = logging.getLogger(__name__)


async def truncate_db(pool: Pool) -> None:
    logging.info("Truncating current database...")
    await pool.execute("truncate table related_token restart identity cascade")
    await pool.execute("truncate table combo restart identity cascade")
    await pool.execute("truncate table card restart identity cascade")
    await pool.execute("truncate table set restart identity cascade")
    await pool.execute("truncate table image restart identity cascade")
    await pool.execute("truncate table illustration restart identity cascade")
    await pool.execute("truncate table legality restart identity cascade")
    await pool.execute("truncate table rule restart identity cascade")
    await pool.execute("truncate table artist restart identity cascade")
