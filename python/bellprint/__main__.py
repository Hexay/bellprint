import sys

from . import _core


def main() -> None:
    _core.main(["bellprint", *sys.argv[1:]])


if __name__ == "__main__":
    main()
