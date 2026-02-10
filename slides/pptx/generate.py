#!/usr/bin/env python3
"""Generate ForgeMaster pitch deck as .pptx for Google Slides.

Extracts Mermaid diagrams from markdown, renders them to PNG via mmdc,
and builds a themed 16:9 PowerPoint presentation using python-pptx.
"""

import argparse
import sys
from pathlib import Path

from pptx import Presentation
from pptx.dml.color import RGBColor
from pptx.enum.text import PP_ALIGN
from pptx.util import Emu, Inches, Pt

# Allow importing shared module from parent directory
sys.path.insert(0, str(Path(__file__).parent.parent))

from shared import (
    DEFAULT_PITCH_DECK_MD,
    PROJECT_ROOT,
    extract_mermaid_blocks,
    generate_mermaid_config,
    hex_to_rgb,
    render_mermaid_diagrams,
    theme,
)

# -- Paths ------------------------------------------------------------------
TARGET_DIR = PROJECT_ROOT / "target" / "slides-pptx"
DIAGRAMS_DIR = TARGET_DIR / "diagrams"
OUTPUT_PPTX = TARGET_DIR / "forgemaster-pitch.pptx"

# -- Theme colors (RGBColor) ------------------------------------------------
BG_COLOR = RGBColor(*hex_to_rgb(theme["bg"]))
TITLE_COLOR = RGBColor(*hex_to_rgb(theme["title"]))
ACCENT_COLOR = RGBColor(*hex_to_rgb(theme["accent"]))
BODY_COLOR = RGBColor(*hex_to_rgb(theme["body"]))
MUTED_COLOR = RGBColor(*hex_to_rgb(theme["muted"]))
CODE_BG = RGBColor(*hex_to_rgb(theme["code_bg"]))
TABLE_HEADER_BG = RGBColor(*hex_to_rgb(theme["table_header"]))
TABLE_ROW_BG = RGBColor(*hex_to_rgb(theme["table_row"]))
TABLE_ALT_BG = RGBColor(*hex_to_rgb(theme["table_alt"]))

# Slide dimensions (16:9)
SLIDE_WIDTH = Inches(13.333)
SLIDE_HEIGHT = Inches(7.5)

FONT_NAME = "Calibri"


def hex_to_rgbcolor(hex_color: str) -> RGBColor:
    return RGBColor(*hex_to_rgb(hex_color))


def set_slide_bg(slide, color: RGBColor):
    """Set solid background color on a slide."""
    bg = slide.background
    fill = bg.fill
    fill.solid()
    fill.fore_color.rgb = color


def add_textbox(slide, left, top, width, height, text, *,
                font_size=Pt(14), font_color=BODY_COLOR, bold=False,
                alignment=PP_ALIGN.LEFT, font_name=FONT_NAME):
    """Add a text box with a single styled run."""
    txbox = slide.shapes.add_textbox(left, top, width, height)
    tf = txbox.text_frame
    tf.word_wrap = True
    p = tf.paragraphs[0]
    p.alignment = alignment
    run = p.add_run()
    run.text = text
    run.font.size = font_size
    run.font.color.rgb = font_color
    run.font.bold = bold
    run.font.name = font_name
    return txbox


def add_bullets(slide, left, top, width, items, *,
                font_size=Pt(16), bullet_color=ACCENT_COLOR,
                text_color=BODY_COLOR):
    """Add a bulleted text box."""
    line_height = int(font_size * 1.8)
    height = Emu(line_height * len(items))
    txbox = slide.shapes.add_textbox(left, top, width, height)
    tf = txbox.text_frame
    tf.word_wrap = True

    for i, item in enumerate(items):
        p = tf.paragraphs[0] if i == 0 else tf.add_paragraph()
        p.alignment = PP_ALIGN.LEFT
        # Bullet character
        bullet_run = p.add_run()
        bullet_run.text = ">  "
        bullet_run.font.size = font_size
        bullet_run.font.color.rgb = bullet_color
        bullet_run.font.name = FONT_NAME
        bullet_run.font.bold = True
        # Item text
        text_run = p.add_run()
        text_run.text = item
        text_run.font.size = font_size
        text_run.font.color.rgb = text_color
        text_run.font.name = FONT_NAME

    return txbox


def add_diagram(slide, png_path: Path, left, top, max_width, max_height):
    """Add a diagram image, scaled to fit within bounds."""
    from PIL import Image
    img = Image.open(png_path)
    img_w, img_h = img.size

    scale = min(max_width / img_w, max_height / img_h)
    w = int(img_w * scale)
    h = int(img_h * scale)

    # Center horizontally
    cx = left + (max_width - w) // 2
    slide.shapes.add_picture(str(png_path), cx, top, w, h)


def add_styled_table(slide, left, top, width, headers, rows, col_widths_pct):
    """Add a styled table with header and alternating row colors."""
    row_count = len(rows) + 1
    col_count = len(headers)
    table_height = Inches(0.4) * row_count

    shape = slide.shapes.add_table(row_count, col_count, left, top, width, table_height)
    table = shape.table

    # Set column widths
    total = sum(col_widths_pct)
    for i, pct in enumerate(col_widths_pct):
        table.columns[i].width = int(width * pct / total)

    # Header row
    for i, header in enumerate(headers):
        cell = table.cell(0, i)
        cell.text = header
        for p in cell.text_frame.paragraphs:
            for run in p.runs:
                run.font.size = Pt(14)
                run.font.color.rgb = ACCENT_COLOR
                run.font.bold = True
                run.font.name = FONT_NAME
        cell.fill.solid()
        cell.fill.fore_color.rgb = TABLE_HEADER_BG

    # Data rows
    for r, row in enumerate(rows):
        bg = TABLE_ROW_BG if r % 2 == 0 else TABLE_ALT_BG
        for c, value in enumerate(row):
            cell = table.cell(r + 1, c)
            cell.text = value
            cell.fill.solid()
            cell.fill.fore_color.rgb = bg
            for p in cell.text_frame.paragraphs:
                for run in p.runs:
                    run.font.size = Pt(13)
                    run.font.name = FONT_NAME
                    if c == 0:
                        run.font.color.rgb = ACCENT_COLOR
                        run.font.bold = True
                    else:
                        run.font.color.rgb = BODY_COLOR

    return shape


def add_code_block(slide, left, top, width, height, code_text):
    """Add a code block with monospace font and background."""
    txbox = slide.shapes.add_textbox(left, top, width, height)
    tf = txbox.text_frame
    tf.word_wrap = True

    # Background fill
    txbox.fill.solid()
    txbox.fill.fore_color.rgb = CODE_BG

    lines = code_text.strip().split("\n")
    for i, line in enumerate(lines):
        p = tf.paragraphs[0] if i == 0 else tf.add_paragraph()
        run = p.add_run()
        run.text = line
        run.font.size = Pt(11)
        run.font.color.rgb = ACCENT_COLOR
        run.font.name = "Courier New"

    return txbox


# -- Slide builders ---------------------------------------------------------

def build_slide_1_title(prs: Presentation):
    """Slide 1: Title."""
    slide = prs.slides.add_slide(prs.slide_layouts[6])  # Blank
    set_slide_bg(slide, BG_COLOR)

    add_textbox(slide, Inches(0), Inches(1.5), SLIDE_WIDTH, Inches(1.2),
                "ForgeMaster",
                font_size=Pt(54), font_color=TITLE_COLOR, bold=True,
                alignment=PP_ALIGN.CENTER)

    add_textbox(slide, Inches(0), Inches(2.8), SLIDE_WIDTH, Inches(0.8),
                "Autonomous Agent Orchestration for Kubernetes",
                font_size=Pt(24), font_color=ACCENT_COLOR,
                alignment=PP_ALIGN.CENTER)

    add_textbox(slide, Inches(0), Inches(5.2), SLIDE_WIDTH, Inches(0.5),
                "AgentForge Hackathon 2026",
                font_size=Pt(16), font_color=MUTED_COLOR,
                alignment=PP_ALIGN.CENTER)

    add_textbox(slide, Inches(0), Inches(5.7), SLIDE_WIDTH, Inches(0.5),
                "Dmytro Shcherbatiuk  |  Team CSM-101",
                font_size=Pt(16), font_color=MUTED_COLOR,
                alignment=PP_ALIGN.CENTER)


def build_slide_2_problem(prs: Presentation, diagrams: dict[str, Path]):
    """Slide 2: The Problem."""
    slide = prs.slides.add_slide(prs.slide_layouts[6])
    set_slide_bg(slide, BG_COLOR)

    add_textbox(slide, Inches(0.8), Inches(0.3), Inches(11), Inches(0.7),
                "AI coding today is one-shot",
                font_size=Pt(32), font_color=TITLE_COLOR, bold=True)

    diagram_key = "slide2_diagram0"
    if diagram_key in diagrams:
        add_diagram(slide, diagrams[diagram_key],
                    Inches(0.8), Inches(1.2), Inches(11.5), Inches(3.5))

    add_bullets(slide, Inches(1), Inches(5.0), Inches(10), [
        "No autonomous infrastructure",
        "No automated testing or deployment",
        "Manual agent setup and coordination",
        "Human in the loop for every step",
    ])


def build_slide_3_solution(prs: Presentation, diagrams: dict[str, Path]):
    """Slide 3: The Solution."""
    slide = prs.slides.add_slide(prs.slide_layouts[6])
    set_slide_bg(slide, BG_COLOR)

    add_textbox(slide, Inches(0.8), Inches(0.3), Inches(11), Inches(0.7),
                "ForgeMaster: Kubernetes manages the agents",
                font_size=Pt(32), font_color=TITLE_COLOR, bold=True)

    diagram_key = "slide3_diagram0"
    if diagram_key in diagrams:
        add_diagram(slide, diagrams[diagram_key],
                    Inches(0.8), Inches(1.2), Inches(11.5), Inches(3.2))

    add_bullets(slide, Inches(1), Inches(4.8), Inches(10), [
        "AgentTask CRD - user's intent as a Kubernetes resource",
        "Agent CRD - each agent is a managed K8s object",
        "Operators handle creation, health, scaling, cleanup",
        "No manual setup - infrastructure IS the orchestrator",
    ])


def build_slide_4_protocols(prs: Presentation, diagrams: dict[str, Path]):
    """Slide 4: Three Protocols."""
    slide = prs.slides.add_slide(prs.slide_layouts[6])
    set_slide_bg(slide, BG_COLOR)

    add_textbox(slide, Inches(0.8), Inches(0.3), Inches(11), Inches(0.7),
                "Standards-based, not reinvented",
                font_size=Pt(32), font_color=TITLE_COLOR, bold=True)

    add_styled_table(
        slide, Inches(0.8), Inches(1.2), Inches(8),
        ["Protocol", "Purpose", "Standard"],
        [
            ["MCP", "Agent-to-Tools", "Anthropic"],
            ["A2A", "Agent-to-Agent", "Google / Linux Foundation"],
            ["A2UI", "Agent-to-User", "Google"],
        ],
        [20, 35, 45],
    )

    diagram_key = "slide4_diagram0"
    if diagram_key in diagrams:
        add_diagram(slide, diagrams[diagram_key],
                    Inches(0.8), Inches(3.5), Inches(11.5), Inches(3.5))


def build_slide_5_pipeline(prs: Presentation, diagrams: dict[str, Path]):
    """Slide 5: Agent Pipeline."""
    slide = prs.slides.add_slide(prs.slide_layouts[6])
    set_slide_bg(slide, BG_COLOR)

    add_textbox(slide, Inches(0.8), Inches(0.3), Inches(11), Inches(0.7),
                "Six agents, self-coordinating via A2A",
                font_size=Pt(32), font_color=TITLE_COLOR, bold=True)

    diagram_key = "slide5_diagram0"
    if diagram_key in diagrams:
        add_diagram(slide, diagrams[diagram_key],
                    Inches(0.5), Inches(1.2), Inches(12), Inches(3.5))

    add_bullets(slide, Inches(1), Inches(5.0), Inches(10), [
        "Own Kubernetes pod",
        "Own Claude API session",
        "Own MCP tools (filesystem, Docker, Helm)",
        "A2A endpoint for peer communication",
    ])


def build_slide_6_k8s(prs: Presentation, diagrams: dict[str, Path]):
    """Slide 6: Kubernetes-Native CRDs."""
    slide = prs.slides.add_slide(prs.slide_layouts[6])
    set_slide_bg(slide, BG_COLOR)

    add_textbox(slide, Inches(0.8), Inches(0.3), Inches(11), Inches(0.7),
                "Two Custom Resource Definitions",
                font_size=Pt(32), font_color=TITLE_COLOR, bold=True)

    agenttask_yaml = """\
apiVersion: forgemaster.io/v1alpha1
kind: AgentTask
spec:
  description: "Build a REST API..."
status:
  phase: Running
  error: 0.35"""

    agent_yaml = """\
apiVersion: forgemaster.io/v1alpha1
kind: Agent
spec:
  type: code-generator
  model:
    name: claude-sonnet-4-20250514
    maxTokens: 16384
  mcpServers:
    - name: fm-mcp-filesystem
    - name: fm-mcp-devtools"""

    add_code_block(slide, Inches(0.8), Inches(1.2), Inches(5.5), Inches(2.5), agenttask_yaml)
    add_code_block(slide, Inches(6.8), Inches(1.2), Inches(5.8), Inches(2.5), agent_yaml)

    diagram_key = "slide6_diagram0"
    if diagram_key in diagrams:
        add_diagram(slide, diagrams[diagram_key],
                    Inches(0.8), Inches(4.0), Inches(11.5), Inches(3))


def build_slide_7_demo(prs: Presentation):
    """Slide 7: DEMO placeholder."""
    slide = prs.slides.add_slide(prs.slide_layouts[6])
    set_slide_bg(slide, BG_COLOR)

    add_textbox(slide, Inches(0), Inches(2.2), SLIDE_WIDTH, Inches(1.2),
                "LIVE DEMO",
                font_size=Pt(54), font_color=ACCENT_COLOR, bold=True,
                alignment=PP_ALIGN.CENTER)

    add_textbox(slide, Inches(0), Inches(3.8), SLIDE_WIDTH, Inches(0.6),
                "kubectl apply -f agenttask.yaml",
                font_size=Pt(20), font_color=MUTED_COLOR,
                alignment=PP_ALIGN.CENTER, font_name="Courier New")

    add_textbox(slide, Inches(0), Inches(4.5), SLIDE_WIDTH, Inches(0.6),
                "Watch agents spin up, code, deploy, and test autonomously",
                font_size=Pt(18), font_color=MUTED_COLOR,
                alignment=PP_ALIGN.CENTER)


def build_slide_8_engineering(prs: Presentation):
    """Slide 8: Engineering Quality."""
    slide = prs.slides.add_slide(prs.slide_layouts[6])
    set_slide_bg(slide, BG_COLOR)

    add_textbox(slide, Inches(0.8), Inches(0.3), Inches(11), Inches(0.7),
                "Production-grade, not a prototype",
                font_size=Pt(32), font_color=TITLE_COLOR, bold=True)

    add_styled_table(
        slide, Inches(1.2), Inches(1.4), Inches(10),
        ["Aspect", "Detail"],
        [
            ["Language", "Rust (5 crates, single responsibility each)"],
            ["Orchestration", "Kubernetes CRDs + Operators"],
            ["Deployment", "Helm charts + Ansible"],
            ["Environments", "Local (OrbStack) / Remote (ghcr.io)"],
            ["CI/CD", "GitHub Actions to ghcr.io"],
            ["Documentation", "10 ADRs, architecture docs"],
            ["Observability", "Full audit trail per agent conversation"],
        ],
        [25, 75],
    )


def build_slide_9_why_win(prs: Presentation):
    """Slide 9: Why ForgeMaster Should Win."""
    slide = prs.slides.add_slide(prs.slide_layouts[6])
    set_slide_bg(slide, BG_COLOR)

    add_textbox(slide, Inches(0.8), Inches(0.3), Inches(11), Inches(0.7),
                "Why ForgeMaster Should Win",
                font_size=Pt(32), font_color=TITLE_COLOR, bold=True)

    items = [
        ("1. Standards-based",
         "A2A + MCP + A2UI - composing open protocols, not reinventing them.\nAny MCP server, any A2A agent can plug in."),
        ("2. Kubernetes-native",
         "CRDs, operators, namespace isolation, Helm, RBAC.\nScales. Observable. Production-ready."),
        ("3. Control theory meets AI",
         "TCP controller: microseconds, zero cost, deterministic.\nLLM budget reserved for creative work only."),
    ]

    y = 1.4
    for title, desc in items:
        add_textbox(slide, Inches(1), Inches(y), Inches(10), Inches(0.5),
                    title,
                    font_size=Pt(22), font_color=ACCENT_COLOR, bold=True)
        y += 0.5
        add_textbox(slide, Inches(1.2), Inches(y), Inches(10), Inches(0.8),
                    desc,
                    font_size=Pt(16), font_color=BODY_COLOR)
        y += 1.2


def build_slide_10_close(prs: Presentation):
    """Slide 10: Close."""
    slide = prs.slides.add_slide(prs.slide_layouts[6])
    set_slide_bg(slide, BG_COLOR)

    add_textbox(slide, Inches(0), Inches(1.2), SLIDE_WIDTH, Inches(1.2),
                "ForgeMaster",
                font_size=Pt(54), font_color=TITLE_COLOR, bold=True,
                alignment=PP_ALIGN.CENTER)

    lines = [
        "Autonomous agent orchestration,",
        "powered by control theory,",
        "built for Kubernetes.",
    ]
    y = 2.8
    for line in lines:
        add_textbox(slide, Inches(0), Inches(y), SLIDE_WIDTH, Inches(0.5),
                    line,
                    font_size=Pt(22), font_color=ACCENT_COLOR,
                    alignment=PP_ALIGN.CENTER)
        y += 0.5

    add_textbox(slide, Inches(0), Inches(5.2), SLIDE_WIDTH, Inches(0.5),
                "github.com/dshcherbatiuk/forgemaster",
                font_size=Pt(16), font_color=MUTED_COLOR,
                alignment=PP_ALIGN.CENTER)

    add_textbox(slide, Inches(0), Inches(5.7), SLIDE_WIDTH, Inches(0.5),
                "Dmytro Shcherbatiuk  |  Team CSM-101",
                font_size=Pt(16), font_color=MUTED_COLOR,
                alignment=PP_ALIGN.CENTER)


# -- Main -------------------------------------------------------------------

def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Generate ForgeMaster pitch deck .pptx")
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

    print("=== ForgeMaster Pitch Deck Generator (PPTX) ===\n")

    # Step 0: Generate mermaid config from theme
    mermaid_config = generate_mermaid_config(TARGET_DIR)

    # Step 1: Extract Mermaid blocks
    print(f"[1/3] Extracting Mermaid diagrams from {source.name}...")
    blocks = extract_mermaid_blocks(source)
    print(f"  Found {len(blocks)} diagrams: {[b[0] for b in blocks]}\n")

    # Step 2: Render diagrams
    print("[2/3] Rendering Mermaid diagrams to PNG...")
    diagrams = render_mermaid_diagrams(blocks, DIAGRAMS_DIR, mermaid_config)
    print(f"  Rendered {len(diagrams)} diagrams\n")

    # Step 3: Build PPTX
    print("[3/3] Building PowerPoint presentation...")
    TARGET_DIR.mkdir(parents=True, exist_ok=True)

    prs = Presentation()
    prs.slide_width = SLIDE_WIDTH
    prs.slide_height = SLIDE_HEIGHT

    build_slide_1_title(prs)
    build_slide_2_problem(prs, diagrams)
    build_slide_3_solution(prs, diagrams)
    build_slide_4_protocols(prs, diagrams)
    build_slide_5_pipeline(prs, diagrams)
    build_slide_6_k8s(prs, diagrams)
    build_slide_7_demo(prs)
    build_slide_8_engineering(prs)
    build_slide_9_why_win(prs)
    build_slide_10_close(prs)

    prs.save(str(OUTPUT_PPTX))
    print(f"\n  Output: {OUTPUT_PPTX}")
    print(f"  Slides: {len(prs.slides)}")
    print("\nDone!")


if __name__ == "__main__":
    main()
