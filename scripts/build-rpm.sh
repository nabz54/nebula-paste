#!/usr/bin/env bash
# Prepare source archives with network access; rpmbuild itself builds offline.
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.."
for tool in git cargo rpmbuild python3 tar sha256sum; do
    command -v "$tool" >/dev/null || { echo "Missing build tool: $tool" >&2; exit 1; }
done
git diff --quiet && git diff --cached --quiet || { echo 'Commit source changes before packaging.' >&2; exit 1; }
output_dir="$(realpath -m -- "${1:-dist/rpm}")"
mkdir -p -- "$output_dir"
rpm_stage="$(mktemp -d)"
trap 'rm -rf -- "$rpm_stage"' EXIT
mkdir -p "$rpm_stage/source" "$rpm_stage/rpmbuild/SOURCES"
git archive HEAD | tar -x -C "$rpm_stage/source"
cd "$rpm_stage/source"
version="$(python3 -c 'import tomllib; print(tomllib.load(open("Cargo.toml", "rb"))["package"]["version"])')"
tar -czf "$rpm_stage/rpmbuild/SOURCES/nebula-paste-$version.tar.gz" --transform="s,^\.,nebula-paste-$version," .
mkdir -p .cargo
cargo vendor --locked vendor-cargo > .cargo/config.toml
python3 scripts/cargo-licenses.py vendor-cargo cargo-licenses
tar -czf "$rpm_stage/rpmbuild/SOURCES/nebula-paste-cargo-vendor.tar.gz" vendor-cargo .cargo cargo-licenses
rpmbuild -ba --define "_topdir $rpm_stage/rpmbuild" packaging/nebula-paste.spec
find "$rpm_stage/rpmbuild/RPMS" "$rpm_stage/rpmbuild/SRPMS" -type f -name '*.rpm' -exec cp -- {} "$output_dir/" \;
cd "$output_dir"
# GitHub sanitizes '~' in asset names. Normalize before hashing so downloads
# and SHA256SUMS use the same names; the RPM's internal Version is unchanged.
python3 - <<'PYTHON'
from pathlib import Path
for package in Path('.').glob('*.rpm'):
    if '~' in package.name:
        destination = package.with_name(package.name.replace('~', '.'))
        if destination.exists():
            raise SystemExit(f'Refusing to overwrite {destination}')
        package.rename(destination)
PYTHON
sha256sum -- *.rpm > SHA256SUMS
printf 'Packages: %s\n' "$output_dir"
