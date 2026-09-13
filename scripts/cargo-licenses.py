#!/usr/bin/env python3
"""Collect upstream license files and a manifest for vendored Cargo sources."""
import json
import shutil
import sys
import tomllib
from pathlib import Path

root, output = map(Path, sys.argv[1:])
output.mkdir(parents=True, exist_ok=True)
manifest = []
for package in sorted(root.iterdir()):
    cargo = package / "Cargo.toml"
    if not cargo.is_file():
        continue
    data = tomllib.loads(cargo.read_text())["package"]
    manifest.append({key: data.get(key) for key in ("name", "version", "license", "repository")})
    candidates = {p for p in package.iterdir() if p.name.lower().startswith(("license", "licence", "copying", "notice"))}
    if data.get("license-file"):
        candidates.add(package / data["license-file"])
    for source in candidates:
        if not source.resolve().is_relative_to(package.resolve()):
            raise SystemExit(f"License outside package: {source}")
        if source.is_file():
            target = output / package.name / source.name
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(source, target)
        elif source.is_dir():
            shutil.copytree(source, output / package.name / source.name, dirs_exist_ok=True)
(output / "packages.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
