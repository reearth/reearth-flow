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

## What does not ship

PROJ-data publishes far more than the binary carries: 161 geoid grids
(554 MiB), 150 horizontal grids (196 MiB) and 42 deformation models, counting
historical realizations (GEOID12B, AUSGeoid09, RAF18, `HT2_1997`), tidal and
chart datums, overseas territories, and alternative realizations on other
reference frames. Korea, China, Italy, Brazil and much of Asia and Africa
publish national geoid models that PROJ-data cannot redistribute at all.

None of that is available out of the box, and shipping the whole catalogue with a
deployment is not addressed here. A transformation needing one of those grids
fails, naming the grid PROJ wanted and how to supply it.

## Supplying grids the embedded set lacks

`FLOW_PROJ_GRID_DIR` names directories of grid files, searched ahead of the
embedded set. It takes a path list, so several sources compose:

```bash
export FLOW_PROJ_GRID_DIR=/run/grids/workspace:/srv/proj-grids
```

Those directories are only ever read. The one path that writes — unpacking the
embedded grids — goes to a cache directory of its own, and is skipped for grids an
external directory already supplies under the same name, so a process handed the
full set writes nothing at start-up. That check costs one `stat` per embedded grid
at process start.

| Variable | Effect |
|---|---|
| `FLOW_PROJ_GRID_DIR` | Grid directories searched before the embedded set, as a path list (`:` on Unix, `;` on Windows) |
| `FLOW_PROJ_GRID_CACHE_DIR` | Where embedded grids are unpacked when an earlier directory does not already supply them. Defaults to the user cache directory, then the temporary directory. Needs to be writable |

Two things to know before wiring anything to this:

- **The file name is the interface.** PROJ resolves a grid by the exact name
  `proj.db` records, so a supplied file cannot be renamed. PROJ also accepts the
  publisher's original name and format (`.gsb`, `.gtx`, `.isg`) where EPSG
  registers it that way.
- **A grid `proj.db` does not reference is inert.** It will sit on the search path
  and never be used, with no error to explain why. Validate with
  `proj_grid_get_info_from_database`, which reports both whether the name is known
  and whether it is now available.

## Pairs no grid can fix

Some CRS pairs cannot be transformed accurately however complete the grid
coverage is, because the EPSG registry publishes no accurate operation between
their datums — only a ballpark one, which is refused. `NAD83(CSRS)` to `WGS 84`
is the common example, so `EPSG:6649` (NAD83(CSRS) + CGVD2013 height) to
`EPSG:4979` fails even with the whole catalogue present, despite
`ca_nrc_CGG2013n83.tif` being named in the operation PROJ found.

The error distinguishes the two cases, because they call for different responses:
a missing grid can be supplied, whereas a ballpark-only datum pair cannot. If
such a pair has to be supported, the decision to make is whether an approximate
result is acceptable for it — not which grid to fetch.

## Changing the embedded set

Edit the `GRIDS` list in `runtime/geometry/grids/update.sh` and run it: it
downloads each grid, verifies it against the SHA-256 the CDN publishes, and
regenerates `MANIFEST.tsv` and `NOTICE.md`. Then mirror the list in
`EMBEDDED_GRIDS` in `runtime/geometry/src/ops/reproject/grids.rs` — a test fails
if the two drift apart, and another fails if a grid is not one `proj.db` knows
about, which is the usual sign that a name is newer than the pinned PROJ.

Every grid added this way is carried in every binary, which is why the set is one
model per vertical datum. Broad coverage belongs behind `FLOW_PROJ_GRID_DIR`.

## Moving to a newer PROJ-data release

PROJ-data releases quarterly, adding a handful of files; names are immutable, so
an existing grid never changes underneath a release. A grid is only usable once
the *PROJ* release in `proj-sys` references it in `proj.db`, so the natural time
to bump is alongside a PROJ upgrade. Bump `PROJ_DATA_VERSION` in
`runtime/geometry/grids/update.sh` and run it; `cargo make
check-proj-grids-version` checks that the script and the manifest agree on the
release, and runs in CI.

Grids supplied through `FLOW_PROJ_GRID_DIR` are searched first, so they are what
decides which model PROJ picks. Keeping them on the same release as the embedded
set is worth the trouble: from different releases PROJ can select a different
geoid model in one environment than in another, and the same workflow produces
heights differing by centimetres with nothing to point at.

## Attribution

Grids carry licences of their own. Across the embedded set: CC-BY-4.0, CC0-1.0,
BSD-2-Clause, CC-BY-SA-4.0, the Etalab Open Licence, and one public-domain file.
`update.sh` copies the licence PROJ-data records for each file into `NOTICE.md`,
and fails rather than carrying a file it cannot find a licence for. The Japanese
grids additionally carry the Geospatial Information Authority of Japan's
permission under the Survey Act, reproduced in the notice.

Attribution is not only a redistribution obligation here. The embedded grids are
redistributed, since they are compiled into a binary that ships in a public
image. But the Etalab Open Licence (`fr_ign_RAF20.tif`) conditions acknowledgement
on reuse generally, so reading that grid on a server engages the requirement even
though nothing is distributed. Credit the publishers where the product credits its
other sources, rather than reasoning per channel.
