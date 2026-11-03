"""Identify the bell tower heard in a recording by matching bell nominals against Dove's Guide."""

import json
from os import PathLike

from . import _core

__all__ = ["locate"]


def locate(
    path: str | PathLike,
    dove_dir: str | PathLike = "data/dove",
    near: str | None = None,
    start: float = 0.0,
    end: float | None = None,
    top: int = 10,
) -> dict:
    """Ranked candidate towers for `path`. `near` is "lat,lon[,radius_km]"."""
    return json.loads(_core.locate_json(str(path), str(dove_dir), near, start, end, top))
