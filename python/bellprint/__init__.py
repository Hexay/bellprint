"""Identify the bell tower heard in a recording by matching bell nominals against Dove's Guide."""

import json
from importlib.metadata import version
from os import PathLike

from . import _core

__all__ = ["locate"]
__version__ = version("bellprint")


def locate(
    path: str | PathLike,
    near: str | None = None,
    start: float = 0.0,
    end: float | None = None,
    top: int = 10,
    dove_dir: str | PathLike | None = None,
    use_geotag: bool = False,
) -> dict:
    """Ranked candidate towers for `path`. `near` is "lat,lon[,radius_km]".

    `use_geotag` restricts to a default-radius circle around the GPS position in the file's own
    metadata, if any; `near` wins. Uses the Dove snapshot built into the package unless `dove_dir` holds
    a newer dove.csv and bells.csv.
    """
    dove = None if dove_dir is None else str(dove_dir)
    return json.loads(_core.locate_json(str(path), dove, near, start, end, top, use_geotag))
