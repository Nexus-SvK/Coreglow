"""Download every dev-* package listed in iCUE's cuepkg.registry (one at a time).

usage: fetch_devices.py REGISTRY OUTDIR
Only URLs on www3.corsair.com from the registry are fetched; existing files are skipped.
"""
import os
import sqlite3
import sys
import time
import urllib.request

reg, out = sys.argv[1], sys.argv[2]
os.makedirs(out, exist_ok=True)
db = sqlite3.connect(f"file:{reg}?mode=ro", uri=True)
rows = db.execute(
    "select p.name, p.download_size, r.location from packages p join repository_info r on r.package_id = p.package_id "
    "where p.type = 'repository' and p.name like 'dev-%' order by p.name"
).fetchall()
prefix = "https://www3.corsair.com/software/CUE_V5/public/modules/windows/packages/"
ok = fail = skip = 0
for name, size, url in rows:
    if not url.startswith(prefix) or "/" in url[len(prefix):]:
        print("skipping unexpected URL:", url, flush=True)
        continue
    dest = os.path.join(out, url[len(prefix):])
    if os.path.exists(dest) and (size <= 0 or os.path.getsize(dest) == size):
        skip += 1
        continue
    for attempt in range(3):
        try:
            req = urllib.request.Request(url, headers={"User-Agent": "coreglow-research (device catalogue)"})
            with urllib.request.urlopen(req, timeout=60) as r, open(dest + ".part", "wb") as f:
                while chunk := r.read(1 << 16):
                    f.write(chunk)
            os.replace(dest + ".part", dest)
            ok += 1
            break
        except Exception as e:
            print(f"{name}: attempt {attempt + 1} failed: {e}", flush=True)
            time.sleep(3)
    else:
        fail += 1
    time.sleep(0.3)
print(f"done: {ok} downloaded, {skip} already there, {fail} failed, of {len(rows)}", flush=True)
