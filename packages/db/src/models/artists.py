from typing import TYPE_CHECKING, Self

from pydantic import BaseModel
from utils.normalise import normalise

if TYPE_CHECKING:
    from utils.custom_types import JSONType

MISSING_ID_ID = ["aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"]
MISSING_ARTIST = "Anonymous"


class Artist(BaseModel):
    id: str
    name: str
    normalised_name: str

    @classmethod
    def from_card(cls: type[Self], card: dict[str, JSONType]) -> Self:
        return cls(
            id=card.get("artist_ids", MISSING_ID_ID)[0],
            name=card["artist"] or MISSING_ARTIST,
            normalised_name=normalise(card["artist"] or MISSING_ARTIST),
        )

    @classmethod
    def from_side(cls: type[Self], side: dict[str, JSONType], card: dict[str, JSONType]) -> Self:
        artist_id = (side.get("artist_ids") or card.get("artist_ids") or MISSING_ID_ID)[0]
        artist_name = side.get("artist") or card.get("artist") or MISSING_ARTIST
        return cls(
            id=artist_id,
            name=artist_name,
            normalised_name=normalise(artist_name),
        )
