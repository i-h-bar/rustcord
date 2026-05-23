import asyncio
import base64
import logging
import os

from aiohttp import ClientSession, TCPConnector
from anyio import Path
from dotenv import load_dotenv
from tqdm import tqdm

load_dotenv()

logger = logging.getLogger(__name__)

DISCORD_API = "https://discord.com/api/v10"
# Application emojis are capped at 2000; MTG has ~900+ sets so we log a warning if we approach that.
EMOJI_LIMIT = 2000


def _sanitize_name(name: str) -> str:
    """Discord emoji names: 2-32 chars, alphanumeric + underscores only."""
    sanitized = "".join(c if c.isalnum() or c == "_" else "_" for c in name)
    if len(sanitized) < 2:
        sanitized = sanitized + "_"
    return sanitized[:32]


async def _fetch_existing_emojis(session: ClientSession, application_id: str) -> dict[str, str]:
    url = f"{DISCORD_API}/applications/{application_id}/emojis"
    response = await session.get(url)
    if response.status != 200:
        text = await response.text()
        logger.error(f"Failed to fetch existing emojis: {response.status} {text}")
        return {}

    data = await response.json()
    return {item["name"]: item["id"] for item in data.get("items", [])}


async def _delete_emoji(
    session: ClientSession,
    application_id: str,
    name: str,
    emoji_id: str,
    semaphore: asyncio.Semaphore,
) -> None:
    async with semaphore:
        url = f"{DISCORD_API}/applications/{application_id}/emojis/{emoji_id}"
        response = await session.delete(url)
        if response.status == 429:
            retry_after = (await response.json()).get("retry_after", 1.0)
            await asyncio.sleep(retry_after)
            response = await session.delete(url)
        if response.status not in (200, 204):
            text = await response.text()
            logger.error(f"Failed to delete emoji {name}: {response.status} {text}")


async def _upload_emoji(
    session: ClientSession,
    application_id: str,
    name: str,
    png_path: Path,
    pbar: tqdm,
    semaphore: asyncio.Semaphore,
) -> None:
    async with semaphore:
        png_bytes = await png_path.read_bytes()
        encoded = base64.b64encode(png_bytes).decode()
        payload = {"name": name, "image": f"data:image/png;base64,{encoded}"}

        url = f"{DISCORD_API}/applications/{application_id}/emojis"
        response = await session.post(url, json=payload)

        if response.status == 201:
            logger.debug(f"Uploaded emoji: {name}")
        elif response.status == 429:
            retry_after = (await response.json()).get("retry_after", 1.0)
            logger.warning(f"Rate limited uploading {name}, retrying after {retry_after}s")
            await asyncio.sleep(retry_after)
            # Re-attempt once after backoff
            response = await session.post(url, json=payload)
            if response.status != 201:
                logger.error(f"Failed to upload emoji {name} after retry: {response.status}")
        else:
            text = await response.text()
            logger.error(f"Failed to upload emoji {name}: {response.status} {text}")

        pbar.update()


async def sync_set_symbol_emojis(replace: bool = False) -> None:
    bot_token = os.getenv("BOT_TOKEN")
    application_id = os.getenv("APPLICATION_ID")
    images_dir = os.getenv("IMAGES_DIR")

    if not bot_token or not application_id or not images_dir:
        logger.error("BOT_TOKEN, APPLICATION_ID, and IMAGES_DIR must be set to sync emojis")
        return

    app_id: str = application_id
    sets_dir = Path(images_dir) / "sets"
    if not await sets_dir.exists():
        logger.warning("Sets directory not found, skipping emoji sync")
        return

    headers = {"Authorization": f"Bot {bot_token}"}
    connector = TCPConnector(limit=3)
    async with ClientSession(headers=headers, connector=connector) as session:
        existing = await _fetch_existing_emojis(session, app_id)
        logger.info(f"Found {len(existing)} existing application emojis")

        png_files = [f async for f in sets_dir.iterdir() if f.name.endswith(".png")]
        local_names = {_sanitize_name(f.stem): f for f in png_files}

        if replace and existing:
            to_delete = {name: emoji_id for name, emoji_id in existing.items() if name in local_names}
            logger.info(f"Replacing {len(to_delete)} existing emojis")
            semaphore = asyncio.Semaphore(3)
            with tqdm(total=len(to_delete), desc="Deleting existing emojis") as pbar:

                async def _delete_and_tick(name: str, emoji_id: str) -> None:
                    await _delete_emoji(session, app_id, name, emoji_id, semaphore)
                    pbar.update()

                await asyncio.gather(*(_delete_and_tick(name, eid) for name, eid in to_delete.items()))
            existing = {name: eid for name, eid in existing.items() if name not in to_delete}

        to_upload = [(name, path) for name, path in local_names.items() if name not in existing]
        logger.info(f"Uploading {len(to_upload)} set symbol emojis")

        if len(existing) + len(to_upload) > EMOJI_LIMIT:
            logger.warning(
                f"Would exceed Discord emoji limit ({EMOJI_LIMIT}). "
                f"Truncating to {EMOJI_LIMIT - len(existing)} uploads."
            )
            to_upload = to_upload[: EMOJI_LIMIT - len(existing)]

        if not to_upload:
            logger.info("All set symbol emojis are already synced")
            return

        semaphore = asyncio.Semaphore(3)
        with tqdm(total=len(to_upload)) as pbar:
            pbar.set_description("Uploading set symbol emojis")
            await asyncio.gather(
                *(_upload_emoji(session, app_id, name, path, pbar, semaphore) for name, path in to_upload)
            )
