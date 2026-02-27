"""
DVC bridge — wraps the `dvc` CLI so Noetron can use DVC for data and model
versioning when DVC is installed, falling back gracefully to the built-in
Versioner when it isn't.

Usage::

    from noetron_runtime.data.dvc_bridge import DvcBridge

    bridge = DvcBridge(project_root=".aiproj")
    bridge.init()                              # dvc init (if not already)
    bridge.add("data/train.csv")               # dvc add
    bridge.push()                              # dvc push (if remote configured)
    bridge.pull("data/train.csv")              # dvc pull
    bridge.status()                            # dict of changed files
"""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path
from typing import Any


class DvcBridge:
    """Thin wrapper around the DVC CLI.

    If DVC is not installed, all methods either no-op or raise a clear error
    depending on whether *require_dvc* is True (default: False).
    """

    def __init__(self, project_root: str | Path = ".", require_dvc: bool = False) -> None:
        self.root = Path(project_root).resolve()
        self.require_dvc = require_dvc
        self._available: bool | None = None

    # ── availability ─────────────────────────────────────────────────────────

    def is_available(self) -> bool:
        if self._available is None:
            try:
                result = subprocess.run(
                    ["dvc", "version"],
                    capture_output=True,
                    text=True,
                    check=False,
                )
                self._available = result.returncode == 0
            except FileNotFoundError:
                self._available = False
        return self._available  # type: ignore[return-value]

    def require(self) -> None:
        if not self.is_available():
            raise RuntimeError(
                "DVC is not installed. Install it with: pip install dvc\n"
                "Or use noetron_runtime.data.Versioner for built-in versioning."
            )

    # ── DVC commands ─────────────────────────────────────────────────────────

    def init(self, no_scm: bool = False) -> bool:
        """Run `dvc init` in the project root if not already initialised."""
        if not self.is_available():
            return False
        if (self.root / ".dvc").exists():
            return True  # already initialised
        cmd = ["dvc", "init"]
        if no_scm:
            cmd.append("--no-scm")
        return self._run(cmd)

    def add(self, path: str | Path) -> bool:
        """Track a file/directory with DVC (`dvc add <path>`)."""
        if not self._check():
            return False
        return self._run(["dvc", "add", str(path)])

    def push(self) -> bool:
        """Push tracked files to the configured remote (`dvc push`)."""
        if not self._check():
            return False
        return self._run(["dvc", "push"])

    def pull(self, path: str | Path | None = None) -> bool:
        """Pull tracked files from remote (`dvc pull [path]`)."""
        if not self._check():
            return False
        cmd = ["dvc", "pull"]
        if path:
            cmd.append(str(path))
        return self._run(cmd)

    def repro(self, target: str | None = None) -> bool:
        """Reproduce a DVC pipeline stage (`dvc repro [target]`)."""
        if not self._check():
            return False
        cmd = ["dvc", "repro"]
        if target:
            cmd.append(target)
        return self._run(cmd)

    def status(self) -> dict:
        """Return `dvc status --json` as a Python dict."""
        if not self._check():
            return {}
        result = subprocess.run(
            ["dvc", "status", "--json"],
            capture_output=True,
            text=True,
            cwd=self.root,
        )
        if result.returncode != 0:
            return {"error": result.stderr.strip()}
        try:
            return json.loads(result.stdout)
        except json.JSONDecodeError:
            return {"raw": result.stdout}

    def list_tracked(self) -> list[str]:
        """Return a list of files currently tracked by DVC."""
        if not self._check():
            return []
        result = subprocess.run(
            ["dvc", "ls", "--dvc-only", "--recursive"],
            capture_output=True,
            text=True,
            cwd=self.root,
        )
        if result.returncode != 0:
            return []
        return [line.strip() for line in result.stdout.splitlines() if line.strip()]

    # ── helpers ───────────────────────────────────────────────────────────────

    def _check(self) -> bool:
        if self.require_dvc:
            self.require()
            return True
        return self.is_available()

    def _run(self, cmd: list[str]) -> bool:
        result = subprocess.run(
            cmd,
            cwd=self.root,
            capture_output=False,
        )
        return result.returncode == 0
