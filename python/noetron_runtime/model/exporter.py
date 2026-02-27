"""
Exporter — save trained models to various formats.
Supports: pickle (.pkl), joblib (.joblib), ONNX (.onnx).
"""

from __future__ import annotations

import pickle
from pathlib import Path
from typing import Any


class Exporter:
    """Export a trained sklearn/compatible model to disk.

    Usage::

        exp = Exporter(trainer.estimator)
        exp.to_pickle("models/rf_v1.pkl")
        exp.to_onnx("models/rf_v1.onnx", input_shape=(None, 12))
    """

    def __init__(self, model: Any) -> None:
        self.model = model

    def to_pickle(self, path: str | Path) -> Path:
        dest = Path(path)
        dest.parent.mkdir(parents=True, exist_ok=True)
        with dest.open("wb") as f:
            pickle.dump(self.model, f, protocol=5)
        return dest

    def to_joblib(self, path: str | Path) -> Path:
        try:
            import joblib  # type: ignore
        except ImportError:
            raise ImportError("joblib is required: pip install joblib")
        dest = Path(path)
        dest.parent.mkdir(parents=True, exist_ok=True)
        joblib.dump(self.model, dest)
        return dest

    def to_onnx(self, path: str | Path, input_shape: tuple, feature_names: list[str] | None = None) -> Path:
        """Convert to ONNX via skl2onnx.

        Args:
            path: output file path
            input_shape: shape tuple, e.g. (None, 12)
            feature_names: list of feature names; defaults to numbered columns
        """
        try:
            from skl2onnx import convert_sklearn  # type: ignore
            from skl2onnx.common.data_types import FloatTensorType  # type: ignore
        except ImportError:
            raise ImportError("skl2onnx is required for ONNX export: pip install skl2onnx")

        n_features = input_shape[-1] if len(input_shape) > 1 else None
        initial_type = [("float_input", FloatTensorType([None, n_features]))]
        onnx_model = convert_sklearn(self.model, initial_types=initial_type)

        dest = Path(path)
        dest.parent.mkdir(parents=True, exist_ok=True)
        with dest.open("wb") as f:
            f.write(onnx_model.SerializeToString())
        return dest
