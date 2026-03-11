import re
import string
import unicodedata

from unidecode import unidecode

SPACE_PUNC = re.compile(r"[_-]+")
OTHER_PUNC = re.compile(rf"[{string.punctuation}]+")


def normalise(name: str) -> str:
    return OTHER_PUNC.sub("", SPACE_PUNC.sub(" ", unidecode(unicodedata.normalize("NFKC", name)).lower()))
