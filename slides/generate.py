#!/usr/bin/env python3
"""Generate ForgeMaster pitch deck PDF from docs/pitch-deck.md.

Automatically extracts Mermaid diagrams, renders them to PNG via mmdc,
and builds a themed 16:9 PDF presentation using fpdf2.
"""

import argparse
from pathlib import Path

from fpdf import FPDF
from PIL import Image

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
TARGET_DIR = PROJECT_ROOT / "target" / "slides"
DIAGRAMS_DIR = TARGET_DIR / "diagrams"
OUTPUT_PDF = TARGET_DIR / "forgemaster-pitch.pdf"

# -- Theme (RGB tuples) -----------------------------------------------------
BG_COLOR = hex_to_rgb(theme["bg"])
TITLE_COLOR = hex_to_rgb(theme["title"])
ACCENT_COLOR = hex_to_rgb(theme["accent"])
BODY_COLOR = hex_to_rgb(theme["body"])
MUTED_COLOR = hex_to_rgb(theme["muted"])
CODE_BG = hex_to_rgb(theme["code_bg"])
TABLE_HEADER_BG = hex_to_rgb(theme["table_header"])
TABLE_ROW_BG = hex_to_rgb(theme["table_row"])
TABLE_ALT_BG = hex_to_rgb(theme["table_alt"])

# Slide dimensions (16:9 in mm)
SLIDE_W = 338.67  # ~13.33 inches
SLIDE_H = 190.50  # ~7.5 inches

FONT_TITLE = "Helvetica"
FONT_BODY = "Helvetica"


class PitchDeck(FPDF):
    """16:9 themed PDF presentation."""

    def __init__(self):
        super().__init__(orientation="L", unit="mm", format=(SLIDE_H, SLIDE_W))
        self.set_auto_page_break(auto=False)

    def dark_bg(self):
        """Fill the current page with background color."""
        self.set_fill_color(*BG_COLOR)
        self.rect(0, 0, SLIDE_W, SLIDE_H, "F")

    def add_slide(self):
        """Add a new slide with background."""
        self.add_page()
        self.dark_bg()

    def slide_title(self, text: str, y: float = 12):
        """Draw a slide title."""
        self.set_font(FONT_TITLE, "B", 28)
        self.set_text_color(*TITLE_COLOR)
        self.set_xy(25, y)
        self.cell(SLIDE_W - 50, 15, text, align="L")

    def slide_subtitle(self, text: str, y: float = 30):
        """Draw a subtitle line."""
        self.set_font(FONT_BODY, "", 16)
        self.set_text_color(*ACCENT_COLOR)
        self.set_xy(25, y)
        self.cell(SLIDE_W - 50, 10, text, align="L")

    def body_text(self, text: str, x: float, y: float, size: int = 14, bold: bool = False):
        """Draw body text at a position."""
        style = "B" if bold else ""
        self.set_font(FONT_BODY, style, size)
        self.set_text_color(*BODY_COLOR)
        self.set_xy(x, y)
        self.multi_cell(SLIDE_W - x - 25, size * 0.6, text)

    def accent_text(self, text: str, x: float, y: float, size: int = 14, bold: bool = False):
        """Draw accent-colored text."""
        style = "B" if bold else ""
        self.set_font(FONT_BODY, style, size)
        self.set_text_color(*ACCENT_COLOR)
        self.set_xy(x, y)
        self.multi_cell(SLIDE_W - x - 25, size * 0.6, text)

    def bullet(self, text: str, x: float, y: float, size: int = 13):
        """Draw a bullet point."""
        self.set_font(FONT_BODY, "", size)
        self.set_text_color(*ACCENT_COLOR)
        self.set_xy(x, y)
        self.cell(6, size * 0.55, ">")
        self.set_text_color(*BODY_COLOR)
        self.cell(0, size * 0.55, f"  {text}")

    def embed_diagram(self, png_path: Path, x: float, y: float,
                      max_w: float, max_h: float):
        """Embed a diagram PNG, scaled to fit within max_w x max_h."""
        img = Image.open(png_path)
        img_w, img_h = img.size
        scale = min(max_w / img_w, max_h / img_h)
        w = img_w * scale
        h = img_h * scale
        cx = x + (max_w - w) / 2
        self.image(str(png_path), cx, y, w, h)

    def draw_table(self, headers: list[str], rows: list[list[str]],
                   x: float, y: float, col_widths: list[float],
                   row_height: float = 10):
        """Draw a styled table."""
        self.set_font(FONT_BODY, "B", 12)
        self.set_fill_color(*TABLE_HEADER_BG)
        self.set_text_color(*ACCENT_COLOR)
        for i, header in enumerate(headers):
            self.set_xy(x + sum(col_widths[:i]), y)
            self.cell(col_widths[i], row_height, f"  {header}", fill=True)

        self.set_font(FONT_BODY, "", 11)
        for r, row in enumerate(rows):
            ry = y + row_height * (r + 1)
            bg = TABLE_ROW_BG if r % 2 == 0 else TABLE_ALT_BG
            self.set_fill_color(*bg)
            self.set_text_color(*BODY_COLOR)
            for i, cell in enumerate(row):
                self.set_xy(x + sum(col_widths[:i]), ry)
                if i == 0:
                    self.set_font(FONT_BODY, "B", 11)
                    self.set_text_color(*ACCENT_COLOR)
                else:
                    self.set_font(FONT_BODY, "", 11)
                    self.set_text_color(*BODY_COLOR)
                self.cell(col_widths[i], row_height, f"  {cell}", fill=True)

    def code_block(self, text: str, x: float, y: float, w: float, size: int = 9):
        """Draw a code block with background."""
        self.set_fill_color(*CODE_BG)
        lines = text.strip().split("\n")
        block_h = len(lines) * (size * 0.55) + 8
        self.rect(x, y, w, block_h, "F")
        self.set_font("Courier", "", size)
        self.set_text_color(*ACCENT_COLOR)
        for i, line in enumerate(lines):
            self.set_xy(x + 4, y + 4 + i * (size * 0.55))
            self.cell(w - 8, size * 0.55, line)


def build_slide_1_title(pdf: PitchDeck):
    """Slide 1: Title."""
    pdf.add_slide()
    pdf.set_font(FONT_TITLE, "B", 52)
    pdf.set_text_color(*TITLE_COLOR)
    pdf.set_xy(0, 50)
    pdf.cell(SLIDE_W, 25, "ForgeMaster", align="C")
    pdf.set_font(FONT_BODY, "", 22)
    pdf.set_text_color(*ACCENT_COLOR)
    pdf.set_xy(0, 82)
    pdf.cell(SLIDE_W, 12, "Autonomous Agent Orchestration for Kubernetes", align="C")
    pdf.set_font(FONT_BODY, "", 14)
    pdf.set_text_color(*MUTED_COLOR)
    pdf.set_xy(0, 145)
    pdf.cell(SLIDE_W, 10, "AgentForge Hackathon 2026", align="C")
    pdf.set_xy(0, 155)
    pdf.cell(SLIDE_W, 10, "Dmytro Shcherbatiuk  |  Team CSM-101", align="C")


def build_slide_2_problem(pdf: PitchDeck, diagrams: dict[str, Path]):
    """Slide 2: The Problem."""
    pdf.add_slide()
    pdf.slide_title("AI coding today is one-shot")

    diagram_key = "slide2_diagram0"
    if diagram_key in diagrams:
        pdf.embed_diagram(diagrams[diagram_key], 25, 35, SLIDE_W - 50, 100)

    bullets = [
        "No autonomous infrastructure",
        "No automated testing or deployment",
        "Manual agent setup and coordination",
        "Human in the loop for every step",
    ]
    y_start = 140
    for i, b in enumerate(bullets):
        pdf.bullet(b, 30, y_start + i * 12)


def build_slide_3_solution(pdf: PitchDeck, diagrams: dict[str, Path]):
    """Slide 3: The Solution."""
    pdf.add_slide()
    pdf.slide_title("ForgeMaster: Kubernetes manages the agents")

    diagram_key = "slide3_diagram0"
    if diagram_key in diagrams:
        pdf.embed_diagram(diagrams[diagram_key], 25, 35, SLIDE_W - 50, 90)

    bullets = [
        "AgentTask CRD -user's intent as a Kubernetes resource",
        "Agent CRD -each agent is a managed K8s object",
        "Operators handle creation, health, scaling, cleanup",
        "No manual setup -infrastructure IS the orchestrator",
    ]
    y_start = 135
    for i, b in enumerate(bullets):
        pdf.bullet(b, 30, y_start + i * 12)


def build_slide_4_protocols(pdf: PitchDeck, diagrams: dict[str, Path]):
    """Slide 4: Three Protocols."""
    pdf.add_slide()
    pdf.slide_title("Standards-based, not reinvented")

    headers = ["Protocol", "Purpose", "Standard"]
    rows = [
        ["MCP", "Agent-to-Tools", "Anthropic"],
        ["A2A", "Agent-to-Agent", "Google / Linux Foundation"],
        ["A2UI", "Agent-to-User", "Google"],
    ]
    pdf.draw_table(headers, rows, 25, 35, [50, 80, 100])

    diagram_key = "slide4_diagram0"
    if diagram_key in diagrams:
        pdf.embed_diagram(diagrams[diagram_key], 25, 85, SLIDE_W - 50, 95)


def build_slide_5_pipeline(pdf: PitchDeck, diagrams: dict[str, Path]):
    """Slide 5: Agent Pipeline."""
    pdf.add_slide()
    pdf.slide_title("Six agents, self-coordinating via A2A")

    diagram_key = "slide5_diagram0"
    if diagram_key in diagrams:
        pdf.embed_diagram(diagrams[diagram_key], 15, 35, SLIDE_W - 30, 95)

    bullets = [
        "Own Kubernetes pod",
        "Own Claude API session",
        "Own MCP tools (filesystem, Docker, Helm)",
        "A2A endpoint for peer communication",
    ]
    y_start = 140
    for i, b in enumerate(bullets):
        pdf.bullet(b, 30, y_start + i * 12)


def build_slide_6_k8s(pdf: PitchDeck, diagrams: dict[str, Path]):
    """Slide 6: Kubernetes-Native CRDs."""
    pdf.add_slide()
    pdf.slide_title("Two Custom Resource Definitions")

    agenttask_yaml = """apiVersion: forgemaster.io/v1alpha1
kind: AgentTask
spec:
  description: "Build a REST API..."
status:
  phase: Running
  error: 0.35"""

    agent_yaml = """apiVersion: forgemaster.io/v1alpha1
kind: Agent
spec:
  type: code-generator
  model:
    name: claude-sonnet-4-20250514
    maxTokens: 16384
  mcpServers:
    - name: fm-mcp-filesystem
    - name: fm-mcp-devtools"""

    pdf.code_block(agenttask_yaml, 25, 35, 140)
    pdf.code_block(agent_yaml, 175, 35, 140)

    diagram_key = "slide6_diagram0"
    if diagram_key in diagrams:
        pdf.embed_diagram(diagrams[diagram_key], 25, 105, SLIDE_W - 50, 75)


def build_slide_7_demo(pdf: PitchDeck):
    """Slide 7: DEMO placeholder."""
    pdf.add_slide()
    pdf.set_font(FONT_TITLE, "B", 48)
    pdf.set_text_color(*ACCENT_COLOR)
    pdf.set_xy(0, 65)
    pdf.cell(SLIDE_W, 25, "LIVE DEMO", align="C")
    pdf.set_font(FONT_BODY, "", 18)
    pdf.set_text_color(*MUTED_COLOR)
    pdf.set_xy(0, 100)
    pdf.cell(SLIDE_W, 12, "kubectl apply -f agenttask.yaml", align="C")
    pdf.set_xy(0, 116)
    pdf.cell(SLIDE_W, 12, "Watch agents spin up, code, deploy, and test autonomously", align="C")


def build_slide_8_engineering(pdf: PitchDeck):
    """Slide 8: Engineering Quality."""
    pdf.add_slide()
    pdf.slide_title("Production-grade, not a prototype")

    headers = ["", ""]
    rows = [
        ["Language", "Rust (5 crates, single responsibility each)"],
        ["Orchestration", "Kubernetes CRDs + Operators"],
        ["Deployment", "Helm charts + Ansible"],
        ["Environments", "Local (OrbStack) / Remote (ghcr.io)"],
        ["CI/CD", "GitHub Actions to ghcr.io"],
        ["Documentation", "10 ADRs, architecture docs"],
        ["Observability", "Full audit trail per agent conversation"],
    ]
    pdf.draw_table(headers, rows, 40, 42, [80, 200], row_height=16)


def build_slide_9_why_win(pdf: PitchDeck):
    """Slide 9: Why ForgeMaster Should Win."""
    pdf.add_slide()
    pdf.slide_title("Why ForgeMaster Should Win")

    items = [
        ("1. Standards-based", "A2A + MCP + A2UI -composing open protocols, not reinventing them.\nAny MCP server, any A2A agent can plug in."),
        ("2. Kubernetes-native", "CRDs, operators, namespace isolation, Helm, RBAC.\nScales. Observable. Production-ready."),
        ("3. Control theory meets AI", "TCP controller: microseconds, zero cost, deterministic.\nLLM budget reserved for creative work only."),
    ]

    y = 42
    for title, desc in items:
        pdf.accent_text(title, 30, y, size=18, bold=True)
        y += 14
        pdf.body_text(desc, 35, y, size=13)
        y += 30


def build_slide_10_close(pdf: PitchDeck):
    """Slide 10: Close."""
    pdf.add_slide()
    pdf.set_font(FONT_TITLE, "B", 48)
    pdf.set_text_color(*TITLE_COLOR)
    pdf.set_xy(0, 45)
    pdf.cell(SLIDE_W, 25, "ForgeMaster", align="C")

    lines = [
        "Autonomous agent orchestration,",
        "powered by control theory,",
        "built for Kubernetes.",
    ]
    pdf.set_font(FONT_BODY, "", 20)
    pdf.set_text_color(*ACCENT_COLOR)
    for i, line in enumerate(lines):
        pdf.set_xy(0, 80 + i * 14)
        pdf.cell(SLIDE_W, 12, line, align="C")

    pdf.set_font(FONT_BODY, "", 14)
    pdf.set_text_color(*MUTED_COLOR)
    pdf.set_xy(0, 145)
    pdf.cell(SLIDE_W, 10, "github.com/dshcherbatiuk/forgemaster", align="C")
    pdf.set_xy(0, 158)
    pdf.cell(SLIDE_W, 10, "Dmytro Shcherbatiuk  |  Team CSM-101", align="C")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Generate ForgeMaster pitch deck PDF")
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

    print("=== ForgeMaster Pitch Deck Generator ===\n")

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

    # Step 3: Build PDF
    print("[3/3] Building PDF presentation...")
    pdf = PitchDeck()

    build_slide_1_title(pdf)
    build_slide_2_problem(pdf, diagrams)
    build_slide_3_solution(pdf, diagrams)
    build_slide_4_protocols(pdf, diagrams)
    build_slide_5_pipeline(pdf, diagrams)
    build_slide_6_k8s(pdf, diagrams)
    build_slide_7_demo(pdf)
    build_slide_8_engineering(pdf)
    build_slide_9_why_win(pdf)
    build_slide_10_close(pdf)

    pdf.output(str(OUTPUT_PDF))
    print(f"\n  Output: {OUTPUT_PDF}")
    print(f"  Slides: {pdf.page}")
    print("\nDone!")


if __name__ == "__main__":
    main()
