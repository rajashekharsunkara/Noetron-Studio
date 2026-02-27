# Noetron Studio — Computer Vision Pipeline Template
# Toggle to No-Code view to edit visually.
# Domain: Computer Vision (PyTorch + torchvision)

from noetron_runtime.data import Ingestor
from noetron_runtime.experiment import AutoLogger

import torch
import torch.nn as nn
from torch.utils.data import DataLoader
from torchvision import datasets, transforms, models  # type: ignore

# ── Stage 1: Load image dataset ───────────────────────────────────────────────
# Expected layout: data/images/{class_name}/*.jpg
IMAGE_DIR  = "data/images"
IMG_SIZE   = 224
BATCH_SIZE = 32
N_EPOCHS   = 10
LR         = 1e-4

transform_train = transforms.Compose([
    transforms.RandomResizedCrop(IMG_SIZE),
    transforms.RandomHorizontalFlip(),
    transforms.ToTensor(),
    transforms.Normalize([0.485, 0.456, 0.406], [0.229, 0.224, 0.225]),
])
transform_val = transforms.Compose([
    transforms.Resize(256),
    transforms.CenterCrop(IMG_SIZE),
    transforms.ToTensor(),
    transforms.Normalize([0.485, 0.456, 0.406], [0.229, 0.224, 0.225]),
])

full_ds = datasets.ImageFolder(IMAGE_DIR, transform=transform_train)
n_val   = int(len(full_ds) * 0.2)
n_train = len(full_ds) - n_val
train_ds, val_ds = torch.utils.data.random_split(full_ds, [n_train, n_val])
val_ds.dataset.transform = transform_val  # type: ignore[attr-defined]

train_dl = DataLoader(train_ds, batch_size=BATCH_SIZE, shuffle=True,  num_workers=2)
val_dl   = DataLoader(val_ds,   batch_size=BATCH_SIZE, shuffle=False, num_workers=2)

# ── Stage 2: Build model (Transfer Learning — MobileNetV3) ───────────────────
N_CLASSES = len(full_ds.classes)
device    = torch.device("cuda" if torch.cuda.is_available() else "cpu")

backbone = models.mobilenet_v3_small(weights="IMAGENET1K_V1")
backbone.classifier[-1] = nn.Linear(backbone.classifier[-1].in_features, N_CLASSES)
model = backbone.to(device)

optimizer = torch.optim.AdamW(model.parameters(), lr=LR)
criterion = nn.CrossEntropyLoss()

# ── Stage 3: Train ────────────────────────────────────────────────────────────
with AutoLogger(".aiproj/experiments", run_name="cv-mobilenet") as log:
    log.log_params({
        "architecture": "MobileNetV3Small",
        "pretrained": True,
        "n_epochs": N_EPOCHS,
        "lr": LR,
        "batch_size": BATCH_SIZE,
        "img_size": IMG_SIZE,
        "n_classes": N_CLASSES,
        "classes": full_ds.classes,
    })

    for epoch in range(N_EPOCHS):
        model.train()
        for imgs, labels in train_dl:
            imgs, labels = imgs.to(device), labels.to(device)
            optimizer.zero_grad()
            criterion(model(imgs), labels).backward()
            optimizer.step()

    # ── Stage 4: Evaluate ─────────────────────────────────────────────────────
    model.eval()
    correct = total = 0
    with torch.no_grad():
        for imgs, labels in val_dl:
            imgs, labels = imgs.to(device), labels.to(device)
            preds = model(imgs).argmax(dim=1)
            correct += (preds == labels).sum().item()
            total   += len(labels)

    val_acc = correct / total
    log.log_metric("val_accuracy", val_acc)
    print(f"Classes: {full_ds.classes}")
    print(f"Val accuracy: {val_acc:.4f}")

# ── Stage 5: Export ───────────────────────────────────────────────────────────
import os  # noqa: E402
os.makedirs("models", exist_ok=True)
torch.save(model.state_dict(), "models/cv_model.pt")
print("Model saved to models/cv_model.pt")
