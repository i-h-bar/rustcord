import uuid
from typing import TYPE_CHECKING

from pydantic import BaseModel, Field

if TYPE_CHECKING:
    from utils.custom_types import JSONType


class RelatedToken(BaseModel):
    card_id: str
    token_id: str
    id: str = Field(default_factory=lambda: str(uuid.uuid4()))


def extract_tokens(card: dict[str, JSONType]) -> list[RelatedToken]:
    return [
        RelatedToken(token_id=part["id"], card_id=card["id"])
        for part in card.get("all_parts", ())
        if part["component"] == "token"
    ]
