from typing import TYPE_CHECKING, Self

from pydantic import BaseModel
from utils.art_ids import parse_art_id

if TYPE_CHECKING:
    from utils.custom_types import JSONType


class Image(BaseModel):
    id: str | None
    scryfall_url: str | None

    @classmethod
    def from_card(cls: type[Self], card: dict[str, JSONType]) -> Self | None:
        if image_id := parse_art_id(card["image_uris"]["png"]):
            return cls(id=image_id, scryfall_url=card["image_uris"]["png"])

        return None

    @classmethod
    def from_side(cls: type[Self], side: dict[str, JSONType], card: dict[str, JSONType]) -> Self | None:
        image_uris = side.get("image_uris") or card.get("image_uris")
        if image_uris and (image_id := parse_art_id(image_uris.get("png"))):
            return cls(id=image_id, scryfall_url=image_uris["png"])
        return None
