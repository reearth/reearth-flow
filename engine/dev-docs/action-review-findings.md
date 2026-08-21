# Action Review Findings

Quality review of the base actions against [action-standard.md](action-standard.md).

> **Resuming this work? Start at [HANDOVER](#handover--state-of-the-audit-as-of-2026-08-21) at
> the bottom of this file.** It carries the current palette state, the 20 actions still awaiting
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

`server/api/internal/app/base_actions.go` exposes **69** actions, down from 105. The gate is now
strict: an action is listed only if it **runs in the shipped build** (§7.1) **and** has passed an
engine-side review. Nothing below is a deletion — every hidden action still executes in a
workflow that names it, so no existing workflow broke.

| Bucket | Count | Trigger to re-expose |
|---|---|---|
| Exposed and audited | 69 | — |
| Does not run in the shipped build | 21 | Its new-geometry port landing (Notion FLOW-DEV-182) |
| **Pending audit** | **12** | An engine-side §8 pass — the list below |
| Flagged for removal | 2 | None; they owe an engine-side deletion |
| Retired on design grounds | 2 | A scope decision, see below |

`Coordinate Frame Reprojector` and `Dissolver` were audited after the rest of this section was
written and are **exposed**; their outcomes are at the bottom. Both were picked because they had
new-geometry support and were assumed to be near-compliant. That held for the reprojector, which
postdates the standard, and did not for Dissolver, whose action long predates it — only its
geometry port is recent. Worth remembering when guessing which of the remaining 12 are cheap.

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

### Pending audit — 12 actions, with preliminary findings

Findings below came from a schema scan plus partial code reading. **They are leads, not verdicts** —
none has had the full `impl:` trace except where stated. Grouped as they were batched; the
grouping is a suggestion, not a constraint.

#### Newly running, never listed (1)

```
Elevation Extractor
  runs:    Its new-geometry port landed in #2384, so it now runs in the shipped build — but that
             PR touched neither `base_actions.go` nor this file, so the action is running,
             unexposed and absent from every list here. This is exactly the §7.2 drift the
             standard warns about, caught while merging. It owes an engine-side §8 pass like the
             rest of this section; nothing else blocks it.
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
- `Line On Line Overlayer` — **fixed** in the overlay batch, and it was the worst form of the
  defect: a grouping attribute absent from a source line did not route the feature anywhere, it
  returned `Err` from `finish` and **failed the whole run**. `process` had already grouped that
  same feature without the attribute, so the two halves disagreed.

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

---

### Addendum — Coordinate Frame Reprojector and Dissolver, audited and exposed

Both were picked on the hypothesis that new-geometry support implied recent authorship and
therefore near-compliance. Half right: `Coordinate Frame Reprojector` was born 2026-07-23, three
weeks after the standard, and is compliant. `Dissolver` predates the standard by months — only
its geometry port is recent (#2368) — and its metadata was pre-#2240 style.

```
Coordinate Frame Reprojector — kept, one structural fix
  impl:    All three parameters read and applied; no dead surface. Both input ports and both
             output ports emitted. Base-point features are consumed rather than forwarded,
             which is the §4.3 merge/join exemption and is consistent for both the valid and
             invalid cases. Genuinely well-built: PROJ transform cached per worker thread,
             num_threads pinned to 1 only in the mode that correlates two streams.
  params:  §3.2/§3.4 — `epsgCode` was a top-level Option validated at build time, so the
             schema advertised a CRS destination with no code and failed at runtime. Now rides
             inside the `crs` variant of a #[serde(tag = "type")] enum, which is the idiom the
             same file already uses for `basePoint`, so the invalid combination cannot be
             expressed and the runtime check is gone.
  note:    Internally the EPSG code narrows to u16 (the bound reearth-flow-geometry uses)
             while Feature Writer and the CityGML writers use u32. Out-of-range codes now get
             an explicit error rather than a deserialization failure. Worth unifying one day.

Dissolver — kept, documentation was materially wrong
  impl:    All three parameters read and applied; `tolerance` reaches both geometry worlds
             (glue_vertices_closer_than / dissolve_leaves). Ports complete.
  desc:    Was "Dissolve Features by Grouping Attributes" — Title Case imperative, no period,
             and it never said what dissolving does to the geometry. Rewritten.
  desc:    **The input constraints were entirely undocumented.** `accepts()` requires the
             Euclidean2D variant, all leaves sharing one coordinate frame, no leaf carrying an
             elevation, and areal leaves only (Polygon / PolygonMesh / TriangularMesh) — so 3D
             geometry, mixed frames, elevated 2D and line strings are all silently routed to
             `rejected`. Note the frame is per-LEAF, so a CRS-framed 2D geometry is fine; it is
             a MIXTURE of frames that is refused. The substantive finding of the batch: a user
             could wire this up correctly and lose every feature with no indication why. The
             sibling overlayers already carry the guidance sentence; it is now here too.
  prior art: The planar computation is universal — FME's equivalent, and GEOS/JTS via PostGIS
             ST_Union ("the result is computed using XY only"), all overlay in XY. Refusing 3D
             input is NOT typical: both accept it and resolve Z by a stated policy (FME exposes
             a five-option Connect Z Mode; PostGIS copies, averages or interpolates). Rejecting
             areal-only input matches FME exactly. The mixed-frame check is stricter than either
             and is the one place we are better — both will silently run planar math across
             mismatched coordinate systems. A Z-policy parameter is the natural enhancement here,
             not a defect to fix. The only production user (PLATEAU4 tran) already chains
             Two Dimension Forcer -> Geometry Filter -> Dissolver, so the constraint is
             load-bearing and the added guidance matches what that workflow already does.
  params:  `tolerance`'s default was unstated; it is 0.0, set by an `unwrap_or` carrying a TODO
             that calls it a compatibility choice. Now documented, including that zero can
             leave slivers between edges that nearly meet.
  params:  Variant description leaked the internal `group_by` spelling into user-facing text.
  tags:    Was empty. Now `spatial` (matching Bufferer, Clipper, Grid Divider) and `aggregation`
             (matching the other accumulating processors).
  ports:   `area` → `features` done in a follow-up commit. The cost estimate was wrong twice
             over: 6 edges, not 27 (that count included Area On Area Overlayer's own `area`
             port), and the files under `engine/testing/data/results` that it called committed
             truth fixtures are `.gitignore`d.
```

### Addendum — Batch 5, audited and exposed

Group A plus `Boundary Extractor`, which had never appeared in any audit batch. The headline: three
of the five already carried standard-compliant metadata, because their new-geometry ports were done
to the standard. The defects left were in what the good prose did not say, and in the translations.

```
Offsetter — kept, OK
  impl:    All three offsets read and applied via `delta()`; the documented default of zero
             matches `unwrap_or(0.0)`; the per-axis units are right. No changes. The one
             judgement call was `coordinate-system`, which sits with the reprojector family
             though Offsetter does not change the CRS — kept, because shifting coordinates is
             what a user would look for under that tag from outside `Geometry`.

Boundary Extractor — kept, note deleted
  impl:    Clean. The 27-line AUDIT NOTE left by Geometry A is gone: `no-boundary` + `rejected`
             fixed the silent data loss it suspected, `keepEmptyBoundaries` and `exteriorOnly`
             are out of the shipped schema, and the description was rewritten — all four of its
             leads resolved by the port (#2369). Its closing paragraph also asserted that ports
             "cannot vary by parameter", which was disproved on 2026-08-20 (`builder_dag.rs`
             derives ports from `with` at runtime), so the note was propagating a false
             constraint as well as a stale one.
  params:  The legacy params survive but `parameter_schema` is `None` under new-geometry, so
             the shipped schema is honest. A migration artifact; left per §"How to use".
  desc:    ja carried a trailing 。 the other four omit.

Geometry Coercer — kept, text only
  impl:    `targetType` required and applied, all three variants traced, §4.3 correct — a
             geometry the target does not apply to passes through on `features`.
  desc:    No terminating period, and "Coerces AND CONVERTS ... to specified target geometry
             types" was a redundant doublet that restated the parameter.
  params:  The block description restated the action name (§3.3).

Hole Counter — kept, undocumented behaviour change
  impl:    Clean. No `infer_output_schema`, so no repeat of the Batch 1 contradiction.
  desc:    Beyond the §2 style hit, it was wrong twice: the action is not limited to polygons,
             and in the shipped build it now ALWAYS writes the attribute — a point, or a
             feature with no geometry, records 0 where the legacy build passed it through
             untouched. Nothing documented that. Kept as correct (0 is a real answer for a
             counter, and it makes the output attribute unconditional) and now stated.
  tags:    Zero, and correct: Vertex Counter, Area Calculator, Bounds Extractor and Coordinate
             Extractor are all untagged siblings doing the same kind of work.
  note:    The commercial-product name in the test doc comment was reworded. It did not reach
             actions.json, but the repo is public and the rule covers comments.

Hole Extractor — kept, name confirmed, port renamed
  desc:    Understated the action: the exterior ring ALWAYS leaves too, so this is a ring
             split, not a hole extraction. A face with no holes still emits its exterior.
  desc:    **es, ja and zh all claimed it adds holes AS ATTRIBUTES.** It emits them as
             features on ports and never wrote an attribute. The English was merely vague, so
             this was introduced in translation — most likely by copying Hole Counter's
             phrasing. Corrected in all four languages.
  ports:   `outershell` → `exterior`, NOT the `outer-shell` this file previously proposed.
             OGC SFA reserves "shell" for solids, and the suite already uses it that way in
             Boundary Extractor ("the bounding shells of a volume") and Geometry Validator
             (`shellOrientation`). `outershell` was the only use of the word for a polygon
             ring. `exterior` is the spec term and pairs with `hole`. Blast radius was 11
             `fromPort:` edges in workflow YAML and zero truth fixtures.
  name:     A rename to `Ring Extractor` was proposed and REJECTED on prior art. "Ring
             extractor" is not an established term in PostGIS, GDAL/OGR, GEOS, JTS, shapely or
             QGIS; "hole" is the established user-facing term while "interior ring" is the API
             term, and PostGIS deliberately glosses both ("the Nth interior ring (hole)").
             The description now uses that gloss. Do not reopen.
  params:  Zero parameters, confirmed as genuine minimalism — which parts you want is answered
             by which of `exterior` / `hole` you wire.
  impl:    §4.3 correct, and better than the prior art: a multi-part geometry rejects only the
             members that bound no area rather than discarding the areas beside them.
             ST_DumpRings hard-errors on any non-polygon input, and no library recurses into
             multi-part geometry this way.
```

**One methodological trap worth recording.** `cargo check -p reearth-flow-action-processor` and
`cargo test -p reearth-flow-action-processor` do **not** compile new-geometry code. That crate's
own `default = []`, and `coordinate_frame_reprojector` is a `#[cfg(feature = "new-geometry")]`
module, so per-crate commands silently skip it — a per-crate check passed while four of its tests
were broken by the parameter restructure. `cargo make test-rs` catches it because `--workspace`
lets cli/worker (both `default = ["new-geometry"]`) unify the feature on. **Verify new-geometry
actions with the workspace command, never with `-p`.**

---

### Addendum — Batch 6, the overlay/CSG/Excel batch, audited and exposed

All five kept and exposed. The batch's lesson is the inverse of Batch 5's: these five predate the
standard by a wide margin, and every one of them had a user-facing property the code contradicted.
Four of the five defects below are invisible to a schema scan, and two were only findable by
reading what a *fixture* asked for and comparing it against the parameter struct.

```
Area On Area Overlayer — kept, `tolerance` did not do what it said
  impl:    All five parameters read and applied. Both data ports emitted, and `rejected` covers
             every intake refusal. `tolerance`, however, was **never a snapping distance**: the
             only thing the code did with it was `min_area = tolerance * tolerance`, a threshold
             below which an intersection piece was discarded. The description promised vertex
             equality and delivered sliver filtering.
  prior art: Unanimous, and against us. The reference product's own transformer defines Tolerance
             as "the minimum distance between geometries in 2D before they are considered equal";
             JTS/GEOS OverlayNG makes it a snapping distance (SnappingNoder, snap-rounding) and
             documents sliver removal as a *consequence* of snapping; and the desktop GIS suite
             users arrive from defines its cluster tolerance the same way, as the minimum distance
             between coordinates before they count as equal, with its clustering pass performing
             the sliver removal. None of the three exposes an area threshold inside the overlay;
             area-based sliver removal lives in separate cleanup tools. So the fix was to
             implement the documented behaviour, not to redescribe the code.
  impl:    `tolerance` now snaps. `overlay::snap_areal_operands_2d` is new: it snaps every operand
             of a group in ONE pass and hands each back dissolved. That is the whole point of the
             signature — `snap_shapes` picks its anchors per call, so snapping pairwise as each
             pair is compared would put the boundary three neighbours share in three places and
             the pieces cut from either side would stop meeting. The sub-tolerance area filter is
             kept as a secondary guard, and because the legacy world's overlay has no snapping
             to drive, `snap_group` is a no-op there. That scopes the SNAPPING to new geometry
             only; the port rename, the parameter spellings and the overlap-count default below
             change both worlds.
  ports:   `area` → `overlaps`. 23 edges across 15 files, every one verified to originate from an
             Area On Area Overlayer node before rewriting. `remnants` already named its
             counterpart honestly; `area` named a geometry type rather than a role.
  params:  `accumulationMode` → `attributeAccumulation` with `useOneFeature`/`dropAttributes`,
             matching Dissolver's audited spelling. Note the semantics genuinely differ from
             Dissolver's: `dropAttributes` here drops the grouping attributes too, so the
             variant description had to say something different rather than be copied.
  params:  `generateList` → `listAttribute`. The name read as a boolean and the value was a name.
  params:  `outputAttribute` retitled "Overlap Count Attribute" and given the default
             `overlayCount` it shares with its sibling. Previously omitting it wrote no count.
  tags:    Was empty. Now `spatial` + `aggregation`, matching Dissolver.

Line On Line Overlayer — kept, three separate properties were false or missing
  desc:    Claimed each intersection point carries "the merged attributes of the lines that meet
             there". It does not: a point feature carries only the grouping attributes, copied
             from an arbitrary member of the group (the last one received). It is the *line*
             output that carries the merged attributes. Rewritten to describe both outputs.
  impl:    The overlap count was written to a **hard-coded `overlayCount`**, undocumented, with
             no parameter to change it — while 11 of the 15 fixture nodes pass an
             `outputAttribute` that the parameter struct does not declare and serde silently
             discards. Someone expected the sibling's parameter and got no error. Now declared,
             defaulting to `overlayCount` so the four workflows that read it still work.
  ports:   §4.3, and the worst instance found so far: a grouping attribute missing from a source
             line returned `Err` from `finish` and failed the entire run. `process` had already
             admitted that same feature to a group without the attribute, so the two halves of
             the action disagreed about whether the absence was survivable. Now carried forward
             as absent.
  params:  `groupBy` and `tolerance` had no title and no description at all. `tolerance` is the
             load-bearing parameter — it decides which crossings split a line, which crossings
             are one crossing, and which segments coincide, and at zero the action splits
             nothing — and it shipped as a bare camelCase key.
  params:  `overlaidListsAttrName` → `listAttribute` (§3.1 bans `attr`), and the list is now
             opt-in rather than always written under a default name. The two workflows that read
             `overlaidLists` name it explicitly.
  ports:   `point` / `line` KEPT. They look off-vocabulary but they match the prior art exactly,
             and the action genuinely has two semantically distinct outputs (§4.3).
  tags:    Was empty. Now `spatial` + `aggregation`.

CSG Builder — kept, the description misnamed the thing it builds
  desc:    Said "**Consecutive** Solid Geometry" — it is Constructive, which its own sibling
             spells correctly. Also four sentences (§2 allows two), and "it detects union,
             intersection, difference" describes a choice the action does not make: it emits all
             three, unconditionally, on three ports.
  params:  `pairIdAttribute` was schema-optional and the action **rejected every feature without
             it** — a node dropped into a workflow unconfigured sent all its input to `rejected`
             and reported nothing. Now required. Renamed `pairId`: it is an expression, not the
             name of an attribute, and the old name said otherwise.
  params:  `createList` + `listAttributeName` folded into one optional `listAttribute`. The pair
             could express a state that did nothing (`createList: true` with no name silently
             wrote no list), and the description of the second was backwards — "the attribute to
             create the list *from*", when it names the attribute the list is written *to*.
  impl:    Output features carry no attribute from either source unless the list is requested.
             True before and after; it is now in the parameter's description.
  ports:   `left` + `right` inputs and the three operation outputs are all correct and emitted.
  tags:    Was empty. Now `spatial` + `3d`.

CSG Evaluator — kept, one port name and one parameter type
  ports:   `nullport` → `empty`. Unwired everywhere, so the rename cost nothing.
  params:  `tolerance` was a required `Code<FlowExpr>` whose one caller passed the constant
             "0.01", and `build` failed outright when `with` was absent. It is now an optional
             plain number: the kernel already falls back to a small distance at or below zero,
             so "when omitted" had a real answer that the schema was hiding behind a required
             expression.
  impl:    `operation: intersection` appears on all 5 fixture nodes and has never been a
             parameter of this action — serde discarded it silently. Removed from the fixtures.
             It is redundant now that CSG Builder emits one port per operation.
  desc:    Said it "computes the resulting mesh"; it produces a Solid. The input constraints were
             undocumented and they are strict — `evaluate` refuses geographic coordinates
             outright, because it merges vertices within a distance and a degree is not a
             distance, and an open or mis-wound boundary yields an arbitrary volume rather than
             an error. Documented, following the pattern Dissolver established.
  tags:    Was empty. Now `spatial` + `3d`.

Excel Writer — kept, and it had an undocumented feature that did not work
  cat:     `File` → `Output`. It is a genuine sink with no output ports.
  impl:    The real finding: an entire configuration surface driven by companion attributes.
             `<key>.formatting`, `<key>.formula` and `<key>.hyperlink` changed how that column's
             cell was written, and none of it was documented anywhere — not the convention, not
             the semicolon-separated `key,value` formatting grammar, not its PascalCase
             alignment and underline values.
  impl:    Four defects, all in code no test covered:
             1. Applying `.formatting` wrote an empty string with the format into the cell that
                had just been given its value, so **formatting a cell erased its contents**.
             2. Companion attributes were themselves keys of the feature, so they each got their
                own visible column holding the raw directive string.
             3. Numbers were stringified and written as text, so a numeric column arrived in
                Excel as text and could not be summed or sorted. Booleans and composites became
                empty strings.
             4. The header row came from the FIRST feature alone, and any later feature carrying
                a key it lacked returned `Err` and **failed the run**. Optional attributes are
                ordinary, so this was easy to hit.
  scope:   `.formatting` removed rather than fixed (user decision, 2026-08-21). Its grammar has
             nowhere legal to be documented — it is not a parameter, so it gets no title or
             description, and §2 caps the description at two sentences — so fixing it would have
             shipped a feature users cannot learn to use. `.formula` and `.hyperlink` are
             self-describing enough to name in the description, and they stay. Zero usage
             anywhere made the removal free. The other three defects are fixed: header from the
             union of all features, companion keys excluded from the columns, one write per cell,
             and numbers and booleans written in their own type.
  impl:    §7.3 — a 60-line `#[allow(dead_code)] write_map_entry` and a commented-out
             `parse_row_formatting` block both removed, along with the duplicate `ExcelWriterParam`
             that existed only to pass one field between the two files.
  params:  `output` and `sheetName` gained titles; `output`'s per-feature expression evaluation
             (one workbook per distinct path) was undocumented.
  tags:    Was empty. Now `excel`, matching every other audited writer, which all carry their
             format. `excel` is new §6 vocabulary.
```

**Deferred, with triggers.**

- **Line On Line Overlayer's intersection points carry only grouping attributes**, from an
  arbitrary member of the group, where the prior art gives them the attributes of the lines that
  cross there. Doing it properly means tracking, per candidate crossing, which other feature
  produced it, through `polyline_intersections` and `split_polyline` and the point dedup — and
  the legacy world reaches its split through a geometry-crate call that returns no provenance at
  all. Revisit when the legacy overlay path is removed, so it only has to be built once.
- **CSG Builder's `rejected` port mixes two populations**: a feature whose geometry is not a
  solid, and a feature whose partner never arrived. Feature Joiner separates these
  (`unjoined-requestor` / `unjoined-supplier`). Adding an `unpaired` port is a data-loss change
  (§4.3) and wants the same treatment as the Attribute Duplicate Filter `duplicate` port.
  Revisit when that one is decided, so the pair is settled together.
- **The group-key `filter_map` is a family-wide pattern, not a defect of one action.** Both
  overlayers and `Dissolver` build their group key with
  `group_by.iter().filter_map(|a| attributes.get(a))`, so a feature missing one grouping
  attribute produces a *shorter* key and can collide with a feature missing a different one.
  Changing it changes grouping behaviour for all three. Revisit when `Attribute Duplicate
  Filter`'s known key-collision defect is fixed, and fix them with one shared rule.

**Verification note.** `test-qc` ignores every plateau4 quality-check case
(`skipNewGeometry`), so it does not execute any workflow this batch touched. The suite that does
is `cargo test -p workflow-tests -- --test-threads=4`, the legacy world, 185 cases — run before
and after. Leaving `snap_group` a no-op in the legacy build is what makes that run useful: the
snapping is the one change those cases cannot see, so they stay a real regression check on the
rewiring and the parameter renames, which they DO see, rather than a wall of expected diffs.
