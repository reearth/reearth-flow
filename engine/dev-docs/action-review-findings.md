# Action Review Findings

Quality review of the base actions against [action-standard.md](action-standard.md).

> **Resuming this work? Start at [HANDOVER](#handover--state-of-the-audit-as-of-2026-08-21) at
> the bottom of this file.** It carries the current palette state, the 22 actions still awaiting
> an audit with preliminary findings for each, and the cross-cutting issues that should shape
> whatever order they are tackled in. The sections above it are older per-batch residue.

**How to use:**

- Fill each action with either `ActionName — OK` or the checklist format from §8 of the standard
- Improvement PRs should reference this file and delete completed sections as fixes land
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

---

## Re-verification of already-audited base actions (in progress)

Prompted by PR #2365: `GeoPackage Reader` had been audited in #2280 (Input batch), yet
shipped a documented tile-reading feature that had been disabled since #1460 ten months
earlier, plus four parameters that were never read. #2280 is also the PR that added the
ACCURACY-BEFORE-STYLE clause to the standard — the rule was written in the commit that
failed to apply it.

This section records a systematic re-check of the **72** base actions audited by prior
batches (82 in `base_actions.go`, minus the 4 verified in #2365 and the 6 promoted by
#2356 that still await a first pass).

**Method.** Detectors were derived from the specific defects #2280 missed, and calibrated
against the pre-#2365 `geopackage.rs` — all 9 calibration checks reproduce the known
defects. Detector output is a set of *leads*; every entry below was confirmed by reading
the code path.

### Clean classes (no findings across the 72)

- **Dead parameters** — zero. Every schema parameter resolves to a Rust field that is read
  and applied. This is the class that produced GeoPackage's four; #2365 removed the last
  instances (`glTF.triangulate`, `OBJ.includeNormals`). Verified with `#[serde(rename)]`
  and `#[serde(flatten)]` resolution, so `geometryPartType`→`part_type`,
  `force2D`→`force_2d` and CSV Reader's flattened `offset`/`headerRows`/`geometry` were all
  checked rather than skipped.
- **Enum variants with identical match-arm bodies** — zero. This is the exact shape of the
  `Tiles`/`All` bug (separate arms, duplicated bodies, both calling `read_features`).
  GeoPackage was the only instance.
- **Unreachable code behind `#[allow(dead_code)]`** — one site remains
  (`attribute/null_attribute_mapper.rs:202`); not yet assessed.

### Confirmed findings

```
Horizontal Reprojector / Vertical Reprojector          <-- most severe
  desc:    Both actions cannot run in the shipped build. Their
             `#[cfg(feature = "new-geometry")] process` returns Err unconditionally
             ("use Coordinate Frame Reprojector instead" —
             `horizontal_reprojector.rs:419-430`, `vertical_reprojector.rs:116-127`),
             and new-geometry is the DEFAULT feature of the cli and worker since #2343.
             Both are still `true` in `base_actions.go:52,61`, so two user-visible base
             actions fail on every feature — while carrying fully-audited, polished
             parameter descriptions for behaviour that never executes. This is the
             GeoPackage pattern at maximum severity.
  scope:   This was known and deliberately deferred during Geometry B, on the grounds
             that the replacement was gated out of the shipped build and so could not be
             named in `base_actions.go`. **That precondition has expired** — `Coordinate
             Frame Reprojector` is now present in `actions.json` (verified). Nothing
             re-triggered the deferred work when the blocker cleared, which is the
             process failure worth fixing as much as the code.
  fix:     Delete both actions, swap `Coordinate Frame Reprojector` into
             `base_actions.go`, migrate fixtures. Interim mitigation if the deletion is
             not immediate: remove both from `base_actions.go` so the palette stops
             offering actions that always fail.

Image Rasterizer
  params:  `saveTo` — description says "When omitted, the image is written to the cache
             directory". It is written to `$HOME/.cache/reearth-flow-generated-images`
             via a raw `std::env::var("HOME")` with a `"."` fallback
             (`geometry/image_rasterizer.rs:250-255`), bypassing `executor_cache_subdir`
             which every other accumulating action uses (cf. `dissolver.rs`,
             `area_on_area_overlayer.rs`). In a worker container with no `HOME` the image
             lands in the process's current working directory. The description makes an
             arbitrary location sound managed. Fix the code, not the text — this is the
             long-deferred `save_image_with_path_option` item, now user-visible.

Ray Intersector
  params:  `geomId` — description promises "every intersection carries a `geom_id`
             attribute naming the geometry it hit". The attribute written is `geomId`
             (`geometry/ray_intersector.rs:504`), camelCase, consistent with its sibling
             `distanceToIntersection` at :496. The code is right and the description names
             an attribute that does not exist; anyone reading `geom_id` downstream gets
             nothing. Fix the description.
```

### Verified accurate (spot-checked, no action needed)

- `Area Calculator.areaType` — "Has no effect on a geometry with no elevation" is true;
  the parameter is only consulted inside the 3D branch (`area_calculator.rs:148-161`).
- `Feature Counter.countStart` — "Value assigned to the first feature" is true;
  `fetch_add` returns the pre-increment value seeded from `start`.
- `XML Validator.{attribute,inputType,validationType}` — all three applied
  (`xml/validator.rs:280,405,409`).
- `CityGML Reader.flatten` — applied at `file/reader/citygml.rs:58`.
- `CSV Reader.{offset,headerRows,geometry}` — applied via the flattened
  `csv::CsvReaderParam` passed whole to the shared reader (`file/csv.rs:157`).
- `Attribute Manager` / `Feature Joiner` shared match arms — `Method::Create | Method::Convert`
  shares only the schema-inference arm (`attribute/manager.rs:105`) and diverges at
  `:320`/`:324`; a detector false positive, not a defect.

### Control sample — 10 actions no detector flagged, read by hand

`Feature Merger` · `Feature Sorter` · `Bulk Attribute Renamer` · `Attribute Mapper` ·
`Feature Type Filter` · `List Exploder` · `File Property Extractor` · `Bounds Extractor` ·
`Two Dimension Forcer` · `GeoJSON Writer`

**Zero findings.** Every declared port is emitted on a real path, and every parameter is
applied. Two cases that looked like the GeoPackage forwarding signature resolved clean:
`Bulk Attribute Renamer.textToFind` has a single read but is compiled into the `regex`
field the processor uses (`bulk_renamer.rs:68-84`), and `selectedAttributes` is read in the
rename path at `:186`.

This corroborates the detectors' clean classes: the structural defects really are absent
from the audited set, rather than merely invisible to grep.

### Verdict

**Prior batches do not need wholesale redoing.** The structural failure modes that produced
GeoPackage's four dead parameters and its unreachable tile chain are absent across all 72
audited actions, and a hand-read control sample agrees.

**But the accuracy class is not clean:** 4 of the 72 carry documentation that misstates
behaviour, and one case is severe enough to be a release blocker (both reprojectors always
fail). The pattern is not sloppiness spread evenly — it is concentrated in actions whose
behaviour changed *after* they were audited.

That points at the real systemic gap, which is a process one rather than a review-quality
one: **a deferred finding whose precondition later expires has nothing watching it.** The
reprojectors were correctly identified during Geometry B and correctly deferred, because
`Coordinate Frame Reprojector` was gated out of the shipped build at the time. When that
gate lifted, nothing re-opened the item. Fixing individual descriptions does not address
this; the deferred-items sections in this file need an owner and a re-check trigger.

### Outside the 72: vendor-name leaks in two Batch C actions

Found while checking the §2 rule against the code. Both actions are currently non-functional
in the shipped build, so nothing is reaching users today — but these must be fixed before
either is repaired or promoted.

```
Neighbor Finder
  desc:    `mergeStrategy`'s `repeatBase` variant has a `///` doc comment naming the
             commercial product the action was ported from (`neighbor_finder.rs:143`).
             Doc comments compile into the schema, so the name is present in
             `actions.json` today — this is the only action in the whole schema that
             leaks it. Rewrite to describe the behaviour directly (§2).

Center Point Replacer
  params:  Writes an output attribute literally named `fme_rejection_code` onto every
             rejected feature (`center_point_replacer.rs:152`, asserted in tests at
             :785, :823, :842), plus a `//` comment at :339. This is worse than a
             documentation leak: the vendor name ends up in the user's *data*, where it
             becomes a compatibility surface someone may write a downstream filter
             against. Rename to something ours — e.g. `rejectionCode`, matching the
             camelCase convention the codebase uses for written attributes
             (cf. `geomId`, `distanceToIntersection`) — and update the three tests.
```

### Still to verify

Behavioural claims not yet traced to a code path (7): `Attribute Aggregator.calculationValue`
precedence · `Statistics Calculator.groupBy` single-group · `Image Rasterizer.onOverlap`
arrival-order default · `JSON Writer.converter` omitted-case · `Shapefile Reader.encoding`
case-insensitivity · `Directory Decompressor.findDeepestSingleFolder` · `Cesium 3D Tiles
Writer.targetTileSize` merge behaviour.

Stated-default mismatches to adjudicate (3, all likely wording rather than defect):
`Footprint Replacer.projectionPlane` · `Geometry Validator.degenerateThresholds` ·
`Horizontal Reprojector.sourceEpsgCode`.

Plus: the one remaining `#[allow(dead_code)]` site, and the §4.3 inverse (features consumed
and silently discarded with no port) which has no syntactic signature and needs reading the
accumulate/emit path of the ~20 actions declaring a non-`features` port.

---

## HANDOVER — state of the audit as of 2026-08-21

Work paused here ahead of FOSS4G. This section is the resumption point: what is done, what is
not, and every preliminary finding gathered but not acted on. Read this before restarting.

### Where the palette stands

`server/api/internal/app/base_actions.go` exposes **57** actions, down from 105. The gate is now
strict: an action is listed only if it **runs in the shipped build** (§7.1) **and** has passed an
engine-side review. Nothing below is a deletion — every hidden action still executes in a
workflow that names it, so no existing workflow broke.

| Bucket | Count | Trigger to re-expose |
|---|---|---|
| Exposed and audited | 57 | — |
| Does not run in the shipped build | 22 | Its new-geometry port landing (Notion FLOW-DEV-182) |
| **Pending audit** | **22** | An engine-side §8 pass — the list below |
| Flagged for removal | 2 | None; they owe an engine-side deletion |
| Retired on design grounds | 2 | A scope decision, see below |

**The most important consequence:** `Coordinate Frame Reprojector` is in the pending-audit
bucket, and the two legacy reprojectors are gone, so **the palette currently offers no
coordinate reprojection at all.** That makes it the strongest candidate to audit first,
regardless of category order.

### What "audited" now means — read §"How to use" and §8 first

The standard was materially rescoped on 2026-08-21 (see its Changelog). The verify-against-
implementation duty used to be worded as a precondition for *editing* a title or description,
which exempted anything that already read well. It now attaches to the action itself, and §8
gained an `impl:` line — the one checklist line that cannot be answered from `actions.json`.

This matters for planning: **a schema-level scan cannot triage this work.** In Batch 1 the one
action a mechanical scan marked `OK` (`Attribute Table Extractor`) turned out to have an
entirely undocumented configuration surface and a parameter name that actively misled. Any
estimate of "these just need superficial fixes" is unearned until the code is read.

---

### Pending audit — 22 actions, with preliminary findings

Findings below came from a schema scan plus partial code reading. **They are leads, not verdicts** —
none has had the full `impl:` trace except where stated. Grouped as they were batched; the
grouping is a suggestion, not a constraint.

#### Group A — Geometry metadata (5)

```
Coordinate Frame Reprojector          <-- AUDIT FIRST (see above)
  scan:    Clean on every mechanical check. Replaces Horizontal + Vertical Reprojector.
  unread:  destinationFrame / epsgCode / basePointSource never traced; `base-point`
             second input port never verified as emitted-from/consumed.

Hole Counter
  desc:    "Count Polygon Holes to Attribute" — Title Case imperative, no period (§2).
  note:    Carries a doc comment naming a commercial product, but on a TEST function inside
             `mod tests`, so it does NOT reach actions.json — verified. Not a §2 violation.

Hole Extractor
  desc:    "Extract Polygon Holes as Separate Features" — Title Case imperative, no period.
  ports:   `outershell` should be `outer-shell` (§4.1 kebab-case for multi-word). Renaming a
             port is a data-loss change (§4.3) — size the blast radius first.
  params:  Zero parameters. Confirm that is genuine minimalism, not a missing control.

Geometry Coercer
  desc:    No terminating period (§2).
  unread:  `targetType` enum variants never traced to their branches.

Offsetter
  scan:    Clean on every mechanical check — titles, descriptions, defaults, tags all present.
             Best-documented of the group going in. Code never read.
```

#### Group B — List and feature utilities (5)

```
Feature Duplicate Filter
  desc:    "Filter Out Duplicate Features" — Title Case imperative, no period.
  params:  ZERO parameters, while its sibling Attribute Duplicate Filter takes `filterBy`.
             What is the dedup key? If it is whole-feature equality that must be documented;
             if it is implicit, it likely needs a parameter. UNRESOLVED — this is the open
             design question of the group.

List Concatenator
  desc:    No terminating period.
  params:  all 4 of list/attribute/separateCharacter/outputAttributeName lack a `title`, so
             they render as bare camelCase labels.
  note:    Relevant to the retired Attribute Bulk Array Joiner — this is the targeted,
             separator-aware version of a similar operation. Decide them together.

List Indexer
  desc:    No terminating period.
  params:  all 4 lack a `title`.

Feature CityGML 3 Reader
  desc:    No terminating period; param block has no root description (§3.3).
  cat:     `Feature`. See the open Feature-vs-Input question below.

Feature GeoJSON Writer
  scan:    Clean on every mechanical check. Code never read.
  cat:     `Feature` while every true sink uses `Output` — but it is a processor, not a sink.
             Same open question as Feature Writer.
```

#### Group C — CSG pair and Excel Writer (3)

```
CSG Builder
  desc:    4 sentences; §2 allows 1–2.
  ports:   `left` + `right` inputs are properly semantic (§4.2) — good.

CSG Evaluator
  ports:   **`nullport`** is not in the §4.2 vocabulary, is not a word, and is a label users
             see on the node. Needs a real name. Defined at csg_evaluator.rs:24.

Excel Writer
  cat:     `File`, but it is a genuine sink (type: sink, no output ports), so §5 wants
             `Output`. `File` is for decompression and path utilities.
  params:  `output` and `sheetName` both lack a `title`.
```

#### Group D — Overlay family (3)

```
Line On Line Overlayer                                  <-- worst metadata in the palette
  params:  `groupBy` and `tolerance` have NO description at all — the only non-PLATEAU action
             in the schema in that state. They render as bare camelCase keys with no label.
             `overlaidListsAttrName` lacks a title. Sibling Area On Area Overlayer has both
             parameters fully described; confirm the semantics match, then copy.

Area On Area Overlayer
  ports:   `area` was agreed to become `overlaps` (breaking).

Dissolver
  desc:    "Dissolve Features by Grouping Attributes" — Title Case imperative, no period.
  ports:   `area` was agreed to become `features` (breaking).
```

Review these three together — they share `groupBy` / `tolerance` / accumulation semantics, and
the two port renames were already agreed in principle but never sized against fixtures.

#### Group E — Root-level `oneOf` restructuring (4, plus 1 already-audited)

```
Feature Reader · Feature Writer · JSON Fragmenter · Geometry Filter
  params:  The whole parameter block is a root-level `oneOf`, which §3.4 prohibits because
             apply_parameter_i18n cannot reach the variants — its definitions traversal is
             scoped inside `definitions`, so a root `oneOf` is never visited. The block's own
             title/description DO translate via the "" key, so the action looks localised
             while the mode labels the user picks between stay English permanently.
             Geometry Filter's Japanese entry is in exactly this state today.
  fix:     Restructuring, not re-wording (§3.4 names the idiom: a #[serde(tag = "type")] enum
             as a property VALUE, not as the whole block).

Geometry Filter
  ports:   20 output ports. Wants per-variant port declaration, which would also retire the
             action-name-keyed special cases in builder_dag.rs:258-300 and
             schema_infer.rs::effective_output_ports that the engine authors themselves
             flagged as needing "an architectural revision of port handling".
  desc:    "Filter Features by Geometry Type" — Title Case imperative, no period.

XML Fragmenter  — ALREADY AUDITED, still non-compliant
  params:  Same root-level `oneOf`. It was audited and its oneOf was deliberately extended
             before §3.4 existed. Owed a re-check per the Changelog rule, not a new finding.
```

This group is schema-breaking by nature and wants its own PR.

#### Group F — Known-heavy (2)

```
HTTP Caller
  params:  13 parameters including `timeouts`, `retry`, `rateLimit`, `httpOptions` and
             `observability`. §3.5 "No implementation leakage" names `timeout` and
             `retryCount` as its canonical examples of what must NOT be exposed, and §3.5's
             volume guideline is 8. Applying the rule as written would delete roughly half
             this action's surface — that is a product decision, not an audit call.
  cat:     Was `Web` (off-taxonomy, would have landed in a phantom palette group);
             recategorised to `Feature` in #2373. Done.

Attribute Duplicate Filter
  desc:    "Remove Duplicate Features Based on Attribute Values" — Title Case, no period.
             Param block has no root description.
  bug:     Known key-collision defect, plus keep-first semantics to settle and a `duplicate`
             port to consider. Adding that port is a data-loss change (§4.3).
```

---

### Cross-cutting findings — read these before planning any batch

**1. The §4.3 missing-attribute defect is systemic, not isolated.** The standard's §4.3 was
corrected on 2026-08-20: a missing attribute is normally a no-op that must pass through, not a
failure. Confirmed instances so far:

- `Date Time Converter` — **fixed** in this PR.
- `Attribute File Path Info Extractor` — moot, retired.
- `JSON Fragmenter` — **not fixed.** Routes a missing attribute to `rejected`, and this is
  asserted as intended in a test named `test_missing_attribute_rejected`. It sits in Group E.

Three instances across three different batches means the remaining 22 should be assumed to
contain more. The reliable tell is a `rejected`/`failed` branch guarded by an attribute lookup
returning `None`. Note that fixing it is a routing change: those ports are unwired in most
workflows, so features currently vanishing there will start flowing onward.

**2. §6 tag debt in the already-audited set.** §6 was rewritten on 2026-08-21 (tags cut *across*
categories; zero tags is now explicitly valid). Consequences not yet applied:

- Genuine omissions, proven by a tagged sibling doing the same work: `Two Dimension Forcer`
  has no tags while `Three Dimension Forcer` has `3d`; `Feature Filter` has none while every
  other `Filter` action is tagged and `Input Router`/`Output Router` both carry `routing`.
- Category-restating tags, now findings under the new wording: `Directory Decompressor` and
  `File Property Extractor` both carry `file` while sitting in category `File`.
- Correctly zero, needing no change: `Noop Processor`, `Noop Sink`.
- Undecided, and deliberately not guessed: `Attribute Manager`, `Attribute Flattener`,
  `Bulk Attribute Renamer`, `Feature Joiner`, `Feature Merger`, `Feature Sorter`,
  `Geometry Extractor`, `Geometry Remover`, `Geometry Splitter`, `Bounds Extractor`. Each needs
  the code read to judge whether an orthogonal axis exists.

**3. Open question — `Feature` versus `Input` for mid-flow readers.** Unresolved, and it
implicates a merged change. `Feature CityGML Reader`, `Feature CityGML 2 Reader`,
`Feature CityGML 3 Reader` and `Feature Reader` are all **processors** (features in, features
out) that read a path taken from the incoming feature. They are not graph sources. §5 assigns
"CityGML reading" to `Feature`, yet the first two are categorised `Input`:

| Action | Category today | Set by |
|---|---|---|
| Feature CityGML Reader | `Input` | #2114, which predates the standard |
| Feature CityGML 2 Reader | `Input` | #2365 — changed to match the sibling, not to match §5 |
| Feature CityGML 3 Reader | `Feature` | original |
| Feature Reader | `Feature` | original |

The #2365 change was made for the wrong reason: matching a sibling rather than checking §5,
which points the other way. Both directions are defensible — `Feature` follows §5 as written and
reflects what these actions are; `Input` follows where a user looks for "the thing that reads
CityGML" and would mean §5's `Feature` row should drop "and CityGML reading". Needs a decision,
not a unilateral fix. Same class: `Feature Writer` / `Feature GeoJSON Writer` are processors
categorised `Feature` while every true sink uses `Output`.

**4. Off-taxonomy categories on hidden actions.** `OBJ Writer` has `['File','3D']` and
`Python Script Processor` has `['Script','Python']`. Neither is in the §5 taxonomy nor the UI's
category filter, so both would land in a phantom palette group if exposed. Fix before exposing,
not after.

**5. Vendor-name leak still shipping.** `neighbor_finder.rs:143` names a commercial product in a
`///` doc comment, which compiles into `actions.json` (§2). `center_point_replacer.rs:152`
emits an output attribute named `fme_rejection_code`, referenced by 3 tests — that one is a
schema-visible rename. Both actions are currently hidden (neither runs), so this is not live
user-facing text today, but it must be fixed before either is re-exposed.

**6. Proposed CI ratchets, neither implemented.** Both are cheap and deterministic:
- Fail when a `baseActions` key does not exist in `actions.json`, or names a `builtin: false`
  action. Nothing guards this today; the 105 names were verified by hand.
- Fail when a parameter in `actions.json` lacks a `description`. Would have caught all four
  dead GeoPackage Reader parameters in 2025. Start description-only — a `title` rule needs the
  Group B/C/D cleanup above to land first.

---

### Batch 1 outcome — the only batch completed under the revised standard

Audited and **kept exposed**:

- `Attribute Range Mapper` — well designed; changes were documentation only. Now documents the
  `to == from` exact-match branch, the string/bool coercion, and that `defaultValue` also
  applies when the attribute is absent or non-numeric. Gained the `mapping` tag its three
  siblings carry.
- `Date Time Converter` — well designed. Gained 4 parameter titles, 12 enum variant titles and a
  root description; enum values corrected from `unix_s`/`unix_ms` to `unixS`/`unixMs` (§3.4);
  §4.3 fixed so a feature lacking the attribute passes through on `features`. That fix also
  required correcting `infer_output_schema`, which promised the output attribute was `always`
  present on `features` — now `maybe`. Regression test
  `missing_attribute_passes_through_on_features` added; this behaviour had no coverage.
- `Attribute Table Extractor` — sound concept, and the only one of the five with real production
  use (PLATEAU6 01-bldg). `inline` is now typed, so `ExtractRule` finally generates into the
  schema with titles and `required`. `jsonPath`/`attribute` renamed to
  `sourcePath`/`destinationPath`: the old name promised JSONPath but the implementation is a
  space-separated key chain, so `$.a.b` failed silently. Renaming rather than implementing
  JSONPath was deliberate — the destination path needs *write* semantics, which JSONPath has
  none of; `$` appears as a literal key in real CityGML tables; multi-match semantics would
  have to be invented; and feature attributes are an `IndexMap`, so JSONPath would mean
  serialising every feature per rule. Space separation is correct here because the keys are XML
  QNames, which cannot contain whitespace but do contain colons. All 106 rules in the PLATEAU6
  config migrated.

**Retired, not audited:**

- `Attribute File Path Info Extractor` — an exact duplicate of `File Property Extractor`: same
  five output attributes (`fileType`, `fileSize`, `fileAtime`, `fileMtime`, `fileCtime`), same
  `"File"`/`"Directory"` values, same single path-attribute parameter. `File Property Extractor`
  is the better one — documented description covering all five outputs and the recursive
  directory-size behaviour, plus tags. Zero usage anywhere made retiring free. It also had
  inverted §4.3 ports (absent attribute → `rejected`; nonexistent path → silent pass-through)
  and five entirely undocumented output attributes, none of which now need fixing.
- `Attribute Bulk Array Joiner` — needs a scope decision before it is worth documenting. Joins
  every array attribute with a hard-coded `,` (no separator parameter, while `List Concatenator`
  has one); `ignoreAttributes` is opt-out only, so joining one attribute means enumerating every
  other array attribute on the feature; `_ => {}` silently drops non-scalar elements, so an
  array of two maps becomes `""`; and a single-element array containing a map is *unwrapped*
  rather than joined, making it two operations under one name. The `FlattenerFactory` error
  variant it still uses suggests it was built as an `Attribute Flattener` sibling for collapsing
  CityGML multi-valued attributes before writing to a flat format. If that is the intent, it
  should say so and probably be named for it. Zero usage anywhere.
