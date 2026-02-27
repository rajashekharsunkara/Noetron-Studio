"""
PipelineRunner — load and execute a Noetron pipeline script.

The runner is invoked by the Rust executor as a subprocess::

    python -m noetron_runtime.pipeline.runner \\
        --script .aiproj/pipelines/train.py \\
        --experiments-dir .aiproj/experiments \\
        --run-name run-001
"""

from __future__ import annotations

import argparse
import importlib.util
import sys
from pathlib import Path


class PipelineRunner:
    """Execute a pipeline script file inside an AutoLogger context."""

    def __init__(
        self,
        script_path: str | Path,
        experiments_dir: str | Path,
        run_name: str | None = None,
    ) -> None:
        self.script_path = Path(script_path)
        self.experiments_dir = Path(experiments_dir)
        self.run_name = run_name

    def run(self) -> str:
        """Execute the script and return the run_id."""
        from ..experiment.auto_logger import AutoLogger

        with AutoLogger(self.experiments_dir, self.run_name) as log:
            _exec_script(self.script_path)
            return log.run_id


def _exec_script(path: Path) -> None:
    """Execute a Python file in the caller's namespace."""
    spec = importlib.util.spec_from_file_location("__pipeline__", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"Cannot load pipeline script: {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)  # type: ignore[attr-defined]


# ── CLI entry point ───────────────────────────────────────────────────────────

def _cli() -> None:
    parser = argparse.ArgumentParser(description="Noetron pipeline runner")
    parser.add_argument("--script", required=True, help="Path to pipeline .py")
    parser.add_argument("--experiments-dir", required=True, dest="experiments_dir")
    parser.add_argument("--run-name", default=None, dest="run_name")
    args = parser.parse_args()

    runner = PipelineRunner(args.script, args.experiments_dir, args.run_name)
    run_id = runner.run()
    print(f"NOETRON_RUN_ID={run_id}", flush=True)
    sys.exit(0)


if __name__ == "__main__":
    _cli()
