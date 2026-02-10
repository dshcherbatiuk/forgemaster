"""Shared utilities for slide generators.

Provides theme loading, Mermaid extraction, and diagram rendering
used by all generator backends (fpdf2, LaTeX, pptx).
"""

import json
import os
import re
import subprocess
import tempfile
from pathlib import Path

SLIDES_DIR = Path(__file__).parent
PROJECT_ROOT = SLIDES_DIR.parent
DEFAULT_PITCH_DECK_MD = PROJECT_ROOT / "docs" / "pitch-deck.md"
THEME_JSON = SLIDES_DIR / "theme.json"

theme = json.loads(THEME_JSON.read_text())


def hex_to_rgb(hex_color: str) -> tuple[int, int, int]:
    h = hex_color.lstrip("#")
    return (int(h[0:2], 16), int(h[2:4], 16), int(h[4:6], 16))


def generate_mermaid_config(target_dir: Path) -> Path:
    """Generate mermaid-config.json from theme.json. Returns config path."""
    bg = theme["bg"]
    accent = theme["accent"]
    is_dark = sum(hex_to_rgb(bg)) < 384
    config = {
        "theme": "dark" if is_dark else "default",
        "themeVariables": {
            "primaryColor": theme["table_header"],
            "primaryTextColor": theme["title"],
            "primaryBorderColor": accent,
            "lineColor": accent,
            "secondaryColor": theme["table_row"],
            "tertiaryColor": bg,
            "background": bg,
            "mainBkg": theme["table_header"],
            "nodeBorder": accent,
            "clusterBkg": theme["table_row"],
            "clusterBorder": theme["muted"],
            "titleColor": theme["title"],
            "edgeLabelBackground": bg,
        },
    }
    target_dir.mkdir(parents=True, exist_ok=True)
    config_path = target_dir / "mermaid-config.json"
    config_path.write_text(json.dumps(config, indent=2))
    return config_path


def extract_mermaid_blocks(md_path: Path) -> list[tuple[str, str]]:
    """Extract mermaid code blocks from markdown. Returns (slide_id, code) pairs."""
    content = md_path.read_text()
    blocks = []
    slide_pattern = re.compile(
        r"## Slide (\d+)[^\n]*\n(.*?)(?=\n## Slide |\Z)", re.DOTALL
    )
    mermaid_pattern = re.compile(r"```mermaid\n(.*?)```", re.DOTALL)

    for slide_match in slide_pattern.finditer(content):
        slide_num = slide_match.group(1)
        slide_content = slide_match.group(2)
        for i, mermaid_match in enumerate(mermaid_pattern.finditer(slide_content)):
            block_id = f"slide{slide_num}_diagram{i}"
            blocks.append((block_id, mermaid_match.group(1).strip()))

    return blocks


def render_mermaid_diagrams(
    blocks: list[tuple[str, str]],
    diagrams_dir: Path,
    mermaid_config: Path,
) -> dict[str, Path]:
    """Render mermaid blocks to PNG files using mmdc. Returns {block_id: png_path}."""
    diagrams_dir.mkdir(parents=True, exist_ok=True)
    rendered = {}

    for block_id, code in blocks:
        png_path = diagrams_dir / f"{block_id}.png"
        if png_path.exists():
            print(f"  [cached] {block_id}")
            rendered[block_id] = png_path
            continue

        with tempfile.NamedTemporaryFile(mode="w", suffix=".mmd", delete=False) as f:
            f.write(code)
            mmd_path = f.name

        try:
            cmd = [
                "npx", "--yes", "@mermaid-js/mermaid-cli",
                "-i", mmd_path,
                "-o", str(png_path),
                "-c", str(mermaid_config),
                "-b", theme["bg"],
                "-w", "1600",
                "-s", "2",
            ]
            result = subprocess.run(
                cmd, capture_output=True, text=True, timeout=60
            )
            if result.returncode != 0:
                print(f"  [WARN] {block_id}: {result.stderr[:200]}")
                continue
            print(f"  [rendered] {block_id}")
            rendered[block_id] = png_path
        finally:
            os.unlink(mmd_path)

    return rendered
