"""
Profiler — automatic dataset profiling for tabular data.
Returns a lightweight profile dict that the UI renders as cards.
"""

from __future__ import annotations

from typing import Any


class Profiler:
    """Compute basic statistics for a pandas DataFrame.

    Usage::

        profile = Profiler(df).run()
        print(profile["shape"])              # (1000, 12)
        print(profile["columns"]["age"])     # {...stats...}
    """

    def __init__(self, df: Any) -> None:
        self.df = df

    def run(self) -> dict:
        """Return the full profile as a plain dict (JSON-serialisable)."""
        try:
            import pandas as pd  # type: ignore
            import numpy as np  # type: ignore
        except ImportError:
            raise ImportError("pandas and numpy are required: pip install pandas numpy")

        df = self.df
        profile: dict = {
            "shape": list(df.shape),
            "memory_mb": round(df.memory_usage(deep=True).sum() / 1e6, 3),
            "columns": {},
        }

        for col in df.columns:
            series = df[col]
            dtype = str(series.dtype)
            null_count = int(series.isna().sum())
            unique_count = int(series.nunique())
            info: dict = {
                "dtype": dtype,
                "null_count": null_count,
                "null_pct": round(null_count / len(df) * 100, 2) if len(df) else 0.0,
                "unique_count": unique_count,
            }

            if pd.api.types.is_numeric_dtype(series):
                desc = series.describe()
                info.update({
                    "mean": _safe_float(desc.get("mean")),
                    "std": _safe_float(desc.get("std")),
                    "min": _safe_float(desc.get("min")),
                    "p25": _safe_float(desc.get("25%")),
                    "median": _safe_float(desc.get("50%")),
                    "p75": _safe_float(desc.get("75%")),
                    "max": _safe_float(desc.get("max")),
                })
            elif pd.api.types.is_string_dtype(series) or pd.api.types.is_categorical_dtype(series):
                top = series.value_counts().head(5).to_dict()
                info["top_values"] = {str(k): int(v) for k, v in top.items()}

            profile["columns"][col] = info

        return profile


def _safe_float(val: Any) -> float | None:
    try:
        import math
        f = float(val)
        return None if math.isnan(f) else round(f, 6)
    except (TypeError, ValueError):
        return None
