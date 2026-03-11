from datetime import datetime
from typing import TYPE_CHECKING, Self

from pydantic import BaseModel
from utils.normalise import normalise

if TYPE_CHECKING:
    from utils.custom_types import JSONType

    from models.artists import Artist
    from models.illustrations import Illustration
    from models.images import Image
    from models.sets import Set


class Card(BaseModel):
    id: str
    oracle_id: str
    name: str
    normalised_name: str
    scryfall_url: str
    flavour_text: str | None
    release_date: datetime
    reserved: bool
    rarity: str
    artist_id: str
    image_id: str | None
    illustration_id: str | None
    set_id: str
    backside_id: str | None = None

    @classmethod
    def from_card(
        cls: type[Self], card: dict[str, JSONType], artist: Artist, image: Image, set_: Set, illustration: Illustration
    ) -> Self:
        return cls(
            id=card["id"],
            oracle_id=card["oracle_id"],
            name=card["name"],
            normalised_name=normalise(card["name"]),
            scryfall_url=card["scryfall_uri"],
            flavour_text=card.get("flavor_text"),
            release_date=datetime.strptime(card["released_at"], "%Y-%m-%d"),
            reserved=card["reserved"],
            rarity=card["rarity"],
            artist_id=artist.id,
            image_id=image.id,
            illustration_id=None if not illustration else illustration.id,
            set_id=set_.id,
        )

    @classmethod
    def from_side(
        cls: type[Self],
        side_id: str,
        side_oracle_id: str,
        reverse_side_id: str,
        side: dict[str, JSONType],
        card: dict[str, JSONType],
        artist: Artist,
        image: Image,
        set_: Set,
        illustration: Illustration,
    ) -> Self:
        return cls(
            id=side_id,
            oracle_id=side_oracle_id,
            name=side["name"],
            normalised_name=normalise(side["name"]),
            scryfall_url=card["scryfall_uri"],
            flavour_text=side.get("flavor_text"),
            release_date=datetime.strptime(card["released_at"], "%Y-%m-%d"),
            reserved=card["reserved"],
            rarity=card["rarity"],
            artist_id=artist.id,
            image_id=image.id,
            illustration_id=None if not illustration else illustration.id,
            set_id=set_.id,
            backside_id=reverse_side_id,
        )
