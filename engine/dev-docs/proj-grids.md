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

## The rest of the catalogue, from the deployment mirror

PROJ-data publishes far more than the binary can carry: 161 geoid grids
(554 MiB), 150 horizontal grids (196 MiB), 42 deformation models — 865 MiB in
all, counting historical realizations (GEOID12B, AUSGeoid09, RAF18, `HT2_1997`),
tidal and chart datums, overseas territories, and alternative realizations on
other reference frames. Deployments carry **all** of it, out of a bucket rather
than the image.

`tools/proj-grids/mirror.py` builds that set into a directory, verifying every
file against the catalogue's SHA-256 and writing `MANIFEST.tsv`, `NOTICE.md` and
the publishers' README files beside the grids. The directory is synced to a
bucket, which the workers mount **read-only** through a Cloud Storage volume, and
`FLOW_PROJ_GRID_DIR` points at the mount.

Nothing ever writes through that mount, and nothing needs to: the mount supplies
grids to read, and the only path that writes — unpacking the embedded grids — goes
to a container-local cache directory. When the mirror already supplies all 27
embedded grids under the same names, even that is skipped, so a worker writes
nothing at startup.

Two things to watch on a lazy mount: the first read of a large grid crosses the
network (enable the mount's file cache if that shows up in run times), and the
skip check costs one `stat` per embedded grid at process start.

## Supplying a grid that is not in PROJ-data

Korea, China, Italy, Brazil and much of Asia and Africa publish national geoid
models that PROJ-data cannot redistribute, so the mirror does not carry them.
Those have to come from whoever holds the licence, through the same variable,
which takes a list so a user-supplied directory composes with the mirror:

```bash
export FLOW_PROJ_GRID_DIR=/run/grids/workspace:/mnt/proj-grids
```

A grid uploaded through the product is written to storage by the API and staged
into the run's own directory, the way `worker/src/asset.rs` already stages
assets. That keeps one workspace's grids off another run's search path, and needs
no write access to the shared mount.

| Variable | Effect |
|---|---|
| `FLOW_PROJ_GRID_DIR` | Grid directories searched before the embedded set, as a path list (`:` on Unix, `;` on Windows). In a deployment: the run's staged grids, then the mounted mirror |
| `FLOW_PROJ_GRID_CACHE_DIR` | Where embedded grids are unpacked when an earlier directory does not already supply them. Defaults to the user cache directory, then the temporary directory. Needs to be writable |

Two things to know before wiring an upload path to this:

- **The file name is the interface.** PROJ resolves a grid by the exact name
  `proj.db` records, so an uploaded file cannot be renamed. PROJ also accepts the
  publisher's original name and format (`.gsb`, `.gtx`, `.isg`) where EPSG
  registers it that way.
- **A grid `proj.db` does not reference is inert.** It will sit on the search path
  and never be used, with no error to explain why. Validate at upload time with
  `proj_grid_get_info_from_database`, which reports both whether the name is known
  and whether it is now available.

## Pairs no grid can fix

Some CRS pairs cannot be transformed accurately however complete the grid
coverage is, because the EPSG registry publishes no accurate operation between
their datums — only a ballpark one, which is refused. `NAD83(CSRS)` to `WGS 84`
is the common example, so `EPSG:6649` (NAD83(CSRS) + CGVD2013 height) to
`EPSG:4979` fails even with the whole catalogue present, despite
`ca_nrc_CGG2013n83.tif` being there and named in the operation PROJ found.

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

## Moving to a newer PROJ-data release

PROJ-data releases quarterly, adding a handful of files; names are immutable, so
an existing grid never changes underneath a release. A grid is only usable once
the *PROJ* release in `proj-sys` references it in `proj.db`, so the natural time
to bump is alongside a PROJ upgrade.

1. Bump `PROJ_DATA_VERSION` in `runtime/geometry/grids/update.sh` and run it.
2. Rebuild the deployment mirror from the same version and sync it to the bucket
   (`tools/proj-grids/mirror.py`).

`cargo make check-proj-grids-version` checks step 1 for self-consistency and runs
in CI. It cannot check step 2: the mirror's version lives outside the repository.
Doing the two together matters because the embedded set governs development and
CI while the mirror governs production — from different releases, PROJ can select
a different geoid model in each, and the same workflow produces heights differing
by centimetres with nothing to point at.

## Attribution

Grids carry licences of their own: mostly CC-BY-4.0, some public domain, CC0,
OGL-Canada, Etalab, DL-DE-BY or CC-BY-SA-4.0. Both `update.sh` and `mirror.py`
copy the licence PROJ-data records for each file into a `NOTICE.md`, and fail
rather than carrying a file they cannot find a licence for. The Japanese grids
additionally carry the Geospatial Information Authority of Japan's permission
under the Survey Act, reproduced in the notice.

Attribution is not only a redistribution obligation here. The embedded grids are
redistributed, since they are compiled into a binary that ships in a public
image. But OGL-Canada-2.0 conditions acknowledgement on "copy, modify, publish,
translate, adapt, distribute **or otherwise use**", and the Etalab Open Licence
conditions it on reuse generally — so reading those grids on a server engages
the requirement even though nothing is distributed. Between them that is 92 of
the mirrored files. Credit the publishers where the product credits its other
sources, rather than reasoning per channel.
