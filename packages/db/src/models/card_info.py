from typing import TYPE_CHECKING, Self

from pydantic import BaseModel, Field
from utils.maths import increment_uuid
from utils.normalise import normalise

from models.artists import Artist
from models.cards import Card
from models.combos import Combo, extract_combos
from models.illustrations import Illustration
from models.images import Image
from models.legalities import Legality
from models.price import Price
from models.related_tokens import RelatedToken, extract_tokens
from models.rules import Rule
from models.sets import Set

if TYPE_CHECKING:
    from utils.custom_types import JSONType


class CardInfo(BaseModel):
    card: Card
    artist: Artist
    legality: Legality
    image: Image
    illustration: Illustration | None
    rule: Rule
    set: Set
    related_tokens: list[RelatedToken] = Field(default_factory=list)
    combos: list[Combo] = Field(default_factory=list)
    price: Price

    @classmethod
    def from_card(cls: type[Self], card: dict[str, JSONType]) -> Self | None:
        image = Image.from_card(card)
        if not image:
            return None

        artist = Artist.from_card(card)
        rule = Rule.from_card(card)
        legality = Legality.from_card(card)
        illustration = Illustration.from_card(card)
        set_ = Set.from_card(card)
        card_model = Card.from_card(card, artist, image, set_, illustration)
        price = Price.from_card(card)

        combos = extract_combos(card)
        related_tokens = extract_tokens(card)

        return CardInfo(
            card=card_model,
            artist=artist,
            rule=rule,
            legality=legality,
            image=image,
            illustration=illustration,
            set=set_,
            related_tokens=related_tokens,
            combos=combos,
            price=price,
        )

    @classmethod
    def produce_side(
        cls: type[Self],
        side_id: str,
        side_oracle_id: str,
        reverse_side_id: str,
        side: dict[str, JSONType],
        card: dict[str, JSONType],
        set_: Set,
    ) -> Self | None:
        if not (image := Image.from_side(side, card)):
            return None

        artist = Artist.from_side(side, card)
        legality = Legality.from_side(side_oracle_id, card)
        rule = Rule.from_side(side_oracle_id, side, card)
        illustration = Illustration.from_side(side, card)
        card_model = Card.from_side(
            side_id, side_oracle_id, reverse_side_id, side, card, artist, image, set_, illustration
        )
        combos = extract_combos(card)
        related_tokens = extract_tokens(card)

        return CardInfo(
            card=card_model,
            artist=artist,
            rule=rule,
            image=image,
            illustration=illustration,
            legality=legality,
            set=set_,
            related_tokens=related_tokens,
            combos=combos,
            price=Price.from_card(card),
        )

    @classmethod
    def produce_sides(
        cls: type[Self], card: dict[str, JSONType], front: dict[str, JSONType], back: dict[str, JSONType]
    ) -> tuple[Self, Self] | None:
        back_id = increment_uuid(card["id"])
        front_oracle_id = front.get("oracle_id") or card.get("oracle_id")
        back_oracle_id = increment_uuid(front_oracle_id)

        set_ = Set(
            id=card["set_id"],
            name=card["set_name"],
            normalised_name=normalise(card["set_name"]),
            abbreviation=card["set"],
        )

        front = CardInfo.produce_side(card["id"], front_oracle_id, back_id, front, card, set_)
        if not front:
            return None

        back = CardInfo.produce_side(back_id, back_oracle_id, front.card.id, back, card, set_)
        if not back:
            return None

        return front, back

    @classmethod
    def produce_sides_matching_names(
        cls: type[Self], card: dict[str, JSONType], front: dict[str, JSONType], back: dict[str, JSONType]
    ) -> tuple[Self, Self] | None:
        back_id = increment_uuid(card["id"])
        oracle_id = front.get("oracle_id") or card.get("oracle_id")

        set_ = Set(
            id=card["set_id"],
            name=card["set_name"],
            normalised_name=normalise(card["set_name"]),
            abbreviation=card["set"],
        )

        front = CardInfo.produce_side(card["id"], oracle_id, back_id, front, card, set_)
        if not front:
            return None

        back = CardInfo.produce_side(back_id, oracle_id, front.card.id, back, card, set_)
        if not back:
            return None

        return front, back

    @classmethod
    def parse_card(cls: type[Self], card: dict[str, str | int | list]) -> tuple[Self] | tuple[Self, Self] | None:
        if not (sides := card.get("card_faces")):
            if card := CardInfo.from_card(card):
                return (card,)
            return None

        if sides[0].get("name") == sides[1].get("name"):
            return CardInfo.produce_sides_matching_names(card, sides[0], sides[1])

        return CardInfo.produce_sides(card, sides[0], sides[1])
