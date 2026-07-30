# Action Review Findings

Phase 3 quality review of the 73 base actions against [action-standard.md](action-standard.md).

**How to use:**

- Fill each action with either `ActionName — OK` or the checklist format from §7 of the standard
- Phase 4 improvement PRs should reference this file and delete completed sections as fixes land
- File is deleted when all sections are cleared

**Global fix (applies to all base actions):**

- `ports` — all ports currently named `default` must be assessed and renamed (§4.2); not called out per-action below. Check the Rust implementation to confirm semantics before renaming: if the port carries the primary feature stream rename to `features`; if it is a catch-all for unmatched features consider `unfiltered` instead. Make the change in Rust, then regenerate the schema (`cargo make schema-base`). Do not treat this as a mechanical find-replace.

---

## Deferred: Extended action documentation (not yet started)

While applying the standard, descriptions and parameter descriptions are being made concise per §2 and §3.3 (1–2 sentences, no reference dumps). This is correct, but some actions carried genuinely useful **reference-level** detail in their descriptions that concise text cannot hold — e.g. the Shapefile/CSV `encoding` params previously enumerated ~20 supported encodings with examples and priority order. That depth is trimmed during the audit and currently survives only in git history and source.

There is no home for this today:

- The schema `description` is the only user-facing text, and the UI renders it as **plain text** (no markdown) — long structured content renders poorly there anyway.
- The mdbook `docs/mdbook/src/action.md` is **generated** from the schema (`cargo run -- doc-action`), so it cannot hold anything the schema does not.
- Hand-written guides in `engine/docs/` (e.g. `czml-timeseries.md`) can hold arbitrary depth but are **orphaned** — not in mdbook `SUMMARY.md`, not linked from any action, not surfaced in the app.

**Task (needs planning before implementation):**

1. Decide the home + format for per-action extended docs (candidate: `engine/docs/actions/<action>.md`; wire into mdbook `SUMMARY.md` and/or extend `doc-action` to emit a "See also" link; consider a UI affordance linking action → doc).
2. Fold the existing orphan (`czml-timeseries.md`) into that convention.
3. Recover the reference detail trimmed during the audit (pull from git history of the touched factory files) and migrate it into the new docs.

Until this is planned, concise wins (Option A) — do not re-inflate descriptions to preserve reference material.

---

## Input — deferred items only (batch resolved in PR)

The Input batch (10 actions) was resolved per the standard. Deferred items remain, split out by decision:

```
GeoPackage Reader
  params:  includeMetadata, attributeFilter, batchSize, spatialFilter — these four
             params are accepted but never read (stored with `_` prefixes, no effect).
             They should be REMOVED (drops 10→6 params, clears the >8 flag). Removal is
             deferred to the `feat/engine-geopackage-reader-new-geometry` branch, which
             is actively editing the same file — folding the removal there avoids a
             conflicting structural change. The Input PR applied only the safe findings
             (schema description, titles/descriptions + enum docs for the working params,
             tags). Once removed, re-check ordering (natural order becomes dataset →
             inline → readMode → layerName → tileFormat → force2D).
  desc/enum: tile reading is currently stubbed out — the dispatch in geopackage.rs
             (`match params.read_mode`) routes `Tiles` and `All` to `read_features`
             with a `// Temporarily disabled tile processing` comment; `read_tiles` /
             `read_layer_tiles` exist but are never called, so `tileFormat` has no
             runtime effect. As a result these texts describe non-functional behavior:
             the action description ("supporting vector features, tiles, and metadata"),
             the `Tiles` variant ("Reads raster tiles."), the `All` variant ("Reads both
             vector features and raster tiles."), and the `tileFormat` param ("Image
             format to decode when reading raster tiles."). Deferred to the same
             `feat/engine-geopackage-reader-new-geometry` branch: either re-enable tile
             reading (making the text accurate) or, if tiles stay out, correct these
             descriptions. `features`, `metadataOnly`, `layerName`, and `force2D` are
             accurate.
```

---

## Feature · File · Transform — deferred items only (batch resolved in PR)

The Feature (1) · File (2) · Transform (4) batch was resolved per the standard. One item
found while auditing it is deferred:

```
XML Fragmenter
  i18n:    the parameter schema is a root-level `oneOf` (source variants), and the i18n
             overlay only reaches root `properties`, `definitions[*].properties`, and
             `definitions[*].oneOf` enum variants (`cli/src/utils.rs::apply_parameter_i18n`).
             So this action's per-parameter titles/descriptions, and the variant labels
             "XML File"/"XML Text", can never be translated — they stay English in all
             languages. Fixing this needs either i18n support for root-level oneOf
             variants or a flat param struct with a plain `source` enum. Not urgent: the
             root title/description do translate, and the English text is accurate.
```

---

## Geometry A — deferred items only (batch resolved in PR)

The Geometry A batch (12 actions) was resolved per the standard. Items found while
auditing it that belong elsewhere are deferred:

```
Bufferer
  params:  BufferType has a single `area2d` variant. The reference implementation
             this action was ported from offers both a 2D-area and a solid buffer
             type, and the PLATEAU 品質検査02 建築物 workspace our surface_validator
             graph is based on carries both branches, so a second variant is
             genuinely missing rather than hypothetical. Adding it needs a
             solid-buffering algorithm and an edge-resolution control that
             reearth-flow-geometry does not have, so the oneOf is kept with a TODO
             in bufferer.rs (standard §3.4, "variants planned but not
             implemented"). Own PR when the algorithm lands.
           interpolationAngle is applied when buffering a point or a curve but
             not a polygon — buffer_polygon() takes only a distance. The
             description now says so. Honouring it for polygons is an algorithm
             change, not a metadata one.
  impl:    only points, curves and single polygons are buffered. Every other
             type — multi-polygons above all, but also multi-points,
             multi-curves, solids, triangles and collections — is emitted on
             `features` unbuffered (the 3D arm projects it to 2D first). This
             deviates from every standard implementation: JTS defines `buffer()`
             on the base Geometry type and "the buffer operation always returns a
             polygonal result", so no type is un-bufferable. The projection
             itself is correct and should stay — PostGIS: "This function ignores
             the Z dimension. It always gives a 2D result even when used on a 3D
             geometry."
             CONSEQUENCE: a distance tolerance is silently not applied to those
             features. The PLATEAU surface_validator graph buffers by 0.005 as a
             near-touching tolerance, so any feature reaching that path is
             checked without it.
             Fixing it means buffering the full type space (union of the members'
             buffers) and would move quality-check results — passing the geometry
             through unchanged instead already fails 4 plateau6 02-bldg tests, so
             the truth data needs review by someone who can adjudicate PLATEAU
             conformance. Own PR: "Bufferer: buffer all geometry types per
             OGC/JTS semantics".

Image Rasterizer
  ports:   the `features` output carries two unrelated things: the features that
             arrived on `texture-coordinates`, re-emitted unchanged, or — when
             none did — a single synthetic feature holding a `png_image` path
             attribute. A rasterizer should emit the raster on a port of its own,
             separate from any pass-through stream. Nothing in-repo consumes our
             `features` output.
             Splitting it is a design change, and it is entangled with the
             save-path issue below, so both belong in one follow-up.
  impl:    save_image_with_path_option() falls back to std::env::var("HOME")
             instead of executor_cache_subdir.

  ui:      the OnOverlap variants render as "option 1/2/…" in the UI even now
             that every variant carries a `/// # Title`. The UI's
             consolidateOneOfToEnum (patchSchemaTypes.ts) bails out when any
             oneOf variant is an object type (`max`, `min` carry an expression),
             handing the schema to RJSF, which ignores the titles. Fixing it is
             a UI-side change: handle object-type oneOf variants using their
             `title` as the selector label.

CityGML Attribute Inserter (PLATEAU, outside the base set)
  ports:   input port `textureBounds` is camelCase (§4.1) and should become
             `texture-bounds`. Image Rasterizer's matching output port was
             renamed in this batch; the edge now joins two differently-named
             ports until the PLATEAU action is reviewed.
```

---

## Geometry B (11)

<!-- Session 9 — GeometryReplacer through VerticalReprojector -->

```
Geometry Replacer
  desc:    title-case — "Replace Feature Geometry from Attribute"; suggest "Replaces a
             feature's geometry with the compressed geometry data stored in a named attribute."
  ports:   inputPorts `default` — global note; outputPorts `default` — global note; no
             `rejected` — evaluate for missing or malformed attribute value (§4.3)

Geometry Splitter
  desc:    title-case — "Split Multi-Geometries into Individual Features"; suggest "Splits
             multi-part geometries into individual single-geometry features."
  params:  schema-level description "Parameters for GeometrySplitter" is an internal name
             recycled as description — not descriptive (§3.3); suggest "Configure how
             multi-part geometries are split into individual features."
           splitLevel — missing title (§3.3); description duplicates the oneOf variant
             content; trim to one sentence
  ports:   inputPorts `default` — global note; outputPorts `default` — global note; no
             `rejected` — evaluate for features without multi-part geometry (§4.3)
  tags:    ["split", "geometry"] — `split` not in vocabulary; `geometry` duplicates category
             (§6); remove both → 0 tags acceptable

Geometry Validator
  desc:    title-case — "Validate Feature Geometry Quality"; suggest "Validates feature
             geometry for issues such as duplicate points, corrupt geometry, or
             self-intersection."
  params:  ValidationType oneOf — `duplicatePoints` and `duplicateConsecutivePoints` variants
             missing per-variant descriptions; `corruptGeometry` and `selfIntersection` have
             descriptions but no `title` (§3.4). Same UI rendering bug as ImageRasterizer
             OnOverlap: `ValidationType` mixes one string variant (`duplicatePoints`) with three
             object variants, causing `consolidateOneOfToEnum` in `patchSchemaTypes.ts` to bail
             and RJSF to label all variants "option N". Requires both the Rust-side `/// # Title`
             fix on all variants and the UI-side fix to `patchSchemaTypes.ts`.
  ports:   inputPorts `default` — global note; outputPorts `success` ✓, `failed` ✓,
             `rejected` ✓
  tags:    ["validate"] — not in vocabulary; `validation` is; correct to ["validation"]

Grid Divider
  desc:    title-case — "Divide Polygons into Regular Grid Cells"; suggest "Divides polygon
             geometries into a regular grid of equal-sized cells."
  params:  schema-level description missing (§3.3)
           ordering — required `unitSquareSize` comes after optionals `groupBy` and
             `keepSquareOnly`; correct order: unitSquareSize → keepSquareOnly → groupBy (§3.5)
  ports:   inputPorts `default` — global note; outputPorts `default` + `rejected` ✓
  tags:    ["2d"] — not in vocabulary; replace with ["spatial"]

Horizontal Reprojector
  desc:    title-case — "Reproject Geometry to Different Coordinate System"; suggest
             "Reprojects feature geometry from one horizontal coordinate system to another
             using EPSG codes."
  params:  sourceEpsgCode — description is 4 sentences; exceeds 2-sentence guideline (§3.3)
           ordering — sourceEpsgCode (optional) appears before targetEpsgCode (required);
             correct order: targetEpsgCode → sourceEpsgCode (§3.5)
  ports:   inputPorts `default` — global note; outputPorts `default` — global note; no
             `rejected` — evaluate for invalid EPSG codes or reprojection failure (§4.3)
  tags:    ["projection", "2d"] — neither in vocabulary; replace with ["coordinate-system"]

Polygon Normal Extractor
  desc:    imperative not verb-first — "Extract normal vectors and other properties for
             polygon features"; "other properties" is vague; suggest "Extracts the normal
             vector and geometric properties from polygon features and stores them as
             attributes."
  ports:   inputPorts `default` — global note; outputPorts `default` — global note; no
             `rejected` — evaluate for non-polygon features (§4.3)
  tags:    ["normal", "3d"] — `normal` not in vocabulary; remove → ["3d"]

Ray Intersector
  params:  schema-level description "RayIntersector Parameters" is an internal name, not a
             description (§3.3); suggest "Configure how rays and geometries are paired and
             how intersection results are output."
           closestIntersectionOnly, geomId, includeRayOrigin, outputGeometryType, pairId,
             ray, tolerance — all 7 top-level params missing title (§3.3)
           RayDefinition sub-properties dirX, dirY, dirZ, posX, posY, posZ — all missing
             title (§3.3)
           ordering — required pairId and ray come after all optional params; correct:
             pairId → ray → outputGeometryType → closestIntersectionOnly → includeRayOrigin
             → geomId → tolerance (§3.5)
  ports:   inputPorts `ray`, `geom` ✓; outputPorts `no_intersection` — snake_case violates
             §4.1; rename to `no-intersection`; `intersection` ✓, `rejected` ✓
  tags:    ["ray", "intersection", "3d"] — `ray` and `intersection` not in vocabulary;
             replace with ["3d", "spatial"]

Refiner
  desc:    title-case — "Refine Complex Geometries into Simple Geometries"; suggest "Refines
             complex geometry types into simpler primitives."
  ports:   inputPorts `default` — global note; outputPorts `remain` — suggest rename to
             `remaining` for natural English; `default` — global note

Three Dimension Forcer
  name:    → "Three Dimension Forcer" — "Dimension" should be plural or adjectival; suggest
             "Three Dimensions Forcer" or "Three-Dimensional Forcer"
  desc:    title-case — "Convert 2D Geometry to 3D by Adding Z-Coordinates"; suggest "Adds
             Z-coordinates to 2D geometries to produce 3D output."
  ports:   inputPorts `default`, outputPorts `default` — global note

Two Dimension Forcer
  name:    → "Two Dimension Forcer" — same English issue as ThreeDimensionForcer; suggest
             "Two Dimensions Forcer" or "Two-Dimensional Forcer"
  desc:    title-case — "Force 3D Geometry to 2D by Removing Z-Coordinates"; suggest
             "Removes Z-coordinates from 3D geometries to produce 2D output."
  ports:   inputPorts `default`, outputPorts `default` — global note
  tags:    ["2d"] — not in vocabulary; `geometry` duplicates category (§6); remove both
             → 0 tags acceptable

Vertical Reprojector
  desc:    title-case — "Reproject Vertical Coordinates Between Datums"; suggest "Reprojects
             the vertical coordinate of feature geometry between vertical datums."
  params:  VerticalReprojectorType plain enum — single value `jgd2011ToWgs84` only
             (incomplete design); no per-variant description (§3.4)
  ports:   inputPorts `default` — global note; outputPorts `default` — global note; no
             `rejected` — evaluate for geometry without Z or transformation failure (§4.3)
  tags:    ["projection", "3d"] — `projection` not in vocabulary; replace with
             ["coordinate-system", "3d"]
```
