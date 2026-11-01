"""Extract a Qt .rcc resource bundle (format v1-3): rcc_extract.py BUNDLE OUTDIR"""
import os
import struct
import sys
import zlib

src, out = sys.argv[1], os.path.realpath(sys.argv[2])
d = open(src, "rb").read()
assert d[:4] == b"qres", "not a Qt resource bundle"
version, tree, data, names = struct.unpack_from(">IIII", d, 4)
entry = 14 if version == 1 else 22


def name(off):
    n, = struct.unpack_from(">H", d, names + off)
    return d[names + off + 6:names + off + 6 + 2 * n].decode("utf-16-be")


def walk(index, path):
    base = tree + index * entry
    name_off, flags = struct.unpack_from(">IH", d, base)
    nm = name(name_off) if index else ""
    here = os.path.join(path, nm) if nm else path
    if flags & 2:  # directory
        count, first = struct.unpack_from(">II", d, base + 6)
        for i in range(first, first + count):
            walk(i, here)
        return
    _, _, off = struct.unpack_from(">HHI", d, base + 6)
    size, = struct.unpack_from(">I", d, data + off)
    blob = d[data + off + 4:data + off + 4 + size]
    if flags & 1:
        blob = zlib.decompress(blob[4:])
    elif flags & 4:
        print("zstd-compressed, skipped:", here)
        return
    dest = os.path.realpath(os.path.join(out, here))
    assert dest.startswith(out + os.sep), here
    os.makedirs(os.path.dirname(dest), exist_ok=True)
    open(dest, "wb").write(blob)


os.makedirs(out, exist_ok=True)
walk(0, "")
print("done")
