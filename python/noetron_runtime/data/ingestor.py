"""
Ingestor — load tabular, text, image, and tensor datasets into a DataFrame / numpy array.
Supports CSV, JSON, Parquet, and image folders.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any


class Ingestor:
    """Load a dataset from disk into memory.

    Usage::

        df = Ingestor("data/train.csv").load()
        df = Ingestor("data/images/", source_type="image_folder").load()
    """

    SUPPORTED_TYPES = ("csv", "json", "parquet", "image_folder", "text")

    def __init__(self, source: str, source_type: str | None = None) -> None:
        self.source = Path(source)
        self.source_type = source_type or self._detect_type()

    # ── public ──────────────────────────────────────────────────────────────

    def load(self) -> Any:
        """Return a pandas DataFrame or numpy ndarray depending on source type."""
        loaders = {
            "csv": self._load_csv,
            "json": self._load_json,
            "parquet": self._load_parquet,
            "image_folder": self._load_image_folder,
            "text": self._load_text,
        }
        loader = loaders.get(self.source_type)
        if loader is None:
            raise ValueError(
                f"Unknown source type '{self.source_type}'. "
                f"Supported: {self.SUPPORTED_TYPES}"
            )
        return loader()

    # ── private ─────────────────────────────────────────────────────────────

    def _detect_type(self) -> str:
        suffix = self.source.suffix.lower()
        if suffix == ".csv":
            return "csv"
        if suffix == ".json":
            return "json"
        if suffix in (".parquet", ".pq"):
            return "parquet"
        if self.source.is_dir():
            return "image_folder"
        if suffix in (".txt", ".text"):
            return "text"
        return "csv"  # safe fallback

    def _load_csv(self) -> Any:
        import pandas as pd  # type: ignore
        return pd.read_csv(self.source)

    def _load_json(self) -> Any:
        import pandas as pd  # type: ignore
        return pd.read_json(self.source)

    def _load_parquet(self) -> Any:
        import pandas as pd  # type: ignore
        return pd.read_parquet(self.source)

    def _load_image_folder(self) -> Any:
        """Return a dict {label -> [PIL.Image, ...]} from a labelled folder structure.

        Expected layout::

            images/
                cat/  img1.jpg  img2.jpg
                dog/  img3.jpg
        """
        try:
            from PIL import Image  # type: ignore
        except ImportError:
            raise ImportError("Pillow is required for image loading: pip install Pillow")

        folder = self.source
        result: dict[str, list] = {}
        for label_dir in sorted(folder.iterdir()):
            if not label_dir.is_dir():
                continue
            images = []
            for img_path in sorted(label_dir.iterdir()):
                if img_path.suffix.lower() in (".jpg", ".jpeg", ".png", ".bmp", ".webp"):
                    images.append(Image.open(img_path))
            result[label_dir.name] = images
        return result

    def _load_text(self) -> list[str]:
        return self.source.read_text(encoding="utf-8").splitlines()
