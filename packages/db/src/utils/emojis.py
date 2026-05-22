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


async def _fetch_existing_emojis(session: ClientSession, application_id: str) -> set[str]:
    url = f"{DISCORD_API}/applications/{application_id}/emojis"
    response = await session.get(url)
    if response.status != 200:
        text = await response.text()
        logger.error(f"Failed to fetch existing emojis: {response.status} {text}")
        return set()

    data = await response.json()
    return {item["name"] for item in data.get("items", [])}


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


async def sync_set_symbol_emojis() -> None:
    bot_token = os.getenv("BOT_TOKEN")
    application_id = os.getenv("APPLICATION_ID")
    images_dir = os.getenv("IMAGES_DIR")

    if not bot_token or not application_id or not images_dir:
        logger.error("BOT_TOKEN, APPLICATION_ID, and IMAGES_DIR must be set to sync emojis")
        return

    sets_dir = Path(images_dir) / "sets"
    if not await sets_dir.exists():
        logger.warning("Sets directory not found, skipping emoji sync")
        return

    headers = {"Authorization": f"Bot {bot_token}"}
    connector = TCPConnector(limit=3)
    async with ClientSession(headers=headers, connector=connector) as session:
        existing = await _fetch_existing_emojis(session, application_id)
        logger.info(f"Found {len(existing)} existing application emojis")

        png_files = [f async for f in sets_dir.iterdir() if f.name.endswith(".png")]
        to_upload = []
        for png_path in png_files:
            name = _sanitize_name(png_path.stem)
            if name not in existing:
                to_upload.append((name, png_path))

        logger.info(f"Uploading {len(to_upload)} new set symbol emojis")

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
                *(_upload_emoji(session, application_id, name, path, pbar, semaphore) for name, path in to_upload)
            )
