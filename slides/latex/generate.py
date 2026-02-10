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
import tempfile
from pathlib import Path

# -- Paths ------------------------------------------------------------------
SCRIPT_DIR = Path(__file__).parent
SLIDES_DIR = SCRIPT_DIR.parent
PROJECT_ROOT = SLIDES_DIR.parent
DEFAULT_PITCH_DECK_MD = PROJECT_ROOT / "docs" / "pitch-deck.md"
MERMAID_CONFIG = SLIDES_DIR / "mermaid-config.json"
TEMPLATE_TEX = SCRIPT_DIR / "template.tex"
TARGET_DIR = PROJECT_ROOT / "target" / "slides-latex"
DIAGRAMS_DIR = TARGET_DIR / "diagrams"
OUTPUT_TEX = TARGET_DIR / "forgemaster-pitch.tex"
OUTPUT_PDF = TARGET_DIR / "forgemaster-pitch.pdf"

# Placeholder pattern: {{DIAGRAM_<id>_<width>}}
PLACEHOLDER_RE = re.compile(r"\{\{DIAGRAM_(\w+)_([\d.]+)\}\}")


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


def render_mermaid_diagrams(blocks: list[tuple[str, str]]) -> dict[str, Path]:
    """Render mermaid blocks to PNG files using mmdc."""
    DIAGRAMS_DIR.mkdir(parents=True, exist_ok=True)
    rendered = {}

    for block_id, code in blocks:
        png_path = DIAGRAMS_DIR / f"{block_id}.png"
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
                "-c", str(MERMAID_CONFIG),
                "-b", "#0a1628",
                "-w", "1600",
                "-s", "2",
            ]
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=60)
            if result.returncode != 0:
                print(f"  [WARN] {block_id}: {result.stderr[:200]}")
                continue
            print(f"  [rendered] {block_id}")
            rendered[block_id] = png_path
        finally:
            os.unlink(mmd_path)

    return rendered


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

    # Step 1: Extract Mermaid blocks
    print(f"[1/4] Extracting Mermaid diagrams from {source.name}...")
    blocks = extract_mermaid_blocks(source)
    print(f"  Found {len(blocks)} diagrams: {[b[0] for b in blocks]}\n")

    # Step 2: Render diagrams
    print("[2/4] Rendering Mermaid diagrams to PNG...")
    diagrams = render_mermaid_diagrams(blocks)
    print(f"  Rendered {len(diagrams)} diagrams\n")

    # Step 3: Load template and substitute diagrams
    print("[3/4] Generating Beamer .tex from template...")
    TARGET_DIR.mkdir(parents=True, exist_ok=True)
    template = TEMPLATE_TEX.read_text()
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
