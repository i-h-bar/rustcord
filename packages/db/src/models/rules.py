from typing import TYPE_CHECKING, Self

from pydantic import BaseModel

if TYPE_CHECKING:
    from utils.custom_types import JSONType


class Rule(BaseModel):
    id: str
    colour_identity: list[str]
    mana_cost: str | None
    cmc: float
    power: str | None
    toughness: str | None
    loyalty: str | None
    defence: str | None
    type_line: str | None
    oracle_text: str | None
    colours: list[str] | None
    keywords: list[str] | None
    produced_mana: list[str] | None
    rulings_url: str | None

    @classmethod
    def from_card(cls: type[Self], card: dict[str, JSONType]) -> Self:
        return cls(
            id=card["oracle_id"],
            colour_identity=card["color_identity"],
            mana_cost=card["mana_cost"],
            cmc=card.get("cmc", 0.0),
            power=card.get("power"),
            toughness=card.get("toughness"),
            loyalty=card.get("loyalty"),
            defence=card.get("defense"),
            type_line=card["type_line"],
            oracle_text=card.get("oracle_text"),
            colours=card.get("colors", []),
            keywords=card.get("keywords", []),
            produced_mana=card.get("produced_mana"),
            rulings_url=card.get("rulings_uri"),
        )

    @classmethod
    def from_side(cls: type[Self], card_id: str, side: dict[str, JSONType], card: dict[str, JSONType]) -> Self:
        return cls(
            id=card_id,
            colour_identity=card["color_identity"],
            mana_cost=side.get("mana_cost"),
            cmc=card.get("cmc", 0.0),
            power=side.get("power"),
            toughness=side.get("toughness"),
            loyalty=side.get("loyalty"),
            defence=side.get("defense"),
            type_line=side.get("type_line"),
            oracle_text=side.get("oracle_text"),
            colours=card.get("colors", []),
            keywords=card.get("keywords", []),
            produced_mana=side.get("produced_mana"),
            rulings_url=card.get("rulings_uri"),
        )
