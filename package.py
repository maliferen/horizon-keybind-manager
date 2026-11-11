#!/usr/bin/env python3
"""Build the deterministic Horizon bundle for Horizon Keybind Manager."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import tempfile
import tomllib
import zipfile

ROOT = Path(__file__).resolve().parent


def package(output: Path) -> Path:
    compat = json.loads((ROOT / "compatibility.json").read_text(encoding="utf-8"))
    bundle, = (ROOT / "mixins").glob("*/horizon-mixin.toml")
    manifest = tomllib.loads(bundle.read_text(encoding="utf-8"))
    mod = manifest["mod"]
    cargo = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))
    if mod["id"] != bundle.parent.name or mod["version"] != cargo["package"]["version"]:
        raise ValueError("Bundle directory, mod ID and package version must agree")
    if manifest["compatibility"]["horizon_revision"] != compat["horizon_revision"]:
        raise ValueError("The manifest and compatibility.json must pin the same Horizon revision")

    metadata = {
        "schema": 1,
        "id": mod["id"],
        "version": mod["version"],
        "game": compat["game"],
        "horizon": compat["horizon_revision"],
        "mixins": compat["horizon_mixins_revision"],
        "hooks": compat["hooks"],
    }
    files = {"horizon-native.json": (json.dumps(metadata, indent=2, sort_keys=True) + "\n").encode()}
    prefix = f"mixins/{mod['id']}/"
    for path in bundle.parent.rglob("*"):
        if path.is_file() and path.suffix in {".rs", ".toml"}:
            files[prefix + path.relative_to(bundle.parent).as_posix()] = path.read_bytes()
    files[prefix + "LICENSE"] = (ROOT / "LICENSE").read_bytes()

    output.mkdir(parents=True, exist_ok=True)
    destination = output / f"{mod['id']}-{mod['version']}.hnmod"
    with tempfile.NamedTemporaryFile(dir=output, suffix=".tmp", delete=False) as temp:
        staged = Path(temp.name)
    try:
        with zipfile.ZipFile(staged, "w") as archive:
            for name, content in sorted(files.items()):
                entry = zipfile.ZipInfo(name, (2026, 1, 1, 0, 0, 0))
                entry.compress_type = zipfile.ZIP_DEFLATED
                entry.external_attr = 0o100644 << 16
                archive.writestr(entry, content)
        os.replace(staged, destination)
    finally:
        staged.unlink(missing_ok=True)
    print(f"{destination.resolve()}  SHA-256 {hashlib.sha256(destination.read_bytes()).hexdigest()}")
    return destination


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, default=ROOT / "builds")
    package(parser.parse_args().out)
