"""
AutoLogger — automatically records params, metrics, and artifacts for every run.

The run data is written to `.aiproj/experiments/<run_id>/`:
    params.json       — hyperparameters captured from the model
    metrics.json      — evaluation metrics
    artifacts/        — model file, charts, etc.

The executor reads these files after training completes to persist them in the
SQLite database via noetron_db.
"""

from __future__ import annotations

import json
import os
import shutil
import uuid
from contextlib import contextmanager
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Generator


_ACTIVE: "AutoLogger | None" = None


class AutoLogger:
    """Context-manager that captures run parameters and metrics.

    Usage::

        with AutoLogger(".aiproj/experiments") as log:
            log.log_param("n_estimators", 100)
            model.fit(X_train, y_train)
            log.log_metric("accuracy", acc)
            log.log_artifact("model.pkl")
    """

    def __init__(self, experiments_dir: str | Path, run_name: str | None = None) -> None:
        self.experiments_dir = Path(experiments_dir)
        self.run_id = str(uuid.uuid4())
        self.run_name = run_name or f"run-{self.run_id[:8]}"
        self.run_dir = self.experiments_dir / self.run_id
        self._params: dict[str, Any] = {}
        self._metrics: dict[str, Any] = {}
        self._start_time: str | None = None

    # ── context manager ──────────────────────────────────────────────────────

    def __enter__(self) -> "AutoLogger":
        global _ACTIVE
        self.run_dir.mkdir(parents=True, exist_ok=True)
        (self.run_dir / "artifacts").mkdir(exist_ok=True)
        self._start_time = _now_iso()
        _ACTIVE = self
        return self

    def __exit__(self, *_: Any) -> None:
        global _ACTIVE
        self._flush()
        _ACTIVE = None

    # ── logging api ──────────────────────────────────────────────────────────

    def log_param(self, key: str, value: Any) -> None:
        self._params[key] = value

    def log_params(self, params: dict[str, Any]) -> None:
        self._params.update(params)

    def log_metric(self, key: str, value: float) -> None:
        self._metrics[key] = value

    def log_metrics(self, metrics: dict[str, float]) -> None:
        self._metrics.update(metrics)

    def log_artifact(self, path: str | Path) -> None:
        """Copy *path* into the run's artifacts directory."""
        src = Path(path)
        if src.exists():
            shutil.copy2(src, self.run_dir / "artifacts" / src.name)

    def auto_log_sklearn(self, model: Any) -> None:
        """Extract params from any sklearn estimator and log them automatically."""
        try:
            params = model.get_params()
            self.log_params({str(k): v for k, v in params.items()})
        except AttributeError:
            pass

    # ── private ──────────────────────────────────────────────────────────────

    def _flush(self) -> None:
        meta = {
            "run_id": self.run_id,
            "run_name": self.run_name,
            "start_time": self._start_time,
            "end_time": _now_iso(),
        }
        _write_json(self.run_dir / "meta.json", meta)
        _write_json(self.run_dir / "params.json", self._params)
        _write_json(self.run_dir / "metrics.json", self._metrics)


# ── Module-level helpers (used by generated pipeline code) ───────────────────

def get_active() -> "AutoLogger | None":
    """Return the currently active AutoLogger, if any."""
    return _ACTIVE


def log_param(key: str, value: Any) -> None:
    if _ACTIVE:
        _ACTIVE.log_param(key, value)


def log_metric(key: str, value: float) -> None:
    if _ACTIVE:
        _ACTIVE.log_metric(key, value)


def log_artifact(path: str | Path) -> None:
    if _ACTIVE:
        _ACTIVE.log_artifact(path)


# ── Utilities ─────────────────────────────────────────────────────────────────

def _write_json(path: Path, data: Any) -> None:
    with path.open("w") as f:
        json.dump(data, f, indent=2, default=str)


def _now_iso() -> str:
    return datetime.now(timezone.utc).isoformat()
