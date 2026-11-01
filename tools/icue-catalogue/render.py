"""Render the facts-only device catalogue JSON as Markdown: render.py CAT_JSON OUT_MD"""
import collections
import json
import sys

devs = json.load(open(sys.argv[1]))
FAMILY = {
    "Bragi": "Bragi — the same protocol coreglow speaks to the Commander CORE XT",
    "CLink": "CLink — older Commander / Platinum coolers / PSUs / Lighting Node",
    "Legacy": "Legacy — older keyboards, mice and headsets",
    "LLAccess": "LLAccess — memory (DRAM) via SMBus, needs a kernel driver",
}


def ids(d):
    seen = []
    for i in d["ids"]:
        s = f"{i['vid']}:{i['pid']}"
        if s not in seen:
            seen.append(s)
    return ", ".join(f"`{s}`" for s in seen) or "—"


def notes(d):
    n = []
    if d.get("IsWireless"):
        n.append("wireless")
    if d.get("IsLinkDevice"):
        n.append("iCUE LINK")
    if d.get("EmulatedDeviceType"):
        n.append(f"acts as {d['EmulatedDeviceType']}")
    if d.get("lister"):
        n.append(d["lister"].replace("-device-listing", " lister"))
    if d.get("coolers"):
        n.append(f"{len(d['coolers'])} cooler models")
    return ", ".join(n)


by_fam = collections.defaultdict(list)
for d in devs:
    by_fam[d.get("DeviceEnumerator") or "Other"].append(d)

out = [
    "# Corsair device catalogue",
    "",
    "Facts extracted from iCUE 5's device packages (`dev-*.cuepkg`, downloaded from Corsair's public",
    "module server): each package's `meta/auxiliary.json` and the device manifest inside its Qt",
    "resource bundle. Only names, USB IDs and protocol/geometry facts are recorded here; no Corsair",
    "files are redistributed. Regenerate with `tools/icue-catalogue/` (see its README).",
    "",
    f"{len(devs)} device entries. Columns: **USB ID** (vendor:product), **type** (iCUE's device-type),",
    "**HID protocol** (iCUE's `HidProto`, the driver flavour it picks).",
    "",
    "| Protocol family | Devices |",
    "|---|---|",
]
order = sorted(by_fam, key=lambda f: -len(by_fam[f]))
for f in order:
    out.append(f"| {FAMILY.get(f, f)} | {len(by_fam[f])} |")

# what coreglow could realistically support next
light = [d for d in devs if d.get("HidProto") == "Bragi_lighting_node" or d.get("led_channel_kinds")]
out += ["", "## Closest to coreglow", "",
        "Bragi lighting nodes share the Commander CORE XT's command set (wake, open/read/write files,",
        "geometry). Where the manifest lists LED channel kinds they are given as kind → geometry index,",
        "with the LED count per kind.", ""]
seen = set()
for d in light:
    key = (d["name"], ids(d))
    if key in seen:
        continue
    seen.add(key)
    out.append(f"### {d['name']} — {ids(d)}")
    out.append("")
    out.append(f"type `{d['type']}`, protocol `{d.get('HidProto')}`" + (f", {notes(d)}" if notes(d) else ""))
    kinds = d.get("led_channel_kinds") or {}
    counts = d.get("led_counts") or {}
    if kinds:
        out += ["", "| LED channel kind | geometry index | LEDs |", "|---|---|---|"]
        for k, v in sorted(kinds.items(), key=lambda kv: (kv[1], kv[0])):
            out.append(f"| {k} | {v} | {counts.get(k, '')} |")
    if d.get("coolers"):
        out += ["", "Coolers: " + ", ".join(f"{n} ({r} mm)" for n, r in d["coolers"])]
    out.append("")

for f in order:
    out += [f"## {FAMILY.get(f, f)}", "", "| Device | USB ID | type | HID protocol | notes |", "|---|---|---|---|---|"]
    for d in sorted(by_fam[f], key=lambda d: (d["type"] or "", (d["name"] or "").lower())):
        out.append(f"| {d['name']} | {ids(d)} | {d['type']} | {d.get('HidProto') or ''} | {notes(d)} |")
    out.append("")

open(sys.argv[2], "w").write("\n".join(out))
print("wrote", sys.argv[2], len(out), "lines")
