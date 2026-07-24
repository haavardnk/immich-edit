import struct


def s15f16(x):
    return struct.pack(">i", round(x * 65536.0))


def mat_inv(m):
    a, b, c = m[0]
    d, e, f = m[1]
    g, h, i = m[2]
    det = a * (e * i - f * h) - b * (d * i - f * g) + c * (d * h - e * g)
    inv = 1.0 / det
    return [
        [(e * i - f * h) * inv, (c * h - b * i) * inv, (b * f - c * e) * inv],
        [(f * g - d * i) * inv, (a * i - c * g) * inv, (c * d - a * f) * inv],
        [(d * h - e * g) * inv, (b * g - a * h) * inv, (a * e - b * d) * inv],
    ]


def mat_mul(a, b):
    return [[sum(a[r][k] * b[k][c] for k in range(3)) for c in range(3)] for r in range(3)]


def mat_vec(m, v):
    return [sum(m[r][c] * v[c] for c in range(3)) for r in range(3)]


def rgb_to_xyz(primaries, white):
    cols = []
    for (x, y) in primaries:
        cols.append([x / y, 1.0, (1.0 - x - y) / y])
    m = [[cols[0][r], cols[1][r], cols[2][r]] for r in range(3)]
    xw, yw = white
    w = [xw / yw, 1.0, (1.0 - xw - yw) / yw]
    s = mat_vec(mat_inv(m), w)
    return [[m[r][c] * s[c] for c in range(3)] for r in range(3)]


BRADFORD = [
    [0.8951, 0.2664, -0.1614],
    [-0.7502, 1.7135, 0.0367],
    [0.0389, -0.0685, 1.0296],
]


def bradford(src_w, dst_w):
    def wp(w):
        x, y = w
        return [x / y, 1.0, (1.0 - x - y) / y]

    s = mat_vec(BRADFORD, wp(src_w))
    d = mat_vec(BRADFORD, wp(dst_w))
    diag = [[d[r] / s[r] * BRADFORD[r][c] for c in range(3)] for r in range(3)]
    return mat_mul(mat_inv(BRADFORD), diag)


P3 = [(0.680, 0.320), (0.265, 0.690), (0.150, 0.060)]
D65 = (0.3127, 0.3290)
D50 = (0.3457, 0.3585)

m_d65 = rgb_to_xyz(P3, D65)
chad = bradford(D65, D50)
m_d50 = mat_mul(chad, m_d65)


def xyz_type(x, y, z):
    return b"XYZ \x00\x00\x00\x00" + s15f16(x) + s15f16(y) + s15f16(z)


def para_srgb():
    body = b"para\x00\x00\x00\x00"
    body += struct.pack(">H", 3) + b"\x00\x00"
    for v in (2.4, 1.0 / 1.055, 0.055 / 1.055, 1.0 / 12.92, 0.04045):
        body += s15f16(v)
    return body


def mluc(text):
    u = text.encode("utf-16-be")
    body = b"mluc\x00\x00\x00\x00"
    body += struct.pack(">I", 1)
    body += struct.pack(">I", 12)
    body += b"enUS"
    body += struct.pack(">I", len(u))
    body += struct.pack(">I", 28)
    body += u
    return body


def chad_type(m):
    body = b"sf32\x00\x00\x00\x00"
    for r in range(3):
        for c in range(3):
            body += s15f16(m[r][c])
    return body


def pad4(b):
    while len(b) % 4:
        b += b"\x00"
    return b


desc = mluc("Display P3")
cprt = mluc("Public Domain")
wtpt = xyz_type(D50[0] / D50[1], 1.0, (1.0 - D50[0] - D50[1]) / D50[1])
rxyz = xyz_type(m_d50[0][0], m_d50[1][0], m_d50[2][0])
gxyz = xyz_type(m_d50[0][1], m_d50[1][1], m_d50[2][1])
bxyz = xyz_type(m_d50[0][2], m_d50[1][2], m_d50[2][2])
trc = para_srgb()
chad_tag = chad_type(chad)

tags = [
    (b"desc", desc),
    (b"cprt", cprt),
    (b"wtpt", wtpt),
    (b"chad", chad_tag),
    (b"rXYZ", rxyz),
    (b"gXYZ", gxyz),
    (b"bXYZ", bxyz),
    (b"rTRC", trc),
    (b"gTRC", trc),
    (b"bTRC", trc),
]

header = bytearray(128)
header[8:12] = struct.pack(">I", 0x04300000)
header[12:16] = b"mntr"
header[16:20] = b"RGB "
header[20:24] = b"XYZ "
header[36:40] = b"acsp"
header[64:68] = struct.pack(">I", 0)
header[68:80] = s15f16(0.9642) + s15f16(1.0) + s15f16(0.8249)

tag_count = len(tags)
table_size = 4 + tag_count * 12
data_start = 128 + table_size
data_start = (data_start + 3) & ~3

offsets = {}
data = b""
cursor = data_start
seen = {}
entries = []
for sig, body in tags:
    key = bytes(body)
    if key in seen:
        off, size = seen[key]
    else:
        off = cursor
        size = len(body)
        padded = pad4(body)
        data += padded
        cursor += len(padded)
        seen[key] = (off, size)
    entries.append((sig, off, size))

table = struct.pack(">I", tag_count)
for sig, off, size in entries:
    table += sig + struct.pack(">I", off) + struct.pack(">I", size)

profile = bytes(header) + table
profile = pad4(profile)
profile += data
profile = profile[:128] + profile[128:]
profile = bytearray(profile)
profile[0:4] = struct.pack(">I", len(profile))

with open("display-p3.icc", "wb") as f:
    f.write(profile)
print("wrote display-p3.icc", len(profile), "bytes")
