"""
Trainer — unified fit/evaluate wrapper for sklearn and compatible estimators.
Automatically logs params and metrics via AutoLogger if one is active.
"""

from __future__ import annotations

from typing import Any

from ..experiment.auto_logger import get_active


class Trainer:
    """Fit an estimator, evaluate, and auto-log everything.

    Usage::

        trainer = Trainer(RandomForestClassifier(n_estimators=100))
        trainer.fit_classify(X_train, y_train, X_test, y_test)
        print(trainer.metrics_)
    """

    def __init__(self, estimator: Any) -> None:
        self.estimator = estimator
        self.metrics_: dict[str, float] = {}

    # ── classification ───────────────────────────────────────────────────────

    def fit_classify(
        self,
        X_train: Any,
        y_train: Any,
        X_test: Any | None = None,
        y_test: Any | None = None,
    ) -> "Trainer":
        """Fit a classifier and compute accuracy + (optionally) F1."""
        from sklearn.metrics import accuracy_score, f1_score  # type: ignore

        log = get_active()
        if log:
            log.auto_log_sklearn(self.estimator)

        self.estimator.fit(X_train, y_train)

        if X_test is not None and y_test is not None:
            preds = self.estimator.predict(X_test)
            acc = float(accuracy_score(y_test, preds))
            try:
                f1 = float(f1_score(y_test, preds, average="weighted", zero_division=0))
            except Exception:
                f1 = 0.0
            self.metrics_ = {"accuracy": acc, "f1_weighted": f1}
            if log:
                log.log_metrics(self.metrics_)

        return self

    # ── regression ───────────────────────────────────────────────────────────

    def fit_regress(
        self,
        X_train: Any,
        y_train: Any,
        X_test: Any | None = None,
        y_test: Any | None = None,
    ) -> "Trainer":
        """Fit a regressor and compute RMSE + R²."""
        from sklearn.metrics import mean_squared_error, r2_score  # type: ignore
        import math

        log = get_active()
        if log:
            log.auto_log_sklearn(self.estimator)

        self.estimator.fit(X_train, y_train)

        if X_test is not None and y_test is not None:
            preds = self.estimator.predict(X_test)
            rmse = float(math.sqrt(mean_squared_error(y_test, preds)))
            r2 = float(r2_score(y_test, preds))
            self.metrics_ = {"rmse": rmse, "r2": r2}
            if log:
                log.log_metrics(self.metrics_)

        return self

    def predict(self, X: Any) -> Any:
        return self.estimator.predict(X)
