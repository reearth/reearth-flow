# Geodetic grids

The geoid grids compiled into the engine binary. PROJ resolves a vertical datum
change by reading one of these. These grids are added to give the software a
minimal capability of vertical datum shift regardless of where it runs.

External grid models can be supplied at runtime through `FLOW_PROJ_GRID_DIR`
per environment. See [Supplying grids at runtime](#supplying-grids-at-runtime).

## Files

| File | Role |
|---|---|
| `MANIFEST.tsv` | The source of truth. Names the set, and records each file's size, SHA-256, publisher and licence. The header line pins the PROJ-data release the licence table was read from. |
| `*.tif` | The grids themselves, exactly as published by the PROJ-data CDN. |
| `NOTICE.md` | Attribution for every grid, rendered from `MANIFEST.tsv`. Auto-generated. |
| `update.sh` | Downloads and verifies the grids named in `MANIFEST.tsv`, then rewrites it with the metadata read back. Needs `curl`, `python3` and network access. |
| `render_notice.py` | Renders `NOTICE.md` from `MANIFEST.tsv`. |

Nothing else lists the grids. The crate's `build.rs` reads `MANIFEST.tsv` to
decide what to `include_bytes!`, and the container images copy `*.tif` wholesale.

## Adding a grid

1. Find its file name in the [PROJ-data CDN catalogue](https://cdn.proj.org/files.geojson).
2. Append a line to `MANIFEST.tsv` holding just that name. The other columns are
   filled in for you.
3. Run `./update.sh`. It downloads the file, verifies it against the SHA-256 the
   CDN publishes, looks up its licence, and rewrites the manifest.
4. Run `cargo make proj-grids-notice` to bring `NOTICE.md` in line.
5. Run `cargo test -p reearth-flow-geometry --lib ops::reproject::grids`. A grid is only 
    usable if PROJ's database names it in some operation, and a grid too new for the vendored
    PROJ is inert. This catches it.

## Removing a grid

Delete the `.tif` and the corresponding row in `MANIFEST.tsv`.

## Moving to a newer PROJ-data release

Edit the version in the `MANIFEST.tsv` header line and re-run `./update.sh`, then
`cargo make proj-grids-notice`. The version selects the licence table only; the
grids themselves are pinned by SHA-256, not by release, because the CDN is a
rolling mirror.

## Supplying grids at runtime

- `FLOW_PROJ_GRID_DIR` names directories of `.tif` files, in the platform's
  path-list syntax, searched ahead of the embedded set. Use it to add a grid the
  set lacks, or to override one. A directory already holding the whole embedded
  set also suppresses unpacking.
- `FLOW_PROJ_GRID_CACHE_DIR` is where the embedded grids are unpacked when
  `FLOW_PROJ_GRID_DIR` does not already supply them. It defaults to the user
  cache directory, falling back to the temporary directory. Set it when neither
  is writable.

The embedded grids are written out once per process, on the first reprojection,
and not at all if a process never reprojects. Files already present at the right
size are left alone.

Grids installed the way PROJ installs them, in its own user-writable directory
or under `PROJ_DATA`, are still found, after the two above.

A transformation whose grid is missing fails rather than quietly returning an
uncorrected height, because ballpark operations are refused. The feature is sent
to the processor's `rejected` port, so a workflow leaving that port unwired drops
it and still reports success.

## Attribution

Every grid is redistributed under the licence its publisher applied to it. The
full table is in [NOTICE.md](NOTICE.md), which must be shipped with any binary
that embeds them.
