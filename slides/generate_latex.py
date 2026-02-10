#!/usr/bin/env python3
"""Generate ForgeMaster pitch deck PDF via LaTeX Beamer.

Extracts Mermaid diagrams from markdown, renders them to PNG via mmdc,
generates a Beamer .tex file, and compiles to PDF using tectonic.
"""

import argparse
import os
import re
import subprocess
import tempfile
from pathlib import Path

# -- Paths ------------------------------------------------------------------
SCRIPT_DIR = Path(__file__).parent
PROJECT_ROOT = SCRIPT_DIR.parent
DEFAULT_PITCH_DECK_MD = PROJECT_ROOT / "docs" / "pitch-deck.md"
MERMAID_CONFIG = SCRIPT_DIR / "mermaid-config.json"
TARGET_DIR = PROJECT_ROOT / "target" / "slides-latex"
DIAGRAMS_DIR = TARGET_DIR / "diagrams"
OUTPUT_TEX = TARGET_DIR / "forgemaster-pitch.tex"
OUTPUT_PDF = TARGET_DIR / "forgemaster-pitch.pdf"


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


def diagram_cmd(diagrams: dict[str, Path], key: str, width: str = "0.9\\\\linewidth") -> str:
    """Return LaTeX includegraphics command if diagram exists."""
    if key not in diagrams:
        return ""
    return f"\\includegraphics[width={width}]{{{diagrams[key]}}}"


def generate_tex(diagrams: dict[str, Path]) -> str:
    """Generate the Beamer LaTeX source."""

    # Helper to get diagram path for LaTeX (relative to .tex file)
    def img(key: str, width: str = "0.9\\linewidth") -> str:
        if key not in diagrams:
            return ""
        rel_path = os.path.relpath(diagrams[key], TARGET_DIR).replace("\\", "/")
        return f"\\includegraphics[width={width}]{{{rel_path}}}"

    def img_block(key: str, width: str = "0.9\\linewidth") -> str:
        i = img(key, width)
        if not i:
            return ""
        return f"\\begin{{center}}\n{i}\n\\end{{center}}"

    tex = r"""\PassOptionsToPackage{table}{xcolor}
\documentclass[aspectratio=169,14pt]{beamer}

% -- Theme ------------------------------------------------------------------
\usetheme{default}
\usecolortheme{default}

\definecolor{bgdark}{HTML}{0A1628}
\definecolor{accent}{HTML}{5BBCFF}
\definecolor{bodytext}{HTML}{DCDCDC}
\definecolor{muted}{HTML}{8CA0B4}
\definecolor{codebg}{HTML}{0F1E32}
\definecolor{tablehead}{HTML}{1A3A5C}
\definecolor{tablerow}{HTML}{0F2337}
\definecolor{tablealt}{HTML}{142A41}

\setbeamercolor{background canvas}{bg=bgdark}
\setbeamercolor{normal text}{fg=bodytext}
\setbeamercolor{frametitle}{fg=white}
\setbeamercolor{title}{fg=white}
\setbeamercolor{subtitle}{fg=accent}
\setbeamercolor{author}{fg=muted}
\setbeamercolor{date}{fg=muted}
\setbeamercolor{item}{fg=accent}
\setbeamercolor{itemize item}{fg=accent}
\setbeamercolor{itemize subitem}{fg=accent}

\setbeamerfont{title}{size=\Huge,series=\bfseries}
\setbeamerfont{subtitle}{size=\Large}
\setbeamerfont{frametitle}{size=\Large,series=\bfseries}

\setbeamertemplate{navigation symbols}{}
\setbeamertemplate{footline}{}
\setbeamertemplate{itemize items}{$\rangle$}

\usepackage{graphicx}
\usepackage{listings}
\usepackage{booktabs}
\usepackage{colortbl}
\usepackage{array}

\lstdefinestyle{yaml}{
  basicstyle=\ttfamily\scriptsize\color{accent},
  backgroundcolor=\color{codebg},
  frame=single,
  rulecolor=\color{codebg},
  framesep=4pt,
  xleftmargin=6pt,
  xrightmargin=6pt,
  breaklines=true,
  showstringspaces=false,
}

\begin{document}

% ============================================================================
% Slide 1: Title
% ============================================================================
{
\begin{frame}[plain]
\vfill
\begin{center}
{\usebeamerfont{title}\usebeamercolor[fg]{title}ForgeMaster}\\[12pt]
{\usebeamerfont{subtitle}\usebeamercolor[fg]{subtitle}Autonomous Agent Orchestration for Kubernetes}\\[60pt]
{\small\color{muted}AgentForge Hackathon 2026}\\[4pt]
{\small\color{muted}Dmytro Shcherbatiuk\enspace|\enspace Team CSM-101}
\end{center}
\vfill
\end{frame}
}

% ============================================================================
% Slide 2: The Problem
% ============================================================================
\begin{frame}{AI coding today is one-shot}
""" + img_block("slide2_diagram0", "0.65\\linewidth") + r"""
\begin{itemize}
  \item No autonomous infrastructure
  \item No automated testing or deployment
  \item Manual agent setup and coordination
  \item Human in the loop for every step
\end{itemize}
\end{frame}

% ============================================================================
% Slide 3: The Solution
% ============================================================================
\begin{frame}{ForgeMaster: Kubernetes manages the agents}
""" + img_block("slide3_diagram0") + r"""
\begin{itemize}
  \item \textcolor{accent}{AgentTask CRD} -- user's intent as a Kubernetes resource
  \item \textcolor{accent}{Agent CRD} -- each agent is a managed K8s object
  \item Operators handle creation, health, scaling, cleanup
  \item No manual setup -- infrastructure IS the orchestrator
\end{itemize}
\end{frame}

% ============================================================================
% Slide 4: Three Protocols
% ============================================================================
\begin{frame}{Standards-based, not reinvented}
\vspace{-4pt}
\begin{center}
\small
\rowcolors{2}{tablerow}{tablealt}
\begin{tabular}{>{\color{accent}\bfseries}l l l}
\rowcolor{tablehead}
\textcolor{accent}{Protocol} & \textcolor{accent}{Purpose} & \textcolor{accent}{Standard} \\
MCP  & Agent-to-Tools & Anthropic \\
A2A  & Agent-to-Agent & Google / Linux Foundation \\
A2UI & Agent-to-User  & Google \\
\end{tabular}
\end{center}
\vspace{2pt}
""" + img_block("slide4_diagram0", "0.85\\linewidth") + r"""
\end{frame}

% ============================================================================
% Slide 5: Agent Pipeline
% ============================================================================
\begin{frame}{Six agents, self-coordinating via A2A}
""" + img_block("slide5_diagram0") + r"""
\begin{itemize}
  \item Own Kubernetes pod
  \item Own Claude API session
  \item Own MCP tools (filesystem, Docker, Helm)
  \item A2A endpoint for peer communication
\end{itemize}
\end{frame}

% ============================================================================
% Slide 6: Kubernetes-Native
% ============================================================================
\begin{frame}[fragile]{Two Custom Resource Definitions}
\begin{columns}[T]
\begin{column}{0.48\textwidth}
\begin{lstlisting}[style=yaml]
apiVersion: forgemaster.io/v1alpha1
kind: AgentTask
spec:
  description: "Build a REST API..."
status:
  phase: Running
  error: 0.35
\end{lstlisting}
\end{column}
\begin{column}{0.48\textwidth}
\begin{lstlisting}[style=yaml]
apiVersion: forgemaster.io/v1alpha1
kind: Agent
spec:
  type: code-generator
  model:
    name: claude-sonnet-4-20250514
    maxTokens: 16384
  mcpServers:
    - name: fm-mcp-filesystem
    - name: fm-mcp-devtools
\end{lstlisting}
\end{column}
\end{columns}
\vspace{4pt}
""" + img_block("slide6_diagram0", "0.75\\linewidth") + r"""
\end{frame}

% ============================================================================
% Slide 7: DEMO
% ============================================================================
{
\begin{frame}[plain]
\vfill
\begin{center}
{\Huge\bfseries\color{accent}LIVE DEMO}\\[20pt]
{\color{muted}\texttt{kubectl apply -f agenttask.yaml}}\\[12pt]
{\color{muted}Watch agents spin up, code, deploy, and test autonomously}
\end{center}
\vfill
\end{frame}
}

% ============================================================================
% Slide 8: Engineering Quality
% ============================================================================
\begin{frame}{Production-grade, not a prototype}
\vspace{6pt}
\begin{center}
\small
\rowcolors{2}{tablerow}{tablealt}
\begin{tabular}{>{\color{accent}\bfseries}l l}
\rowcolor{tablehead}
& \\
Language       & Rust (5 crates, single responsibility each) \\
Orchestration  & Kubernetes CRDs + Operators \\
Deployment     & Helm charts + Ansible \\
Environments   & Local (OrbStack) / Remote (ghcr.io) \\
CI/CD          & GitHub Actions to ghcr.io \\
Documentation  & 10 ADRs, architecture docs \\
Observability  & Full audit trail per agent conversation \\
\end{tabular}
\end{center}
\end{frame}

% ============================================================================
% Slide 9: Why ForgeMaster Should Win
% ============================================================================
\begin{frame}{Why ForgeMaster Should Win}
\textcolor{accent}{\textbf{1. Standards-based}}\\[2pt]
A2A + MCP + A2UI -- composing open protocols, not reinventing them.\\
Any MCP server, any A2A agent can plug in.\\[14pt]

\textcolor{accent}{\textbf{2. Kubernetes-native}}\\[2pt]
CRDs, operators, namespace isolation, Helm, RBAC.\\
Scales. Observable. Production-ready.\\[14pt]

\textcolor{accent}{\textbf{3. Control theory meets AI}}\\[2pt]
TCP controller: microseconds, zero cost, deterministic.\\
LLM budget reserved for creative work only.
\end{frame}

% ============================================================================
% Slide 10: Close
% ============================================================================
{
\begin{frame}[plain]
\vfill
\begin{center}
{\Huge\bfseries\color{white}ForgeMaster}\\[16pt]
{\Large\color{accent}Autonomous agent orchestration,}\\[4pt]
{\Large\color{accent}powered by control theory,}\\[4pt]
{\Large\color{accent}built for Kubernetes.}\\[50pt]
{\small\color{muted}github.com/dshcherbatiuk/forgemaster}\\[4pt]
{\small\color{muted}Dmytro Shcherbatiuk\enspace|\enspace Team CSM-101}
\end{center}
\vfill
\end{frame}
}

\end{document}
"""
    return tex


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

    # Step 3: Generate .tex
    print("[3/4] Generating Beamer .tex...")
    TARGET_DIR.mkdir(parents=True, exist_ok=True)
    tex_content = generate_tex(diagrams)
    OUTPUT_TEX.write_text(tex_content)
    print(f"  Written: {OUTPUT_TEX}\n")

    # Step 4: Compile to PDF
    print("[4/4] Compiling with tectonic...")
    compile_tex(OUTPUT_TEX)
    print(f"\n  Output: {OUTPUT_PDF}")
    print("\nDone!")


if __name__ == "__main__":
    main()
