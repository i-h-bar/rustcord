import uuid
from typing import TYPE_CHECKING

from pydantic import BaseModel, Field

if TYPE_CHECKING:
    from utils.custom_types import JSONType


class Combo(BaseModel):
    card_id: str
    combo_card_id: str
    id: str = Field(default_factory=lambda: str(uuid.uuid4()))


def extract_combos(card: dict[str, JSONType]) -> list[Combo]:
    return [
        Combo(card_id=card["id"], combo_card_id=part["id"])
        for part in card.get("all_parts", ())
        if part["component"] == "compo_piece"
    ]
