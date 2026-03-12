import asyncio
import os
from logging.config import fileConfig
from typing import TYPE_CHECKING

from dotenv import load_dotenv
from sqlalchemy import engine_from_config, pool
from sqlalchemy.ext.asyncio import AsyncEngine

from alembic import context

if TYPE_CHECKING:
    from asyncpg import Connection

load_dotenv()

config = context.config
if config.config_file_name is not None:
    fileConfig(config.config_file_name)

target_metadata = None

user = os.getenv("POSTGRES_USER")
password = os.getenv("POSTGRES_PW")
host = os.getenv("POSTGRES_HOST", "localhost:5432")
db = os.getenv("POSTGRES_DB")
URI = f"postgresql://{user}:{password}@{host}/{db}"
parts = URI.split(":")
parts[0] = "postgresql+asyncpg"
database_url = ":".join(parts)
config.set_main_option("sqlalchemy.url", database_url)


def run_migrations_online() -> None:
    connectable = context.config.attributes.get("connection", None)
    if connectable is None:
        connectable = AsyncEngine(
            engine_from_config(
                context.config.get_section(context.config.config_ini_section),
                prefix="sqlalchemy.",
                poolclass=pool.NullPool,
                future=True,
            )
        )

    if isinstance(connectable, AsyncEngine):
        asyncio.run(run_async_migrations(connectable))
    else:
        do_run_migrations(connectable)


async def run_async_migrations(connectable: AsyncEngine) -> None:
    async with connectable.connect() as connection:
        await connection.run_sync(do_run_migrations)
    await connectable.dispose()


def do_run_migrations(connection: Connection) -> None:
    context.configure(
        connection=connection,
        target_metadata=target_metadata,
        compare_type=True,
    )
    with context.begin_transaction():
        context.run_migrations()


run_migrations_online()
