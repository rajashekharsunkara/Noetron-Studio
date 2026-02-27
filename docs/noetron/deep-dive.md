# Noetron Studio — Full Deep-Dive Architecture

> This document is the authoritative technical reference for all five
> core systems: the Zed fork, `.aiproj` format, the toggle bar,
> the no-code panels, and the IR sync engine.

---

## 1. The Zed Fork

### Why Fork Instead of Integrate?

Zed is GPL v3. Since we are open-source and don't care about licensing
restrictions, forking is the right move. It gives us:

- Full control over the UI shell, layout, and panels
- Ability to strip everything that AI engineers don't need
- A production-quality GPU-accelerated editor (Rust + GPUI) as our base
- Real-time collaboration built-in from day one
- Tree-sitter parsing, Python LSP, terminal — all already working

### What Gets Stripped

| Feature | Reason for Removal |
|---------|-------------------|
| JavaScript / TypeScript LSP defaults | AI engineers write Python, not JS |
| Ruby, Go, C++ language server configs | Not needed in AI workflows |
| Web preview panel | Not relevant |
| Extension marketplace UI | Replaced by `.aiproj` plugin system |
| Remote SSH editing UI | Out of scope for v1 |
| General project templates (JS, TS, etc.) | Replaced by `.aiproj` domain templates |
| Zed AI (built-in Copilot/Claude) | Replaced by Noetron's own AI features later |
| Channel chat (Zed's Slack-like feature) | Unnecessary complexity |

### What Stays

| Feature | Why We Keep It |
|---------|---------------|
| Core editor (syntax highlight, Tree-sitter) | Foundation |
| Python LSP (Pyright + Ruff) | Primary language for AI engineering |
| Rust LSP (rust-analyzer) | Our own codebase is Rust |
| **Real-time collaboration** | AI teams work together on experiments, pipelines |
| Git integration (diff, blame, commit) | Every AI project needs version control |
| Terminal panel | Running training scripts, DVC commands, etc. |
| Multi-pane layout system | Side-by-side code + panels |
| GPUI rendering engine | The entire no-code layer is built on this |
| Search (file + in-project) | Essential |
| Settings / keybindings system | Developers expect this |
| Task runner (build/run tasks) | Trigger training runs from keybinds |

### Fork Structure

```
noetron-studio/  (forked from zed-industries/zed)
├── crates/
│   ├── zed/                    # main binary — heavily modified entry point
│   ├── editor/                 # kept mostly intact
│   ├── gpui/                   # kept entirely — our rendering foundation
│   ├── language/               # kept, Python-focused
│   ├── collab/                 # kept (real-time collaboration)
│   ├── terminal/               # kept
│   ├── git/                    # kept
│   │
│   ├── noetron_aiproj/         # NEW — .aiproj format + domain detection
│   ├── noetron_ui/             # NEW — all no-code GPUI panels
│   ├── noetron_ir/             # NEW — IR engine + form↔code sync
│   ├── noetron_db/             # NEW — SQLite project database
│   ├── noetron_executor/       # NEW — pipeline execution + auto-logging
│   └── noetron_toggle/         # NEW — the top toggle bar
│
├── python/
│   └── noetron_runtime/        # Python ML runtime (embedded via PyO3)
│
├── docs/
│   └── noetron/                # All Noetron-specific documentation
│
└── .aiproj_templates/          # Built-in domain templates
    ├── ml/
    ├── dl/
    ├── nlp/
    ├── cv/
    └── ...
```

---

## 2. The `.aiproj` Format

### Concept

`.aiproj/` is to an AI project what `.git/` is to a git repo. It is a hidden
directory at the project root that defines everything Noetron Studio needs to
know about the project: its domain, active features, data, models, experiments,
and pipelines.

It is **not** a replacement for git. Both coexist. `.aiproj/` is committed to
git (except large data files handled by DVC).

### Directory Structure

```
my-project/
├── .aiproj/
│   ├── project.toml            # project manifest (domain, name, features)
│   ├── db/
│   │   └── project.db          # SQLite — all metadata (runs, models, etc.)
│   ├── data/
│   │   └── <sha256>.<ext>      # content-addressed dataset snapshots
│   ├── experiments/
│   │   └── <run-id>/
│   │       ├── metrics.json
│   │       ├── params.json
│   │       └── artifacts/
│   ├── models/
│   │   └── <model-id>/
│   │       └── v<n>.<format>   # versioned model files
│   └── pipelines/
│       └── <pipeline-id>/
│           ├── pipeline.json   # IR definition
│           └── pipeline.py     # generated Python (always in sync with IR)
├── src/                        # user's code
└── data -> .aiproj/data/       # optional symlink for convenience
```

### `project.toml` Specification

```toml
[project]
id = "550e8400-e29b-41d4-a716-446655440000"
name = "Customer Churn Prediction"
domain = "machine_learning"
created_at = "2026-02-27T08:00:00Z"

[features]
data_management    = "full"
pipeline_management = "full"
experiment_tracking = "full"
model_management   = "full"
versioning         = "full"
logic              = "full"
labeling           = "disabled"

[domain.machine_learning]
task_type = "classification"
target_column = "churn"
framework = "sklearn"

[versioning]
backend = "dvc"
remote = ""

[python]
interpreter = ".venv/bin/python"
requirements = "requirements.txt"
```

### Domain Detection

```
1. Does .aiproj/project.toml exist?
   YES → read domain + features → activate Noetron UI mode
   NO  → plain Zed editor (no Noetron panels shown)

2. Is this a new folder?
   → Show "Initialize AI Project" prompt
   → User picks domain → .aiproj/ is scaffolded from template
```

---

## 3. The Toggle Bar

A persistent bar pinned to the **top of every editor tab** for `.aiproj` files.

```
┌──────────────────────────────────────────────────────────────────────┐
│  📊 Pipeline: train_pipeline   [◉ No-Code]  [ Full Code ]  ▼ ML     │
└──────────────────────────────────────────────────────────────────────┘
```

### How The Switch Works

The toggle replaces the content area of the current pane with a different
renderer — both backed by the same IR and the same file on disk.

```
No-Code view:
┌───────────────────────────────────────────────────────────┐
│  ┌──Stage 1──┐  →  ┌──Stage 2──┐  →  ┌──Stage 3──┐      │
│  │  Ingest   │     │ Preprocess│     │   Train   │      │
│  │  [form]   │     │  [form]   │     │  [form]   │      │
│  └───────────┘     └───────────┘     └───────────┘      │
└───────────────────────────────────────────────────────────┘

Toggle ↕ (instant — GPUI pane content swap)

Full Code view:
┌───────────────────────────────────────────────────────────┐
│  1  def ingest(source: str) -> pd.DataFrame:              │
│  2      return pd.read_csv(source)                        │
│  3                                                        │
│  4  def preprocess(df):                                   │
│  5      ...                                               │
└───────────────────────────────────────────────────────────┘
```

### Sync Guarantee

- **No-Code → Full Code:** IR runs codegen, Python file is updated, editor shows it
- **Full Code → No-Code:** Tree-sitter parses file, known patterns → form fields,
  unknown patterns → `⚠ custom — edit in code` (nothing is ever lost)

---

## 4. The No-Code Panels

### 4.1 Data Management Panel

```
┌─ Data Management ───────────────────────────────────────┐
│ Source  [ CSV ▼ ]  [Browse…]  /path/to/data.csv         │
│                                                         │
│ ┌─ Profile ─────────────────────────────────────────┐   │
│ │  5,000 rows  |  12 cols  |  3 nulls  |  0 dupes   │   │
│ └───────────────────────────────────────────────────┘   │
│                                                         │
│ ┌─ Preprocessing Steps ─────────────────────────────┐   │
│ │  1. Drop nulls       [columns: all ▼]    [✕]      │   │
│ │  2. Encode labels    [columns: churn ▼]  [✕]      │   │
│ │  3. Normalize        [method: standard ▼][✕]      │   │
│ │  [+ Add Step]                                     │   │
│ └───────────────────────────────────────────────────┘   │
│                                                         │
│ Versions: v3 (current) ← v2 ← v1    [View diff]        │
└─────────────────────────────────────────────────────────┘
```

Domain adaptation: ML→tabular, DL→tensors, NLP→text corpora, CV→image folders.

### 4.2 Pipeline Stage-Lane Editor

```
┌─ Pipeline: train_pipeline ──────────────────────────────┐
│                                                         │
│  ┌─────────┐     ┌─────────────┐     ┌──────────────┐  │
│  │ Ingest  │────▶│ Preprocess  │────▶│    Train     │  │
│  │ CSV     │     │ drop_nulls  │     │ RandomForest │  │
│  └─────────┘     └─────────────┘     └──────────────┘  │
│       [+]              [+]                  [+]         │
│                                                         │
│  [▶ Run Pipeline]   [Save]   [Export as script]         │
└─────────────────────────────────────────────────────────┘
```

### 4.3 Experiment Tracking Dashboard

```
┌─ Experiments ───────────────────────────────────────────┐
│  Run ID   │ Dataset │ accuracy │  F1   │ Duration       │
│  run_042  │ v3      │  0.923   │ 0.911 │   12s    ★    │
│  run_041  │ v3      │  0.901   │ 0.888 │   11s         │
│                                                         │
│  [Compare selected]  [Reproduce run_042]                │
└─────────────────────────────────────────────────────────┘
```

Auto-logging: every run captured automatically — zero config.

### 4.4 Model Registry

```
┌─ Model Registry ────────────────────────────────────────┐
│  ┌─ churn_classifier ───────────────────────────────┐   │
│  │  v3 ← v2 ← v1                                   │   │
│  │  acc=0.923  |  Trained on: dataset v3, run_042   │   │
│  │  [Test inference]  [Export ▼]  [View card]       │   │
│  └──────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

---

## 5. The IR Engine

### Codegen: IR → Python (deterministic)

Same IR → same Python file, every time.

### Parser: Python → IR (best-effort via Tree-sitter)

Known patterns → IR fields. Unknown patterns → `FieldValue::Custom(String)`.

```python
# Known → maps to IR
Step("normalize", method="standard")
RandomForestClassifier(n_estimators=100)

# Unknown → Custom(String), preserved verbatim
model = get_model_from_registry("custom_model")
```

### Sync Loop

```
File saved in Full Code view
  → Tree-sitter parse → new IR
  → diff against current IR
  → emit change events
  → No-Code panel re-renders changed fields only
  → toggle is instant (both views always mounted in GPUI)
```

---

## System Overview

```
┌──────────────────── Noetron Studio (forked Zed) ─────────────────────┐
│                                                                       │
│  ┌─ Zed core ──────────────────────────────────────────────────────┐ │
│  │  GPUI · Tree-sitter · Python LSP · Collaboration · Git · Term   │ │
│  └─────────────────────────────────────────────────────────────────┘ │
│            │                                                          │
│  ┌─ .aiproj detector ──┐        ┌─ Toggle Bar ──────────────────┐   │
│  │  domain detection   │───────▶│  [◉ No-Code] [ Full Code ]    │   │
│  │  template scaffold  │        ├───────────────────────────────┤   │
│  └─────────────────────┘        │  No-Code GPUI panels          │   │
│                                 │    OR                         │   │
│  ┌─ SQLite (.aiproj/db) ──┐     │  Zed editor (full code)       │   │
│  │  all project metadata  │◀───▶│                               │   │
│  └────────────────────────┘     │  Same IR. Same file. In sync. │   │
│                                 └───────────────────────────────┘   │
│  ┌─ IR Engine ─────────────┐                                         │
│  │  codegen ◄──► parser    │  (Tree-sitter, always in sync)          │
│  └─────────────────────────┘                                         │
│                                                                       │
│  ┌─ Python runtime (PyO3) ───────────────────────────────────────┐   │
│  │  Ingestor · Profiler · Trainer · AutoLogger · Exporter        │   │
│  └───────────────────────────────────────────────────────────────┘   │
└───────────────────────────────────────────────────────────────────────┘
```
