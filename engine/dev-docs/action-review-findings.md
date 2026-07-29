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

## Geometry A (12)

<!-- Session 8 — AppearanceRemover through ImageRasterizer -->

```
Appearance Remover
  ports:   inputPorts `default` — global note; outputPorts `default` — global note

Area Calculator
  params:  areaType — missing title (§3.3); description references "PlaneArea"/"SlopedArea" in
             PascalCase but actual enum values are camelCase (misleading)
           multiplier — missing title (§3.3)
           outputAttribute — missing title (§3.3)
           AreaType plain enum — no per-variant descriptions (§3.4); convert to oneOf or expand
             property description to describe each variant
  ports:   inputPorts `default`, outputPorts `default` — global note; no `rejected` — evaluate
             whether non-polygon features need a rejected route (§4.3)
  tags:    ["area", "measurement"] — neither in vocabulary; remove (0 tags acceptable)

Bounds Extractor
  desc:    title-case — "Extract Bounding Box Coordinates from Feature Geometry"; suggest
             "Extracts the bounding box coordinates of a feature's geometry and stores them as
             named attributes."
  params:  schema-level description missing (§3.3)
           ordering — alphabetical (xmax, xmin, ymax, ymin, zmax, zmin); suggest grouping by
             axis: xmin, xmax, ymin, ymax, zmin, zmax (§3.5 readability)
  ports:   inputPorts `default` — global note; outputPorts `default` + `rejected` ✓
  tags:    [] — `geometry` duplicates category (§6); 0 tags acceptable

Bufferer
  desc:    title-case — "Create Buffer Around Features"; suggest "Creates a buffer polygon
             around each input geometry at a specified distance."
  params:  BufferType oneOf with a single `area2d` variant — incomplete design; other buffer
             types planned but unimplemented (same structural flag as XMLFragmenter)
  ports:   inputPorts `default` — global note; outputPorts `default` + `rejected` ✓
  tags:    ["2d"] — not in vocabulary; replace with ["spatial"] (`geometry` duplicates category (§6))

Clipper
  desc:    title-case — "Clip Features Using Boundary Shapes"; suggest "Clips candidate
             features to the boundary geometry, separating results into inside and outside
             portions."
  ports:   inputPorts `clipper`, `candidate` ✓; outputPorts `inside`, `outside`, `rejected` ✓
  tags:    ["2d"] — not in vocabulary; replace with ["spatial"]

Elevation Extractor
  desc:    title-case — "Extract Z-Coordinate Elevation to Attribute"; suggest "Extracts the
             Z-coordinate elevation from a feature's geometry and stores it in a named
             attribute."
  ports:   inputPorts `default` — global note; outputPorts `default` — global note; no
             `rejected` — evaluate whether features lacking 3D geometry need a rejected route
             (§4.3)

Extruder
  desc:    title-case — "Extrude 2D Polygons into 3D Solids"; suggest "Extrudes 2D polygon
             geometries vertically by a specified distance to produce 3D solid geometries."
  ports:   inputPorts `default` — global note; outputPorts `default` — global note; no
             `rejected` — evaluate for non-polygon inputs (§4.3)

Footprint Replacer
  desc:    parenthetical "(supports solids, surfaces, and CityGML)" leaks implementation
             details; compound "Projects... and computes" obscures user-visible result; suggest
             "Replaces a feature's 3D geometry with its 2D footprint projected onto the XY
             plane."
  ports:   inputPorts `default` — global note; outputPorts `footprint` ✓, `rejected` ✓

Geometry Extractor
  desc:    title-case — "Extract Geometry Data to Attribute"; suggest "Serializes the feature's
             geometry to a compressed JSON representation and stores it in a named attribute."
  ports:   inputPorts `default` — global note; outputPorts `default` — global note; no
             `rejected` — evaluate for features with no geometry (§4.3)
  tags:    [] — `geometry` duplicates category (§6); 0 tags acceptable

Geometry Part Extractor
  desc:    imperative not verb-first — "Extract geometry parts (surfaces) from 3D geometries as
             separate features"; suggest "Extracts geometry parts from 3D geometries, emitting
             each part as a separate feature."
  params:  GeometryPartType oneOf with a single `surface` variant — incomplete design; evaluate
             what other part types should be added (Phase 4)
  ports:   inputPorts `default` — global note; outputPorts `extracted`, `remaining`, `untouched`
             — semantics of `remaining` vs `untouched` need clarification in Phase 4 (both
             receive non-extracted features — are they distinct conditions?)
  tags:    ["geometry", "decompose"] — `decompose` not in vocabulary; `geometry` duplicates
             category (§6); replace with ["3d"]

Geometry Remover
  ports:   inputPorts `default`, outputPorts `default` — global note

Image Rasterizer
  desc:    imperative not verb-first — "Convert vector geometries to raster image format";
             suggest "Converts vector geometries to a raster image using configurable overlap
             resolution."
  params:  imageWidth — missing title (§3.3); description "The width of image" incomplete —
             suggest "Width of the output image in pixels."
           OnOverlap — `takeLast`, `takeFirst`, `max`, `min` variants missing per-variant
             descriptions; only `sum` has one (§3.4). UI renders all variants as "option 1/2/3/4"
             due to two compounding issues: (1) no `/// # Title` on any variant, so `schemars`
             groups `takeLast`/`takeFirst` into a single two-value enum entry — fix by adding
             `/// # Title\n/// description` to every variant; (2) more fundamental — the UI's
             `consolidateOneOfToEnum` in `patchSchemaTypes.ts` bails out entirely when any `oneOf`
             variant is an object type (`max`, `min`), handing the schema to RJSF which labels
             variants "option N" regardless of titles. Fix (2) requires a UI-side change to handle
             object-type variants in `oneOf` using their `title` fields as selector labels.
  ports:   inputPorts `textureCoordinates` — camelCase violates §4.1; rename to
             `texture-coordinates`; `default` — global note
           outputPorts `textureBounds` — camelCase violates §4.1; rename to `texture-bounds`;
             `default` — global note; `textured` ✓
  tags:    ["raster", "image", "texture"] — `image` and `texture` not in vocabulary; replace
             with ["raster"]
```

---

## Geometry B — deferred items only (batch resolved in PR)

The Geometry B (11) batch was resolved per the standard. Items deferred out of it:

```
Horizontal Reprojector / Vertical Reprojector
  removal: both are superseded by `Coordinate Frame Reprojector`. Each already has a
             `#[cfg(feature = "new-geometry")] process` that errors with "use Coordinate
             Frame Reprojector instead", above a `TODO(new-geometry)` to delete the action.
             They were audited anyway rather than deferred, because the replacement is
             gated out of the default build: `geometry.rs` gates the module and
             `geometry/mapping.rs` gates the factory registration behind `new-geometry`,
             so `Coordinate Frame Reprojector` is absent from the generated `actions.json`
             and cannot be added to `server/api/internal/app/base_actions.go`. Until
             new-geometry becomes the default build these two ARE the shipped reprojection
             actions. Removal task, to land with that flip: delete both files, drop them
             from `base_actions.go`, add `Coordinate Frame Reprojector` there, and migrate
             the 29 (horizontal) + 20 (vertical) fixture files that reference them.

Coordinate Frame Reprojector
  params:  `epsgCode` is required only when `destinationFrame` is `crs`, which the schema
             cannot express as written — it is declared `Option<u16>` and validated in
             `build`. Folding it into a tagged enum on `destinationFrame`, the way
             `BasePoint` already is in the same file, would make it structurally required.
           `basePointSource` is titled "Base Point" and the nested `BasePoint::Value`
             field `basePoint` is also titled "Base Point", so the UI shows the same
             label twice.
  name:    Notion FLOW-DEV-182 says "Reprojector will be added instead", but the action is
             named "Coordinate Frame Reprojector". Settle which name ships.
  note:    not audited here — it is not in the base action set and does not appear in the
             generated schema. Audit it when it is ungated.

Geometry Validator
  params:  the param struct differs per build. `validationTypes` is
             `#[cfg(not(feature = "new-geometry"))]`, while `disabledOptionalChecks`,
             `planarityThreshold`, `duplicateTolerance` and `degenerateThresholds` are
             `allow(dead_code)` in the default build and only read under `new-geometry`.
             Because `schema-base` generates from the default build, today's `actions.json`
             advertises all five and four of them do nothing. Removing the four would break
             the migrated world, so they were documented rather than removed. Revisit when
             new-geometry becomes the default: `validationTypes` disappears then, and the
             per-variant titles added to `ValidationType` in this batch go with it.

Polygon Normal Extractor
  impl:    CityGML features with more than one polygon are rejected ("not supported yet")
             rather than measured, though the `FlowGeometry3D::MultiPolygon` branch already
             handles the multi-polygon case with `_{index}` attribute suffixes. Marked with
             a TODO in the code.
  scope:   Notion FLOW-DEV-182 has this "On hold" because "PlanarityFilter implicitly does
             the same thing". Only partly true: `PlanarityFilter` writes
             `surfaceNormalX/Y/Z` + `pointOnSurfaceX/Y/Z`, and has no equivalent for
             `slope`, `azimuth` (conventioned 0 deg = South to match SunPositionCalculator)
             or `signedArea2D`. It is also a filter, not an extractor. Merging the two is
             not the free win the note implies.

Refiner
  impl:    `refine_geometry` only descends into a top-level `GeometryCollection` and only
             collects `Multi*` members, so a plain MultiPolygon input refines to nothing and
             now passes through unchanged. CityGML refinement is unimplemented and also
             passes through (TODO in code). A full implementation — homogeneous aggregate to
             multi, single-member aggregate to its part, hole-less donut to polygon,
             single-segment path to segment, and merging consecutive line segments — is a
             separate task.

UI (separate PR, shared with Geometry A)
  `consolidateOneOfToEnum` in the UI's `patchSchemaTypes.ts` bails when a `oneOf` mixes
  string and object variants, so RJSF labels the variants "option 1/2/...". Affects
  `Geometry Validator`'s `ValidationType` and `Image Rasterizer`'s `OnOverlap`. The
  Rust-side `/// # Title` fixes are done in the respective engine PRs; the UI fix is not.
```
