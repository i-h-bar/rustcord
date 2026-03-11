from typing import TYPE_CHECKING, Self

from pydantic import BaseModel

if TYPE_CHECKING:
    from utils.custom_types import JSONType


class Legality(BaseModel):
    id: str
    alchemy: str
    brawl: str
    commander: str
    duel: str
    future: str
    gladiator: str
    historic: str
    legacy: str
    modern: str
    oathbreaker: str
    oldschool: str
    pauper: str
    paupercommander: str
    penny: str
    pioneer: str
    predh: str
    premodern: str
    standard: str
    standardbrawl: str
    timeless: str
    vintage: str
    game_changer: bool

    @classmethod
    def from_card(cls: type[Self], card: dict[str, JSONType]) -> Self:
        return cls(
            id=card["oracle_id"],
            game_changer=card.get("game_changer"),
            **card["legalities"],
        )

    @classmethod
    def from_side(cls: type[Self], card_id: str, card: dict[str, JSONType]) -> Self:
        return cls(
            id=card_id,
            game_changer=card.get("game_changer"),
            **card["legalities"],
        )
