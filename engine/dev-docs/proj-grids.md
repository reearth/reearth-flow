# PROJ grids

Reprojection is backed by PROJ. Horizontal work (map projections, datum shifts,
geographic ↔ geocentric) is formula-driven and always available, but a **change
of vertical datum is not**: the difference between a gravity-related height and
an ellipsoidal one is a measured field, published as a geoid grid. Without the
grid, PROJ has no operation to offer.

The engine refuses PROJ's ballpark fallback (`ALLOW_BALLPARK=NO`), because a
ballpark passes the height through untouched — 16 to 51 m wrong anywhere in
Japan, up to ±107 m worldwide — with no indication anything went missing. A
missing grid is therefore a hard error, never a quiet wrong answer.

## What ships in the binary

`runtime/geometry/grids/` holds 27 grid files (22 MiB) that are compiled into
the binary with `include_bytes!`: current-generation geoid models, one per
national vertical datum, for Japan and Europe, plus the global EGM96 fallback.
`MANIFEST.tsv` records each file's size, SHA-256, publisher and licence;
`NOTICE.md` is the attribution notice that must accompany redistribution.

Two models cover Japan. When both are present PROJ prefers **GSIGEO2024**
(`jp_gsi_jpgeo2024.tif`), the only one of the two whose operation declares an
accuracy (0.03 m); **GSIGEO2011** (`jp_gsi_gsigeo2011.tif`) is what the legacy
pipeline used and puts a central Tokyo point about 10 cm lower. Removing
`jp_gsi_jpgeo2024.tif` from the set is all it takes to go back to GSIGEO2011.

At first use the grids are unpacked into a cache directory and PROJ is pointed
at it. Nothing needs to be installed, and no network access is involved.

## Everything else

PROJ-data publishes 161 geoid grids (554 MiB), 150 horizontal NTv2 grids
(196 MiB) and 42 deformation models; the United States, Canada, Australia and
EGM2008 alone are 135 MiB, and historical realizations (GEOID12B, AUSGeoid09,
RAF18, `HT2_1997`, …) many times that. Those stay out of the binary and are
supplied per deployment:

```bash
mkdir -p /srv/proj-grids
curl -O --output-dir /srv/proj-grids https://cdn.proj.org/us_noaa_g2018u0.tif
export FLOW_PROJ_GRID_DIR=/srv/proj-grids
```

`FLOW_PROJ_GRID_DIR` takes a path list (`:`-separated on Unix, `;` on Windows)
and is searched **before** the embedded grids, so it can also override one of
them. Grid file names come from `proj.db`; the error raised for an unsupported
transformation names the variable, and <https://cdn.proj.org> serves every grid
by name.

| Variable | Effect |
|---|---|
| `FLOW_PROJ_GRID_DIR` | Extra grid directories, searched first |
| `FLOW_PROJ_GRID_CACHE_DIR` | Where the embedded grids are unpacked. Defaults to the user cache directory, then the temporary directory. Needs to be writable |

## Changing the embedded set

Edit the `GRIDS` list in `runtime/geometry/grids/update.sh` and run it: it
downloads each grid, verifies it against the SHA-256 the CDN publishes, and
regenerates `MANIFEST.tsv` and `NOTICE.md`. Then mirror the list in
`EMBEDDED_GRIDS` in `runtime/geometry/src/ops/reproject/grids.rs` — a test fails
if the two drift apart, and another fails if a grid is not one `proj.db` knows
about, which is the usual sign that a name is newer than the pinned PROJ.

Grids carry licences of their own: mostly CC-BY-4.0, some public domain, CC0,
OGL-Canada, Etalab or CC-BY-SA-4.0. `update.sh` copies the licence PROJ-data
records for each file into `NOTICE.md`, and fails rather than embedding a file
it cannot find a licence for. The Japanese grids additionally carry the
Geospatial Information Authority of Japan's permission under the Survey Act,
reproduced in the notice.
