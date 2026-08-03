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
Geometry Part Extractor
  params:  GeometryPartType keeps its single `surface` variant, with a TODO, on the
             same reasoning as Bufferer below (§3.4, "variants planned but not yet
             implemented"). `edge` and `vertex` variants — emitting each edge or
             vertex as its own feature — fit the three ports this action already
             declares, and no other action covers them: Boundary Extractor returns
             the boundary as one geometry on the same feature, and Coordinate
             Extractor writes vertices into attributes. Removing the parameter was
             considered and rejected: it would have foreclosed that space and
             pushed us toward a separate action per part type.
  ports:    `extracted` / `remaining` / `untouched` are distinct and correct, and
             match the reference implementation. `remaining` is the original
             feature with its extracted parts removed; `untouched` is a feature the
             action did not modify. No change needed.

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

Design pass (2026-08-03) — items raised by a usability review of the whole batch, deferred
here because each is larger than a metadata fix:

Refiner vs Geometry Splitter
  design:  their descriptions are not distinguishable by a user — both take a container and
             emit its members on `features` — and Refiner has no workflow using it. It should
             fold into Geometry Splitter. NOT done here: legacy Splitter handles MultiPolygon
             and MultiLineString but falls through GeometryCollection to pass-through, which is
             the only case Refiner actually implements, so folding means ADDING collection
             support to the legacy path — new capability the new-geometry world discards
             anyway, since its generic `Split` op already covers collections, meshes and point
             clouds. Do this at the new-geometry flip: delete refiner.rs, drop it from
             geometry/mapping.rs and from base_actions.go.

Polygon Normal Extractor
  design:  the only Extractor with no output-naming parameter. It writes six fixed attribute
             names (normalX/Y/Z, signedArea2D, slope, azimuth), so a user cannot avoid
             collisions or ask for just one property, and the azimuth convention (0 deg =
             South) is invisible from the schema. For a multi-polygon it also smears results
             across numbered suffixes (normalX_0, normalX_1, …) rather than emitting one
             feature per polygon, which is what the data model wants. Entangled with the
             action's unresolved On-hold status, so it belongs in one follow-up.

Geometry Validator
  design:  the two parameter models are opposites. `validationTypes` is opt-IN (list the checks
             you want, tolerance inline); the new-geometry set is opt-OUT (all checks run,
             `disabledOptionalChecks` removes some, tolerances are separate params). The
             opt-out model is the right one — it matches the OGC model, where validity is a
             property of the geometry and a separate detail call reports which rule failed, not
             a menu chosen up front. Converging costs something real though: 26 Validator nodes
             in this repo each run exactly ONE check, so `failed` currently means "failed that
             check". Under the new model `failed` means "failed anything" and each of those
             nodes needs a downstream filter on `validationResult.checks`, changing which port
             a feature leaves by. Cost this before the flip, and check whether FlowExpr can
             even address a nested attribute like `validationResult.checks.selfIntersection`.

Geometry Validator
  ui:      hits the same `consolidateOneOfToEnum` bug written up under Image Rasterizer in
             the Geometry A section above — `ValidationType` mixes one unit variant
             (`duplicatePoints`) with three that carry a tolerance, so RJSF labels them
             all "option 1/2/…" despite every variant now carrying a `/// # Title`. One
             UI-side fix covers both actions.
```
