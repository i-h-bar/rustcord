import logging

from asyncpg import OutOfMemoryError, Pool
from tqdm import tqdm

from db.queries.materialised_views.drop_all import DROP_MAT_VIEWS

logger = logging.getLogger(__name__)


async def drop_all_mv(pool: Pool) -> None:
    logger.info("Dropping all materialised views...")
    try:
        await pool.execute(DROP_MAT_VIEWS)
    except OutOfMemoryError:
        await slow_drop_all_mv(pool)


async def slow_drop_all_mv(pool: Pool) -> None:
    mvs = await pool.fetchval(
        """SELECT array_agg(oid::regclass::text)
            FROM   pg_class
            WHERE  relkind = 'm';"""
    )
    if mvs is not None:
        with tqdm(total=len(mvs)) as pbar:
            pbar.set_description("Drop all MVs")
            pbar.refresh()
            for mv in mvs:
                await pool.execute(f"DROP MATERIALIZED VIEW {mv};")
                pbar.update()
