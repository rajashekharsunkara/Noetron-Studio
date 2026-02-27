"""
Versioner — content-addressed dataset versioning for the .aiproj store.

Datasets are stored in `.aiproj/data/<sha256_hex>` so identical files are
never stored twice. A version record maps a human name + timestamp to the hash.
"""

from __future__ import annotations

import hashlib
import json
import shutil
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


_INDEX_FILE = "versions.json"


class Versioner:
    """Store and retrieve dataset versions inside an `.aiproj/data/` folder.

    Usage::

        v = Versioner(".aiproj/data")
        version_id = v.save("train-v1", "data/train.csv")
        path = v.load(version_id)
    """

    def __init__(self, store_dir: str | Path) -> None:
        self.store = Path(store_dir)
        self.store.mkdir(parents=True, exist_ok=True)
        self._index_path = self.store / _INDEX_FILE

    # ── public ──────────────────────────────────────────────────────────────

    def save(self, name: str, source_path: str | Path) -> str:
        """Copy *source_path* into the content-addressed store.

        Returns the version ID (a hex digest of the file contents).
        """
        src = Path(source_path)
        digest = _sha256(src)
        dest = self.store / digest
        if not dest.exists():
            shutil.copy2(src, dest)

        index = self._load_index()
        record = {
            "name": name,
            "hash": digest,
            "source": str(src),
            "size_bytes": src.stat().st_size,
            "created_at": _now_iso(),
        }
        index[digest] = record
        self._save_index(index)
        return digest

    def load(self, version_id: str) -> Path:
        """Return the path to the stored file for *version_id*.

        Raises ``KeyError`` if the version does not exist.
        """
        index = self._load_index()
        if version_id not in index:
            raise KeyError(f"Version '{version_id}' not found in store {self.store}")
        return self.store / version_id

    def list_versions(self) -> list[dict]:
        """Return all stored version records (sorted newest first)."""
        return sorted(
            self._load_index().values(),
            key=lambda r: r.get("created_at", ""),
            reverse=True,
        )

    def delete(self, version_id: str) -> None:
        """Remove a version from the index (does NOT delete the blob — blobs are shared)."""
        index = self._load_index()
        index.pop(version_id, None)
        self._save_index(index)

    # ── private ─────────────────────────────────────────────────────────────

    def _load_index(self) -> dict:
        if not self._index_path.exists():
            return {}
        with self._index_path.open() as f:
            return json.load(f)

    def _save_index(self, index: dict) -> None:
        with self._index_path.open("w") as f:
            json.dump(index, f, indent=2)


def _sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()


def _now_iso() -> str:
    return datetime.now(timezone.utc).isoformat()
