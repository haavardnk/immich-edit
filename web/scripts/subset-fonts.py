# /// script
# requires-python = ">=3.12"
# dependencies = ["fonttools==4.68.0", "brotli==1.2.1"]
# ///
from dataclasses import dataclass
from functools import cache
from io import BytesIO
from pathlib import Path

from fontTools import subset
from fontTools.ttLib import TTFont
from fontTools.varLib.instancer import instantiateVariableFont

WEB = Path(__file__).resolve().parent.parent
SOURCES = WEB / "fonts"
OUTPUT = WEB / "static" / "fonts"
STYLESHEET = WEB / "src" / "fonts.css"

CORE_RANGES = (
    range(0x0000, 0x0530),
    range(0x1E00, 0x2C00),
    range(0xFEFF, 0xFF00),
    range(0xFFFD, 0xFFFE),
)


@dataclass(frozen=True)
class Face:
    family: str
    source: str
    stem: str
    pinned: dict[str, float]
    weight: str
    metrics: tuple[str, ...]
    split: bool


@dataclass(frozen=True)
class Subset:
    face: Face
    file: str
    codepoints: frozenset[int]
    ranged: bool


FACES = (
    Face(
        family="Google Sans",
        source="GoogleSans[GRAD,opsz,wght].ttf",
        stem="google-sans",
        pinned={"GRAD": 0},
        weight="400 700",
        metrics=("ascent-override: 106.25%", "size-adjust: 106.25%"),
        split=True,
    ),
    Face(
        family="Google Sans Code",
        source="GoogleSansCode[MONO,wght].ttf",
        stem="google-sans-code",
        pinned={"MONO": 1},
        weight="300 800",
        metrics=(),
        split=False,
    ),
)


def is_core(codepoint: int) -> bool:
    return any(codepoint in block for block in CORE_RANGES)


@cache
def pinned(source: str, axes: tuple[tuple[str, float], ...]) -> bytes:
    font = TTFont(SOURCES / source, recalcTimestamp=False)
    buffer = BytesIO()
    instantiateVariableFont(font, dict(axes)).save(buffer)
    return buffer.getvalue()


def load(face: Face) -> TTFont:
    axes = tuple(sorted(face.pinned.items()))
    return TTFont(BytesIO(pinned(face.source, axes)), recalcTimestamp=False)


def subsets(face: Face) -> list[Subset]:
    codepoints = frozenset(TTFont(SOURCES / face.source).getBestCmap())
    if not face.split:
        return [Subset(face, f"{face.stem}.woff2", codepoints, False)]
    core = frozenset(cp for cp in codepoints if is_core(cp))
    return [
        Subset(face, f"{face.stem}-core.woff2", core, True),
        Subset(face, f"{face.stem}-extended.woff2", codepoints - core, True),
    ]


def write(part: Subset) -> int:
    options = subset.Options()
    options.layout_features = ["*"]
    options.name_IDs = ["*"]
    options.name_languages = ["*"]
    options.notdef_outline = True
    subsetter = subset.Subsetter(options)
    subsetter.populate(unicodes=part.codepoints)
    font = load(part.face)
    subsetter.subset(font)
    font.flavor = "woff2"
    path = OUTPUT / part.file
    font.save(path)
    return path.stat().st_size


def spans(codepoints: frozenset[int]) -> list[tuple[int, int]]:
    result: list[tuple[int, int]] = []
    for cp in sorted(codepoints):
        if result and result[-1][1] == cp - 1:
            result[-1] = (result[-1][0], cp)
        else:
            result.append((cp, cp))
    return result


def unicode_range(codepoints: frozenset[int]) -> str:
    return ", ".join(
        f"U+{start:04X}" if start == end else f"U+{start:04X}-{end:04X}"
        for start, end in spans(codepoints)
    )


def rule(part: Subset) -> str:
    lines = [
        f"font-family: '{part.face.family}';",
        f"src: url('/fonts/{part.file}') format('woff2');",
        f"font-weight: {part.face.weight};",
        "font-style: normal;",
        "font-display: swap;",
        *(f"{metric};" for metric in part.face.metrics),
    ]
    if part.ranged:
        lines.append(f"unicode-range: {unicode_range(part.codepoints)};")
    body = "\n".join(f"  {line}" for line in lines)
    return f"@font-face {{\n{body}\n}}\n"


def main() -> None:
    OUTPUT.mkdir(parents=True, exist_ok=True)
    parts = [part for face in FACES for part in subsets(face)]
    for part in parts:
        size = write(part)
        print(f"{part.file}: {len(part.codepoints)} code points, {size} bytes")
    STYLESHEET.write_text("\n".join(rule(part) for part in parts))


if __name__ == "__main__":
    main()
