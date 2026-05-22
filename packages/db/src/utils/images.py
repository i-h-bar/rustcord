import asyncio
import contextlib
import io
import logging
import os
from typing import TYPE_CHECKING, Any

import cairosvg
from aiohttp import ClientSession, ClientTimeout, TCPConnector
from anyio import Path
from dotenv import load_dotenv
from PIL import Image, ImageOps
from tqdm import tqdm

if TYPE_CHECKING:
    from asyncpg import Pool, Record

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

load_dotenv()


async def fetch_image(record: Record, session: ClientSession, pbar: tqdm, directory: Path) -> None:
    proposed_path = directory / f"{record['id']}.png"

    if not await Path(proposed_path).exists():
        try:
            result = await session.get(record["scryfall_url"])
        except TimeoutError:
            pbar.update()
            return

        if result.status != 200:
            logger.warning(
                f"Could not find image for {record['id']} - {record['scryfall_url']} ~ Status code: {result.status}"
            )
            pbar.update()
            return

        try:
            png = await result.read()
        except Exception:  # noqa: BLE001
            pbar.update()
            return

        await Path(proposed_path).write_bytes(png)

    pbar.update()


async def download_missing_card_images(pool: Pool, base_dir: Path) -> None:
    images_dir = base_dir / "images"
    with contextlib.suppress(FileExistsError):
        await images_dir.mkdir(parents=True)

    all_urls = await pool.fetch("SELECT id, scryfall_url from image")
    all_urls = [record for record in all_urls if not await (images_dir / f"{record['id']}.png").exists()]

    logger.info(f"Found {len(all_urls)} images to download")

    with tqdm(total=len(all_urls)) as pbar:
        pbar.set_description("Fetching missing card images")
        pbar.refresh()

        connector = TCPConnector(limit=5)
        async with ClientSession(connector=connector, timeout=ClientTimeout(total=300)) as session:
            for record in all_urls:
                await fetch_image(record, session, pbar, images_dir)


async def download_missing_illustrations(pool: Pool, base_dir: Path) -> None:
    illustration_dir = base_dir / "illustrations"
    with contextlib.suppress(FileExistsError):
        await illustration_dir.mkdir(parents=True)

    all_urls = await pool.fetch("SELECT id, scryfall_url from illustration")
    all_urls = [record for record in all_urls if not await (illustration_dir / f"{record['id']}.png").exists()]

    logger.info(f"Found {len(all_urls)} illustrations to download")

    with tqdm(total=len(all_urls)) as pbar:
        pbar.set_description("Fetching missing illustrations")
        pbar.refresh()

        connector = TCPConnector(limit=5)
        async with ClientSession(connector=connector, timeout=ClientTimeout(total=300)) as session:
            for record in all_urls:
                await fetch_image(record, session, pbar, illustration_dir)


_RARE_GOLD = (194, 161, 75)


def _colourise_gold(png_bytes: bytes) -> bytes:
    img = Image.open(io.BytesIO(png_bytes)).convert("RGBA")
    _, _, _, alpha = img.split()
    coloured = ImageOps.colorize(img.convert("L"), black=_RARE_GOLD, white=(255, 255, 255))
    coloured = coloured.convert("RGBA")
    coloured.putalpha(alpha)
    output = io.BytesIO()
    coloured.save(output, format="PNG")
    return output.getvalue()


async def symbol_present(data: dict[str, Any], base_dir: Path) -> bool:
    if not (identifier := data.get("code")):
        return True

    return await (base_dir / f"{identifier}.png").exists()


async def filter_existence(data: dict[str, Any], base_dir: Path) -> dict[str, Any] | None:
    if await symbol_present(data, base_dir):
        return None

    return data


async def download_and_convert_symbol(
    session: ClientSession, data: dict[str, Any] | None, base_dir: Path, pbar: tqdm
) -> None:
    if not data:
        pbar.update()
        return

    identifier: str | None = data.get("code")
    svg_url: str | None = data.get("icon_svg_uri")

    if not identifier or not svg_url:
        logger.warning(f"Could not find symbol for {identifier} / {svg_url}")
        pbar.update()
        return

    try:
        response = await session.get(svg_url)
    except TimeoutError:
        logger.warning(f"Timeout downloading symbol for {identifier}")
        pbar.update()
        return

    if response.status != 200:
        logger.warning(f"Could not download symbol for {identifier} ~ Status: {response.status}")
        pbar.update()
        return

    try:
        svg_bytes = await response.read()
    except Exception:
        logger.exception(f"Could not download symbol for {identifier} / {svg_url}")
        pbar.update()
        return

    loop = asyncio.get_event_loop()
    png_bytes = await loop.run_in_executor(
        None, lambda: cairosvg.svg2png(bytestring=svg_bytes, output_width=256, output_height=256)
    )
    png_bytes = await loop.run_in_executor(None, _colourise_gold, png_bytes)
    await (base_dir / f"{identifier}.png").write_bytes(png_bytes)
    pbar.update()


async def download_missing_set_symbols(base_dir: Path, set_data: list[dict[str, Any]]) -> None:
    base_dir = base_dir / "sets"
    with contextlib.suppress(FileExistsError):
        await base_dir.mkdir(parents=True)

    data = await asyncio.gather(*(filter_existence(data, base_dir) for data in set_data))
    with tqdm(total=len(data)) as pbar:
        pbar.set_description("Downloading symbols")
        pbar.refresh()

        connector = TCPConnector(limit=5)
        async with ClientSession(connector=connector, timeout=ClientTimeout(total=300)) as session:
            await asyncio.gather(*(download_and_convert_symbol(session, data, base_dir, pbar) for data in data))


async def download_missing_images(pool: Pool, set_data: list[dict[str, Any]]) -> None:
    base_dir = Path(os.getenv("IMAGES_DIR"))
    await download_missing_card_images(pool, base_dir)
    await download_missing_illustrations(pool, base_dir)
    await download_missing_set_symbols(base_dir, set_data)
    logger.info(f"Card images can be found: {await (await base_dir.resolve()).absolute()!s}")
