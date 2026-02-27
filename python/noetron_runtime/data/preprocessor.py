"""
Preprocessor — a composable, declarative pipeline of data transformation steps.

Steps are described by name so they serialise cleanly into IR / TOML.
Each step is a callable (func or class with __call__) applied to a DataFrame.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, Callable


# ── Step descriptor ───────────────────────────────────────────────────────────

@dataclass
class Step:
    """A single preprocessing step.

    The name must be one of the registered built-in names, or a callable
    can be supplied directly for custom steps (stored as Custom in the IR).

    Usage::

        Step("drop_nulls")
        Step("scale_standard", columns=["age", "income"])
        Step("encode_label", target="species")
    """
    name: str
    kwargs: dict = field(default_factory=dict)

    def __init__(self, name: str, **kwargs: Any) -> None:
        self.name = name
        self.kwargs = kwargs


# ── Built-in step implementations ────────────────────────────────────────────

def _drop_nulls(df: Any, **_kw: Any) -> Any:
    return df.dropna()


def _fill_nulls(df: Any, strategy: str = "mean", **_kw: Any) -> Any:
    import pandas as pd  # type: ignore
    num_cols = df.select_dtypes(include="number").columns
    if strategy == "mean":
        return df.fillna(df[num_cols].mean())
    if strategy == "median":
        return df.fillna(df[num_cols].median())
    if strategy == "zero":
        return df.fillna(0)
    raise ValueError(f"Unknown fill strategy '{strategy}'")


def _scale_standard(df: Any, columns: list[str] | None = None, **_kw: Any) -> Any:
    from sklearn.preprocessing import StandardScaler  # type: ignore
    cols = columns or list(df.select_dtypes(include="number").columns)
    scaler = StandardScaler()
    df = df.copy()
    df[cols] = scaler.fit_transform(df[cols])
    return df


def _scale_minmax(df: Any, columns: list[str] | None = None, **_kw: Any) -> Any:
    from sklearn.preprocessing import MinMaxScaler  # type: ignore
    cols = columns or list(df.select_dtypes(include="number").columns)
    scaler = MinMaxScaler()
    df = df.copy()
    df[cols] = scaler.fit_transform(df[cols])
    return df


def _encode_label(df: Any, target: str, **_kw: Any) -> Any:
    from sklearn.preprocessing import LabelEncoder  # type: ignore
    df = df.copy()
    df[target] = LabelEncoder().fit_transform(df[target])
    return df


def _encode_onehot(df: Any, columns: list[str] | None = None, **_kw: Any) -> Any:
    import pandas as pd  # type: ignore
    cols = columns or list(df.select_dtypes(include=["object", "category"]).columns)
    return pd.get_dummies(df, columns=cols)


def _drop_columns(df: Any, columns: list[str], **_kw: Any) -> Any:
    return df.drop(columns=columns, errors="ignore")


def _rename_columns(df: Any, mapping: dict[str, str], **_kw: Any) -> Any:
    return df.rename(columns=mapping)


_REGISTRY: dict[str, Callable] = {
    "drop_nulls": _drop_nulls,
    "fill_nulls": _fill_nulls,
    "scale_standard": _scale_standard,
    "scale_minmax": _scale_minmax,
    "encode_label": _encode_label,
    "encode_onehot": _encode_onehot,
    "drop_columns": _drop_columns,
    "rename_columns": _rename_columns,
}


# ── Preprocessor ─────────────────────────────────────────────────────────────

class Preprocessor:
    """Apply a sequence of Steps to a DataFrame.

    Usage::

        pre = Preprocessor([
            Step("drop_nulls"),
            Step("scale_standard"),
            Step("encode_label", target="species"),
        ])
        df_clean = pre.fit_transform(df)
    """

    def __init__(self, steps: list[Step]) -> None:
        self.steps = steps

    def fit_transform(self, df: Any) -> Any:
        for step in self.steps:
            fn = _REGISTRY.get(step.name)
            if fn is None:
                raise ValueError(
                    f"Unknown preprocessing step '{step.name}'. "
                    f"Known steps: {sorted(_REGISTRY)}"
                )
            df = fn(df, **step.kwargs)
        return df

    @classmethod
    def register(cls, name: str, fn: Callable) -> None:
        """Register a custom step function globally."""
        _REGISTRY[name] = fn
