#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.."
echo 'Synthetic search, development profile; no user data is read.'
cargo run --locked --example search_benchmark
cargo build --locked
python3 - <<'PY'
import platform,statistics,subprocess,time
print('OCR fixtures; fresh processes, startup included; architecture:',platform.machine())
for language in ('eng','fra','fra+eng'):
    fixture='tests/fixtures/ocr.png' if language=='eng' else 'tests/fixtures/ocr-french.png'
    samples=[]
    for _ in range(3):
        start=time.perf_counter()
        p=subprocess.run(['target/debug/nebula-paste','--ocr',fixture,language],capture_output=True,text=True,check=True)
        assert ('NEBULA PASTE 12345' in p.stdout) if language=='eng' else ('Été' in p.stdout and 'café' in p.stdout)
        samples.append((time.perf_counter()-start)*1000)
    print(language,'median_ms=',round(statistics.median(samples),2),'min_ms=',round(min(samples),2),'max_ms=',round(max(samples),2))
PY
