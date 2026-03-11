from typing import TYPE_CHECKING, Self

from pydantic import BaseModel
from utils.normalise import normalise

if TYPE_CHECKING:
    from utils.custom_types import JSONType


class Set(BaseModel):
    id: str
    name: str
    normalised_name: str
    abbreviation: str

    @classmethod
    def from_card(cls: type[Self], card: dict[str, JSONType]) -> Self:
        return cls(
            id=card["set_id"],
            name=card["set_name"],
            normalised_name=normalise(card["set_name"]),
            abbreviation=card["set"],
        )
