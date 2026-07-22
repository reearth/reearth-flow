#!/usr/bin/env python3
"""Generate a test CityGML of a SINGLE feature carrying many solid-colour
textures — enough to overflow one atlas page and force multi-page packing.

All faces belong to one Building, so they extract into one quadtree cell and
pack into a single atlas build. Each face is a distinct 1x1 m quad with its own
`TILE_PX`-square solid-colour texture, mapped full-UV. With `N_FACES` textures
of `TILE_PX` pixels and no downsampling, packed area is `N_FACES * TILE_PX**2`;
once that exceeds `ATLAS_PX**2` the packer spills onto further pages. Defaults
give ~3 pages at the writer's default 2048 atlas.

Usage:
    python gen_multipage_test_gml.py <output_dir> [n_faces] [tile_px]

Writes <output_dir>/multipage.gml and <output_dir>/textures/tile_<i>.png, for:
    cargo run -p reearth-flow-action-sink --features new-geometry \\
        --example gml_to_3dtiles -- <output_dir>/multipage.gml <tiles_out>
(run with TEXEL_SIZE=0 so textures pack at native resolution).
"""

import colorsys
import random
import sys
from pathlib import Path

from PIL import Image, ImageDraw

# --- knobs -----------------------------------------------------------------
N_FACES = 48        # distinct textured quads on the single feature
# Texture size in pixels: each tile's width and height are drawn independently
# and uniformly from [MIN_PX, MAX_PX], so sizes are irregular, non-power-of-two
# and non-square, like real textures. Independent w/h also exercises the
# packer's per-axis UV remap. Seeded for reproducible output.
SEED = 42
MIN_PX, MAX_PX = 128, 1024
QUAD_M = 1.0        # quad edge length, metres
GAP_M = 0.5         # spacing between quads, metres
COLS = 8            # quads per row (grid layout keeps them in one small cell)
LAT0, LON0 = 35.6800, 139.7600  # anchor (near Tokyo), EPSG:6697
M_PER_DEG_LAT = 111320.0
# At the writer default 2048 atlas, tiles pack until ~2048**2 px per page is
# used, then spill; the mixed sizes below sum past that several times over.
# ---------------------------------------------------------------------------

import math


def m_per_deg_lon(lat):
    return M_PER_DEG_LAT * math.cos(math.radians(lat))


def geo(x, y, z):
    """Local ENU metres (x=east, y=north, z=up) -> 'lat lon height' string."""
    lat = LAT0 + y / M_PER_DEG_LAT
    lon = LON0 + x / m_per_deg_lon(LAT0)
    return f"{lat:.10f} {lon:.10f} {z:.4f}"


UV = [(0, 0), (1, 0), (1, 1), (0, 1)]


def quad_corners(x0, y0):
    """4 corners of a QUAD_M square at local (x0, y0, 0), CCW from bottom-left."""
    return [
        (x0, y0, 0.0),
        (x0 + QUAD_M, y0, 0.0),
        (x0 + QUAD_M, y0 + QUAD_M, 0.0),
        (x0, y0 + QUAD_M, 0.0),
    ]


def color(i, n):
    """Distinct solid colour per tile, evenly spaced around the hue circle."""
    r, g, b = colorsys.hsv_to_rgb(i / n, 0.7, 1.0)
    return (int(r * 255), int(g * 255), int(b * 255))


def main():
    if len(sys.argv) < 2:
        print(f"usage: {sys.argv[0]} <output_dir> [n_faces] [tile_px]",
              file=sys.stderr)
        sys.exit(1)
    out = Path(sys.argv[1])
    n_faces = int(sys.argv[2]) if len(sys.argv) > 2 else N_FACES
    (out / "textures").mkdir(parents=True, exist_ok=True)

    polygons = []   # (poly_id, ring_id, posList)
    targets = []    # (poly_id, ring_id, uv_str)
    rng = random.Random(SEED)
    packed = 0
    step = QUAD_M + GAP_M
    max_x = max_y = 0.0
    for i in range(n_faces):
        w = rng.randint(MIN_PX, MAX_PX)
        h = rng.randint(MIN_PX, MAX_PX)
        packed += w * h
        img = Image.new("RGB", (w, h), color(i, n_faces))
        # 1px black border: extrusion replicates edge pixels outward, so the
        # ring shows up as a black frame around each region in the atlas.
        ImageDraw.Draw(img).rectangle([0, 0, w - 1, h - 1], outline=(0, 0, 0))
        img.save(out / "textures" / f"tile_{i}.png")
        col, row = i % COLS, i // COLS
        x0, y0 = col * step, row * step
        corners = quad_corners(x0, y0)
        pid, rid = f"poly_{i}", f"ring_{i}"
        ring = corners + [corners[0]]  # close
        pos = " ".join(geo(*c) for c in ring)
        polygons.append((pid, rid, pos))
        uv = UV + [UV[0]]
        targets.append((pid, rid, " ".join(f"{u} {v}" for u, v in uv)))
        max_x = max(max_x, x0 + QUAD_M)
        max_y = max(max_y, y0 + QUAD_M)

    lo, hi = geo(0, 0, 0), geo(max_x, max_y, 0)
    ns = (
        'xmlns:core="http://www.opengis.net/citygml/2.0" '
        'xmlns:bldg="http://www.opengis.net/citygml/building/2.0" '
        'xmlns:app="http://www.opengis.net/citygml/appearance/2.0" '
        'xmlns:gml="http://www.opengis.net/gml"'
    )
    srs = "http://www.opengis.net/def/crs/EPSG/0/6697"

    p = ['<?xml version="1.0" encoding="UTF-8"?>', f"<core:CityModel {ns}>"]
    p += [
        "  <gml:boundedBy>",
        f'    <gml:Envelope srsName="{srs}" srsDimension="3">',
        f"      <gml:lowerCorner>{lo}</gml:lowerCorner>",
        f"      <gml:upperCorner>{hi}</gml:upperCorner>",
        "    </gml:Envelope>",
        "  </gml:boundedBy>",
    ]

    # appearance: one ParameterizedTexture per tile, each targeting its own face
    p += ["  <app:appearanceMember>", "    <app:Appearance>",
          "      <app:theme>rgbTexture</app:theme>"]
    for i, (pid, rid, uv_str) in enumerate(targets):
        p += [
            "      <app:surfaceDataMember>",
            "        <app:ParameterizedTexture>",
            f"          <app:imageURI>textures/tile_{i}.png</app:imageURI>",
            "          <app:mimeType>image/png</app:mimeType>",
            f'          <app:target uri="#{pid}">',
            "            <app:TexCoordList>",
            f'              <app:textureCoordinates ring="#{rid}">{uv_str}</app:textureCoordinates>',
            "            </app:TexCoordList>",
            "          </app:target>",
            "        </app:ParameterizedTexture>",
            "      </app:surfaceDataMember>",
        ]
    p += ["    </app:Appearance>", "  </app:appearanceMember>"]

    # single Building whose lod2Solid holds every textured quad
    p += [
        "  <core:cityObjectMember>",
        '    <bldg:Building gml:id="bldg_multipage">',
        "      <bldg:lod2Solid>",
        "        <gml:Solid>",
        "          <gml:exterior>",
        "            <gml:CompositeSurface>",
    ]
    for pid, rid, pos in polygons:
        p += [
            "              <gml:surfaceMember>",
            f'                <gml:Polygon gml:id="{pid}">',
            "                  <gml:exterior>",
            f'                    <gml:LinearRing gml:id="{rid}">',
            f"                      <gml:posList>{pos}</gml:posList>",
            "                    </gml:LinearRing>",
            "                  </gml:exterior>",
            "                </gml:Polygon>",
            "              </gml:surfaceMember>",
        ]
    p += [
        "            </gml:CompositeSurface>",
        "          </gml:exterior>",
        "        </gml:Solid>",
        "      </bldg:lod2Solid>",
        "    </bldg:Building>",
        "  </core:cityObjectMember>",
        "</core:CityModel>",
    ]

    gml_path = out / "multipage.gml"
    gml_path.write_text("\n".join(p), encoding="utf-8")

    print(f"wrote {gml_path}")
    print(f"{n_faces} faces, tile w/h ~ uniform[{MIN_PX}, {MAX_PX}] (seed {SEED})")
    print(f"packed area {packed} px^2 => ~{packed / 2048**2:.1f} page(s) at 2048 atlas")


if __name__ == "__main__":
    main()
