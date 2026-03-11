import re

from utils.maths import increment_uuid

art_id_regex = re.compile(
    r"https://cards\.scryfall\.io/(png|art_crop)/(front|back)/[0-9a-fA-F]/[0-9a-fA-F]/([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12})\.(png|jpg)\?\d+"
)


def parse_art_id(scryfall_url: str | None) -> str | None:
    if not scryfall_url:
        return None

    match = art_id_regex.match(scryfall_url)
    if not match:
        return None

    image_id = match.group(3)

    side = match.group(2)
    if side == "front":
        return image_id

    return increment_uuid(image_id)
