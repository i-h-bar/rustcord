FROM python:3.14-slim AS builder
COPY --from=ghcr.io/astral-sh/uv:latest /uv /uvx /bin/

RUN apt-get update && apt-get upgrade -y && apt-get install -y --no-install-recommends libcairo2 && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY uv.lock pyproject.toml ./
COPY ./packages ./packages/

RUN uv sync --package db-management --frozen --no-dev

FROM python:3.14-slim AS prod

RUN apt-get update && apt-get install -y --no-install-recommends libcairo2 && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/.venv ./.venv
COPY ./packages/db/src ./packages/db/alembic.ini ./
COPY ./packages/db/alembic/ ./alembic/
