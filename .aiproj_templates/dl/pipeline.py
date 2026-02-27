# Noetron Studio — Deep Learning Pipeline Template
# Toggle to No-Code view to edit visually.
# Domain: Deep Learning (PyTorch)

from noetron_runtime.data import Ingestor, Preprocessor, Step
from noetron_runtime.experiment import AutoLogger
from noetron_runtime.model import Exporter

import torch
import torch.nn as nn
from torch.utils.data import DataLoader, TensorDataset


# ── Stage 1: Load dataset ─────────────────────────────────────────────────────
df = Ingestor("data/train.csv").load()

# Separate features and target
TARGET_COLUMN = "label"
X = df.drop(columns=[TARGET_COLUMN]).values.astype("float32")
y = df[TARGET_COLUMN].values.astype("int64")

# ── Stage 2: Preprocess ───────────────────────────────────────────────────────
pre = Preprocessor([
    Step("drop_nulls"),
    Step("scale_standard"),
])
import pandas as pd  # noqa: E402
X_df = pre.fit_transform(pd.DataFrame(X))
X = X_df.values.astype("float32")

# Train/val split
from sklearn.model_selection import train_test_split  # noqa: E402
X_train, X_val, y_train, y_val = train_test_split(X, y, test_size=0.2, random_state=42)

train_ds = TensorDataset(torch.tensor(X_train), torch.tensor(y_train))
val_ds   = TensorDataset(torch.tensor(X_val),   torch.tensor(y_val))
train_dl = DataLoader(train_ds, batch_size=64, shuffle=True)
val_dl   = DataLoader(val_ds,   batch_size=128)

# ── Stage 3: Build model ──────────────────────────────────────────────────────
N_FEATURES = X_train.shape[1]
N_CLASSES  = int(y.max()) + 1
HIDDEN_DIM = 128
N_EPOCHS   = 20
LR         = 1e-3

model = nn.Sequential(
    nn.Linear(N_FEATURES, HIDDEN_DIM),
    nn.ReLU(),
    nn.Dropout(0.3),
    nn.Linear(HIDDEN_DIM, HIDDEN_DIM),
    nn.ReLU(),
    nn.Linear(HIDDEN_DIM, N_CLASSES),
)

optimizer = torch.optim.Adam(model.parameters(), lr=LR)
criterion = nn.CrossEntropyLoss()

# ── Stage 4: Train ────────────────────────────────────────────────────────────
with AutoLogger(".aiproj/experiments", run_name="dl-run") as log:
    log.log_params({
        "hidden_dim": HIDDEN_DIM,
        "n_epochs": N_EPOCHS,
        "lr": LR,
        "batch_size": 64,
        "architecture": "MLP",
    })

    for epoch in range(N_EPOCHS):
        model.train()
        for xb, yb in train_dl:
            optimizer.zero_grad()
            loss = criterion(model(xb), yb)
            loss.backward()
            optimizer.step()

    # ── Stage 5: Evaluate ─────────────────────────────────────────────────────
    model.eval()
    correct = total = 0
    with torch.no_grad():
        for xb, yb in val_dl:
            preds = model(xb).argmax(dim=1)
            correct += (preds == yb).sum().item()
            total   += len(yb)

    val_acc = correct / total
    log.log_metric("val_accuracy", val_acc)
    print(f"Val accuracy: {val_acc:.4f}")

# ── Stage 6: Export ───────────────────────────────────────────────────────────
torch.save(model.state_dict(), "models/dl_model.pt")
print("Model saved to models/dl_model.pt")
