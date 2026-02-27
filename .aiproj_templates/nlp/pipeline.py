# Noetron Studio — NLP Pipeline Template
# Toggle to No-Code view to edit visually.
# Domain: Natural Language Processing (sklearn + HuggingFace Transformers optional)

from noetron_runtime.data import Ingestor, Preprocessor, Step
from noetron_runtime.experiment import AutoLogger

# ── Stage 1: Load text corpus ─────────────────────────────────────────────────
# Expected CSV: two columns — "text" and "label"
df = Ingestor("data/corpus.csv").load()

texts  = df["text"].astype(str).tolist()
labels = df["label"].tolist()

# ── Stage 2: Vectorize ────────────────────────────────────────────────────────
from sklearn.feature_extraction.text import TfidfVectorizer  # noqa: E402
from sklearn.preprocessing import LabelEncoder              # noqa: E402
from sklearn.model_selection import train_test_split        # noqa: E402

le = LabelEncoder()
y  = le.fit_transform(labels)

vectorizer = TfidfVectorizer(max_features=10000, ngram_range=(1, 2), sublinear_tf=True)
X = vectorizer.fit_transform(texts)

X_train, X_test, y_train, y_test = train_test_split(X, y, test_size=0.2, random_state=42)

# ── Stage 3: Train ────────────────────────────────────────────────────────────
from sklearn.linear_model import LogisticRegression         # noqa: E402
from sklearn.metrics import accuracy_score, classification_report  # noqa: E402

with AutoLogger(".aiproj/experiments", run_name="nlp-tfidf-lr") as log:
    log.log_params({
        "vectorizer": "TfidfVectorizer",
        "max_features": 10000,
        "ngram_range": "(1,2)",
        "classifier": "LogisticRegression",
    })

    clf = LogisticRegression(max_iter=1000)
    clf.fit(X_train, y_train)

    # ── Stage 4: Evaluate ─────────────────────────────────────────────────────
    preds = clf.predict(X_test)
    acc   = accuracy_score(y_test, preds)
    log.log_metric("accuracy", float(acc))
    print(f"Accuracy: {acc:.4f}")
    print(classification_report(y_test, preds, target_names=le.classes_))

# ── Stage 5: Save ─────────────────────────────────────────────────────────────
import pickle, os  # noqa: E402
os.makedirs("models", exist_ok=True)
with open("models/nlp_clf.pkl", "wb") as f:
    pickle.dump({"vectorizer": vectorizer, "classifier": clf, "label_encoder": le}, f)
print("Model bundle saved to models/nlp_clf.pkl")
