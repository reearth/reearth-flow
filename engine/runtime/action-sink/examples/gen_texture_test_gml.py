#!/usr/bin/env python3
"""Generate a test CityGML of incrementally sized cubes that all share the same
texture image data (one sharp public-domain photo) but as separate texture
files. The full image is mapped onto each cube face, so the smallest cube packs
the same pixels into the smallest area -> highest texel density (finest
metres-per-pixel). Raising `TEXEL_SIZE` in the gml_to_3dtiles run therefore
blurs the smallest cube first, then progressively larger ones.

Usage:
    python gen_texture_test_gml.py <output_dir>

Writes <output_dir>/cubes.gml and <output_dir>/textures/cube_<i>.png,
ready for:
    cargo run -p reearth-flow-action-sink --features new-geometry \\
        --example gml_to_3dtiles -- <output_dir>/cubes.gml <tiles_out>
"""

import io
import math
import sys
import urllib.request
from pathlib import Path

from PIL import Image

# --- knobs -----------------------------------------------------------------
SIZES_M = [1, 2, 4, 8, 16]  # cube edge length, metres (incremental)
GAP_M = 4.0                 # spacing between cubes, metres
IMG_PX = 512                # texture edge in pixels (SAME for every cube)
LAT0, LON0 = 35.6800, 139.7600  # anchor (near Tokyo), EPSG:6697
M_PER_DEG_LAT = 111320.0
# Sharp, detailed, public-domain (NASA) photo — Aldrin on the Moon, Apollo 11.
TEXTURE_URL = (
    "https://commons.wikimedia.org/wiki/Special:FilePath/"
    "Aldrin_Apollo_11_original.jpg?width=1024"
)
# ---------------------------------------------------------------------------


def m_per_deg_lon(lat):
    return M_PER_DEG_LAT * math.cos(math.radians(lat))


def load_texture():
    """Download the source photo, centre-crop to square, resize to IMG_PX."""
    req = urllib.request.Request(
        TEXTURE_URL, headers={"User-Agent": "reearth-flow-texture-test/1.0"}
    )
    data = urllib.request.urlopen(req, timeout=60).read()
    img = Image.open(io.BytesIO(data)).convert("RGB")
    w, h = img.size
    side = min(w, h)
    left, top = (w - side) // 2, (h - side) // 2
    return img.crop((left, top, left + side, top + side)).resize(
        (IMG_PX, IMG_PX), Image.LANCZOS
    )


def geo(lat0, lon0, x, y, z):
    """Local ENU metres (x=east, y=north, z=up) -> 'lat lon height' string."""
    lat = lat0 + y / M_PER_DEG_LAT
    lon = lon0 + x / m_per_deg_lon(lat0)
    return f"{lat:.10f} {lon:.10f} {z:.4f}"


# 6 cube faces as (corner-index quad), outward CCW; UV maps each to the unit
# square so every face shows the full photo.
FACES = [
    ("south", [0, 1, 5, 4]),
    ("east", [1, 2, 6, 5]),
    ("north", [2, 3, 7, 6]),
    ("west", [3, 0, 4, 7]),
    ("bottom", [0, 3, 2, 1]),
    ("top", [4, 5, 6, 7]),
]
UV = [(0, 0), (1, 0), (1, 1), (0, 1)]


def cube_corners(x0, L):
    """8 corners of a cube with near-bottom at local (x0, 0, 0), edge L."""
    return [
        (x0, 0, 0), (x0 + L, 0, 0), (x0 + L, L, 0), (x0, L, 0),
        (x0, 0, L), (x0 + L, 0, L), (x0 + L, L, L), (x0, L, L),
    ]


def main():
    if len(sys.argv) < 2:
        print(f"usage: {sys.argv[0]} <output_dir>", file=sys.stderr)
        sys.exit(1)
    out = Path(sys.argv[1])
    (out / "textures").mkdir(parents=True, exist_ok=True)

    polygons = []       # (poly_id, ring_id, posList)
    tex_targets = []    # (cube_idx, [(ring_id, uv_str), ...])
    x = 0.0
    max_x = 0.0
    max_z = 0.0
    texture = load_texture()
    for i, L in enumerate(SIZES_M):
        texture.save(out / "textures" / f"cube_{i}.png")
        corners = cube_corners(x, L)
        targets = []
        for name, quad in FACES:
            pid = f"poly_c{i}_{name}"
            rid = f"line_c{i}_{name}"
            ring = quad + [quad[0]]  # close
            pos = " ".join(geo(LAT0, LON0, *corners[k]) for k in ring)
            polygons.append((pid, rid, pos))
            uv = UV + [UV[0]]
            uv_str = " ".join(f"{u} {v}" for u, v in uv)
            targets.append((pid, rid, uv_str))
        tex_targets.append((i, targets))
        max_x = x + L
        max_z = max(max_z, L)
        x += L + GAP_M

    # bounds (lat lon height)
    lo = geo(LAT0, LON0, 0, 0, 0)
    hi = geo(LAT0, LON0, max_x, max(SIZES_M), max_z)

    ns = (
        'xmlns:core="http://www.opengis.net/citygml/2.0" '
        'xmlns:bldg="http://www.opengis.net/citygml/building/2.0" '
        'xmlns:app="http://www.opengis.net/citygml/appearance/2.0" '
        'xmlns:gml="http://www.opengis.net/gml"'
    )
    srs = "http://www.opengis.net/def/crs/EPSG/0/6697"

    parts = ['<?xml version="1.0" encoding="UTF-8"?>']
    parts.append(f"<core:CityModel {ns}>")
    parts.append("  <gml:boundedBy>")
    parts.append(f'    <gml:Envelope srsName="{srs}" srsDimension="3">')
    parts.append(f"      <gml:lowerCorner>{lo}</gml:lowerCorner>")
    parts.append(f"      <gml:upperCorner>{hi}</gml:upperCorner>")
    parts.append("    </gml:Envelope>")
    parts.append("  </gml:boundedBy>")

    # appearance: one ParameterizedTexture per cube, targeting its 6 faces
    parts.append("  <app:appearanceMember>")
    parts.append("    <app:Appearance>")
    parts.append("      <app:theme>rgbTexture</app:theme>")
    for i, targets in tex_targets:
        parts.append("      <app:surfaceDataMember>")
        parts.append("        <app:ParameterizedTexture>")
        parts.append(f"          <app:imageURI>textures/cube_{i}.png</app:imageURI>")
        parts.append("          <app:mimeType>image/png</app:mimeType>")
        for pid, rid, uv_str in targets:
            parts.append(f'          <app:target uri="#{pid}">')
            parts.append("            <app:TexCoordList>")
            parts.append(
                f'              <app:textureCoordinates ring="#{rid}">{uv_str}</app:textureCoordinates>'
            )
            parts.append("            </app:TexCoordList>")
            parts.append("          </app:target>")
        parts.append("        </app:ParameterizedTexture>")
        parts.append("      </app:surfaceDataMember>")
    parts.append("    </app:Appearance>")
    parts.append("  </app:appearanceMember>")

    # one Building per cube, lod2Solid of 6 polygons
    pi = 0
    for i, L in enumerate(SIZES_M):
        parts.append("  <core:cityObjectMember>")
        parts.append(f'    <bldg:Building gml:id="bldg_cube_{i}">')
        parts.append("      <bldg:lod2Solid>")
        parts.append("        <gml:Solid>")
        parts.append("          <gml:exterior>")
        parts.append("            <gml:CompositeSurface>")
        for _ in FACES:
            pid, rid, pos = polygons[pi]
            pi += 1
            parts.append("              <gml:surfaceMember>")
            parts.append(f'                <gml:Polygon gml:id="{pid}">')
            parts.append("                  <gml:exterior>")
            parts.append(f'                    <gml:LinearRing gml:id="{rid}">')
            parts.append(f"                      <gml:posList>{pos}</gml:posList>")
            parts.append("                    </gml:LinearRing>")
            parts.append("                  </gml:exterior>")
            parts.append("                </gml:Polygon>")
            parts.append("              </gml:surfaceMember>")
        parts.append("            </gml:CompositeSurface>")
        parts.append("          </gml:exterior>")
        parts.append("        </gml:Solid>")
        parts.append("      </bldg:lod2Solid>")
        parts.append("    </bldg:Building>")
        parts.append("  </core:cityObjectMember>")

    parts.append("</core:CityModel>")

    gml_path = out / "cubes.gml"
    gml_path.write_text("\n".join(parts), encoding="utf-8")

    print(f"wrote {gml_path}")
    print(f"texture: {IMG_PX}px photo (identical on every cube), "
          f"{len(SIZES_M)} files in {out / 'textures'}")
    print("cube edge (m) -> native m/px (blurs once TEXEL_SIZE exceeds it):")
    for L in SIZES_M:
        print(f"  {L:>2} m -> {L / IMG_PX:.5f} m/px")


if __name__ == "__main__":
    main()
