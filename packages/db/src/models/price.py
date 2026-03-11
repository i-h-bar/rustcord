from datetime import datetime
from typing import TYPE_CHECKING, Self

from pydantic import BaseModel

from utils.data import date_cache

if TYPE_CHECKING:
    from utils.custom_types import JSONType


class Price(BaseModel):
    id: str
    usd: float | None
    usd_foil: float | None
    usd_etched: float | None
    euro: float | None
    euro_foil: float | None
    tix: float | None
    updated_time: datetime

    @classmethod
    def from_card(cls: type[Self], card: dict[str, JSONType]) -> Self:
        prices = card["prices"]
        if usd := prices.get("usd"):
            usd = float(usd)

        if usd_foil := prices.get("usd_foil"):
            usd_foil = float(usd_foil)

        if usd_etched := prices.get("usd_etched"):
            usd_etched = float(usd_etched)

        if euro := prices.get("eur"):
            euro = float(euro)

        if euro_foil := prices.get("eur_foil"):
            euro_foil = float(euro_foil)

        if tix := prices.get("tix"):
            tix = float(tix)

        return cls(
            id=card["id"],
            usd=usd,
            usd_foil=usd_foil,
            usd_etched=usd_etched,
            euro=euro,
            euro_foil=euro_foil,
            tix=tix,
            updated_time=date_cache.date,
        )
