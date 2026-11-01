"""Build a facts-only catalogue of Corsair devices from downloaded iCUE packages.

usage: catalogue.py PACKAGES_DIR OUT_JSON
Reads each .cuepkg (zip): meta/auxiliary.json (names, USB ids, lister) and the
device manifest inside the .rcc bundle (device type, lighting channel kinds).
Writes only facts (names, ids, types, counts), never Corsair's files.
"""
import glob
import json
import os
import struct
import sys
import zipfile
import zlib


def rcc_files(d):
    """{path: bytes} of a Qt .rcc bundle (format 1-3), only small text-like files."""
    if d[:4] != b"qres":
        return {}
    version, tree, data, names = struct.unpack_from(">IIII", d, 4)
    entry = 14 if version == 1 else 22
    out = {}

    def name(off):
        n, = struct.unpack_from(">H", d, names + off)
        return d[names + off + 6:names + off + 6 + 2 * n].decode("utf-16-be")

    def walk(i, path):
        base = tree + i * entry
        noff, flags = struct.unpack_from(">IH", d, base)
        here = path + "/" + name(noff) if i else ""
        if flags & 2:
            count, first = struct.unpack_from(">II", d, base + 6)
            for j in range(first, first + count):
                walk(j, here)
            return
        if not here.endswith((".cuebmf", ".json")):
            return
        _, _, off = struct.unpack_from(">HHI", d, base + 6)
        size, = struct.unpack_from(">I", d, data + off)
        blob = d[data + off + 4:data + off + 4 + size]
        out[here] = zlib.decompress(blob[4:]) if flags & 1 else blob

    walk(0, "")
    return out


def manifest_facts(m):
    c = m.get("common", {})
    facts = {k: c.get(k) for k in ("DeviceEnumerator", "HidProto", "DeviceType", "SdkDeviceType", "EmulatedDeviceType",
                                    "IsWireless", "IsLinkDevice", "DramProtocolType") if c.get(k)}
    geo = c.get("LedChannelTypeToGeometryIndexMap")
    if geo:
        facts["led_channel_kinds"] = geo
    tpl = c.get("DynamicGeometryTemplates") or {}
    leds = {}
    for k, v in tpl.items():
        views = (v or {}).get("ledViews") or {}
        n = {len(view.get("ledsGeometry", [])) for view in views.values()}
        if n:
            leds[k] = max(n)
    if leds:
        facts["led_counts"] = leds
    chans = c.get("SupportedChannelConfigurations")
    if chans:
        facts["channels"] = len(chans)
    if "SupportsLedAutoDetection" in c:
        facts["led_autodetection"] = c["SupportsLedAutoDetection"]
    return facts


devices = []
skipped = []
for path in sorted(glob.glob(os.path.join(sys.argv[1], "dev-*.cuepkg"))):
    pkg = os.path.basename(path).split("_")[0]
    try:
        z = zipfile.ZipFile(path)
    except zipfile.BadZipFile:
        print("not a zip:", path)
        continue
    try:
        aux = json.loads(z.read("meta/auxiliary.json"))
    except KeyError:
        aux = {}
    manifests = {}
    for n in z.namelist():
        if n.endswith(".rcc"):
            for p, b in rcc_files(z.read(n)).items():
                if p.endswith(".cuebmf"):
                    try:
                        manifests[p] = json.loads(b)
                    except ValueError:
                        pass
    if not aux and not manifests:
        skipped.append(pkg)  # placeholder for an obsolete package
        continue
    mf = manifest_facts(next(iter(manifests.values()))) if manifests else {}
    for d in aux.get("devices", []) or [{}]:
        devices.append({
            "package": pkg,
            "name": d.get("display-name"),
            "type": d.get("device-type"),
            "lister": d.get("lister-stage2") or d.get("lister"),
            "ids": [
                {"vid": f"{i['usb-vid']:04x}", "pid": f"{i['usb-pid']:04x}", "cue_mid": i.get("cue-mid")}
                for i in d.get("ids", []) if "usb-vid" in i and "usb-pid" in i
            ],
            **({"coolers": sorted({(x["name"], x.get("radiator-size")) for x in d["cooler-names"]})}
               if d.get("cooler-names") else {}),
            **mf,
        })
json.dump(devices, open(sys.argv[2], "w"), indent=1, ensure_ascii=False)
print(len(skipped), "placeholders skipped;", len(devices), "devices from", len(glob.glob(os.path.join(sys.argv[1], 'dev-*.cuepkg'))), "packages")
