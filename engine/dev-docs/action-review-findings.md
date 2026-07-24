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

## Attribute (8)

<!-- Session 4 -->

```
Attribute Manager
  desc:    title-case — "Create, Convert, Rename, and Remove Feature Attributes"; suggest
             "Creates, converts, renames, or removes feature attributes based on a
             configurable list of operations."
  params:  schema-level description missing (§3.3)
           Method enum ("convert", "create", "rename", "remove") — no per-variant
             descriptions; plain enum type cannot hold descriptions — restructure as oneOf
             or add variant explanations to the method property description (§3.4)
  tags:    empty — `attribute` duplicates category (§6); no other established vocabulary
             terms apply; 0 tags acceptable

Bulk Attribute Renamer
  desc:    title-case — "Rename Feature Attributes in Bulk"; suggest "Renames feature
             attributes in bulk by adding or removing a prefix or suffix, or replacing text."
  params:  RenameAction enum values PascalCase — AddPrefix, AddSuffix, RemovePrefix,
             RemoveSuffix, StringReplace must be camelCase: addPrefix, addSuffix,
             removePrefix, removeSuffix, replaceText (§3.4)
           RenameType enum values PascalCase — All, Selected must be camelCase: all,
             selected (§3.4)
           renameType — description "Choose whether to..." is instructive; suggest "Scope
             of the rename operation: all attributes or a selected subset."
           selectedAttributes — description references old enum value names; update when
             enum is renamed
  tags:    empty — `attribute` duplicates category (§6); no other established vocabulary
             terms apply; 0 tags acceptable

Null Attribute Mapper
  desc:    "Replace" should be "Replaces" (verb-first present tense, third-person singular)
  params:  schema-level description "NullAttributeMapper parameters" is a name restatement
             — replace with a meaningful summary (§3.3)
           defaultReplacement, mappings, nullDefinition, routeNullFeatures, scope — all
             missing title (§3.3)
           NullKind enum — "null" variant description "AttributeValue::Null" and
             "emptyString" variant description "AttributeValue::String(\"\")" expose Rust
             type names; replace with plain language (§3.4)
           routeNullFeatures — description mentions port name "hasNull"; avoid port
             references in parameter descriptions (§2 spirit)
           ordering — alphabetical; suggest: scope → mappings → defaultReplacement →
             nullDefinition → routeNullFeatures (§3.5)
  tags:    ["mapping"] — now in vocabulary; 1 tag acceptable

Statistics Calculator
  params:  groupBy — title "Group by" should be "Group By"
           groupId — title "Group id" should be "Group ID"
  tags:    ["statistics", "aggregate"] — `aggregate` not in vocabulary; replace with
             `aggregation`; `statistics` now in vocabulary; suggest ["aggregation",
             "statistics"]
```

---

## Feature (1) · File (2) · Transform (4)

<!-- Session 7 -->

```
Feature File Path Extractor
  desc:    title-case — "Extract File Paths from Dataset to Features"; suggest "Extracts
             file paths from a dataset source and creates one feature per path."
  params:  extractArchive — required but is a boolean with an obvious false default;
             evaluate as optional with default false (§3.2)
           ordering — required params `extractArchive` and `sourceDataset` are not first;
             `destPrefix` (optional) is 1st; correct order: sourceDataset → extractArchive
             → destPrefix (§3.5)
  ports:   inputPorts `default` — global note
           outputPorts `default` — needs semantic name; `unfiltered` semantics worth
             clarifying during Phase 4
  tags:    ["file", "path"] — `path` not in vocabulary; suggest ["file"]

Directory Decompressor
  desc:    "from specified attributes" is slightly implementation-leaky; suggest
             "Decompresses archive files referenced in feature attributes and emits the
             extracted paths."
  params:  archiveAttributes, findDeepestSingleFolder — both missing title (§3.3)
  ports:   inputPorts `default` — global note
           outputPorts `default` — global note; no `rejected` port — evaluate whether
             failed extractions need a rejected route (§4.3)
  tags:    ["file-system", "compression"] — `file-system` not in vocabulary; replace with
             `file`; `compression` in vocabulary; suggest ["file", "compression"]

File Property Extractor
  params:  filePathAttribute — missing title (§3.3)
  ports:   inputPorts `default` — global note
           outputPorts `default` — needs semantic name; `rejected` ✓
  tags:    ["file-system"] — not in vocabulary; replace with `file`

Feature Transformer
  params:  transformers — missing title (§3.3)
           Transform.expr — missing title (§3.3)
  ports:   inputPorts `default`, outputPorts `default` — global note; rename both to
             `features`
  tags:    empty — 0 tags acceptable

List Exploder
  params:  sourceAttribute — missing title (§3.3)
  ports:   inputPorts `default`, outputPorts `default` — global note
  tags:    ["list"] — in vocabulary ✓

XML Fragmenter
  desc:    suggest "Splits an XML document into features by matching element patterns,
             emitting each matched element as a separate feature."
  params:  oneOf with a single variant suggests incomplete design — other source types
             planned but only "url" implemented
           attribute, elementsToExclude, elementsToMatch, source — all missing title and
             description within the oneOf variant (§3.3)
  ports:   inputPorts `default`, outputPorts `default` — global note; evaluate adding
             `rejected` for malformed XML (§4.3)
  tags:    ["xml"] — in vocabulary ✓

XML Validator
  desc:    "against XSD schemas" inaccurate for syntax/namespace modes; "with
             success/failure routing" references port behavior; suggest "Validates XML
             documents for syntax, namespace conformance, or XSD schema compliance."
  params:  schema title "XmlValidatorParam" — inconsistent casing; should be "XML
             Validator Parameters"
           schema-level description missing (§3.3)
           attribute, inputType, validationType — all missing title and description (§3.3)
           ValidationType enum ("syntax", "syntaxAndNamespace", "syntaxAndSchema") — no
             per-variant descriptions; plain enum type (§3.4)
           XmlInputType enum ("file", "text") — no per-variant descriptions; plain enum
             type (§3.4)
  ports:   inputPorts `default` — global note; outputPorts `success`, `failed` ✓;
             evaluate adding `rejected` for parse errors (§4.3)
  tags:    ["xml", "validate"] — `validate` not in vocabulary; `validation` is; correct
             to ["xml", "validation"]
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
