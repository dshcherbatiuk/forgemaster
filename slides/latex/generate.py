#!/usr/bin/env python3
"""Generate ForgeMaster pitch deck PDF via LaTeX Beamer.

Extracts Mermaid diagrams from markdown, renders them to PNG via mmdc,
substitutes diagram placeholders in the Beamer template, and compiles
to PDF using tectonic.
"""

import argparse
import os
import re
import subprocess
import sys
from pathlib import Path

# Allow importing shared module from parent directory
sys.path.insert(0, str(Path(__file__).parent.parent))

from shared import (
    DEFAULT_PITCH_DECK_MD,
    PROJECT_ROOT,
    extract_mermaid_blocks,
    generate_mermaid_config,
    render_mermaid_diagrams,
    theme,
)

# -- Paths ------------------------------------------------------------------
SCRIPT_DIR = Path(__file__).parent
TEMPLATE_TEX = SCRIPT_DIR / "template.tex"
TARGET_DIR = PROJECT_ROOT / "target" / "slides-latex"
DIAGRAMS_DIR = TARGET_DIR / "diagrams"
OUTPUT_TEX = TARGET_DIR / "forgemaster-pitch.tex"
OUTPUT_PDF = TARGET_DIR / "forgemaster-pitch.pdf"

# Placeholder pattern: {{DIAGRAM_<id>_<width>}}
PLACEHOLDER_RE = re.compile(r"\{\{DIAGRAM_(\w+)_([\d.]+)\}\}")


def resolve_theme_placeholders(template: str) -> str:
    """Replace {{THEME_*}} placeholders with values from theme.json."""
    theme_map = {
        "THEME_BG": theme["bg"],
        "THEME_TITLE": theme["title"],
        "THEME_ACCENT": theme["accent"],
        "THEME_BODY": theme["body"],
        "THEME_MUTED": theme["muted"],
        "THEME_CODE_BG": theme["code_bg"],
        "THEME_TABLE_HEADER": theme["table_header"],
        "THEME_TABLE_ROW": theme["table_row"],
        "THEME_TABLE_ALT": theme["table_alt"],
    }
    for key, value in theme_map.items():
        # Strip # from hex for LaTeX \definecolor{}{HTML}{...}
        hex_val = value.lstrip("#").upper()
        template = template.replace(f"{{{{{key}}}}}", hex_val)
    return template


def resolve_placeholders(template: str, diagrams: dict[str, Path]) -> str:
    """Replace {{DIAGRAM_<id>_<width>}} placeholders with includegraphics."""

    def replacer(match: re.Match) -> str:
        diagram_id = match.group(1)
        width = match.group(2)
        if diagram_id not in diagrams:
            return f"% missing diagram: {diagram_id}"
        rel_path = os.path.relpath(diagrams[diagram_id], TARGET_DIR).replace("\\", "/")
        return (
            f"\\begin{{center}}\n"
            f"\\includegraphics[width={width}\\linewidth]{{{rel_path}}}\n"
            f"\\end{{center}}"
        )

    return PLACEHOLDER_RE.sub(replacer, template)


def compile_tex(tex_path: Path):
    """Compile .tex to PDF using tectonic."""
    result = subprocess.run(
        ["tectonic", "-X", "compile", str(tex_path)],
        capture_output=True,
        text=True,
        timeout=120,
    )
    if result.returncode != 0:
        print(f"  [ERROR] tectonic failed:\n{result.stderr}")
        raise SystemExit(1)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Generate ForgeMaster pitch deck PDF via LaTeX Beamer")
    parser.add_argument(
        "source",
        nargs="?",
        default=str(DEFAULT_PITCH_DECK_MD),
        help=f"Path to pitch deck markdown (default: {DEFAULT_PITCH_DECK_MD})",
    )
    return parser.parse_args()


def main():
    args = parse_args()
    source = Path(args.source)

    print("=== ForgeMaster Pitch Deck Generator (LaTeX) ===\n")

    # Step 0: Generate mermaid config from theme
    mermaid_config = generate_mermaid_config(TARGET_DIR)

    # Step 1: Extract Mermaid blocks
    print(f"[1/4] Extracting Mermaid diagrams from {source.name}...")
    blocks = extract_mermaid_blocks(source)
    print(f"  Found {len(blocks)} diagrams: {[b[0] for b in blocks]}\n")

    # Step 2: Render diagrams
    print("[2/4] Rendering Mermaid diagrams to PNG...")
    diagrams = render_mermaid_diagrams(blocks, DIAGRAMS_DIR, mermaid_config)
    print(f"  Rendered {len(diagrams)} diagrams\n")

    # Step 3: Load template and substitute diagrams
    print("[3/4] Generating Beamer .tex from template...")
    TARGET_DIR.mkdir(parents=True, exist_ok=True)
    template = TEMPLATE_TEX.read_text()
    template = resolve_theme_placeholders(template)
    tex_content = resolve_placeholders(template, diagrams)
    OUTPUT_TEX.write_text(tex_content)
    print(f"  Template: {TEMPLATE_TEX}")
    print(f"  Output:   {OUTPUT_TEX}\n")

    # Step 4: Compile to PDF
    print("[4/4] Compiling with tectonic...")
    compile_tex(OUTPUT_TEX)
    print(f"\n  Output: {OUTPUT_PDF}")
    print("\nDone!")


if __name__ == "__main__":
    main()
