# Action Review Findings

Phase 3 quality review of the 74 base actions against [action-standard.md](action-standard.md).

**How to use:**
- Fill each action with either `ActionName — OK` or the checklist format from §7 of the standard
- Phase 4 improvement PRs should reference this file and delete completed sections as fixes land
- File is deleted when all sections are cleared

**Global fix (applies to all base actions):**
- `ports` — all ports currently named `default` must be assessed and renamed (§4.2); not called out per-action below. Check the Rust implementation to confirm semantics before renaming: if the port carries the primary feature stream rename to `features`; if it is a catch-all for unmatched features consider `unfiltered` instead. Make the change in Rust, then regenerate the schema (`cargo make schema-base`). Do not treat this as a mechanical find-replace.

---

## Debug (5)

<!-- Session 1 -->

```
EchoProcessor
  name:    → "Echo Processor"
  desc:    not imperative — "Debug Echo Features to Logs"; suggest "Echo features to logs and pass them through unchanged."
  tags:    empty — `debug` duplicates category (§6); no other established vocabulary terms apply; consider proposing `logging`

EchoSink
  name:    → "Echo Sink"
  desc:    not imperative — identical to EchoProcessor; suggest "Echo features to logs and discard them."
  tags:    empty — same constraint as EchoProcessor

FeatureCounter
  name:    → "Feature Counter"
  params:  countStart — marked required but has a sensible default (0 or 1); should be optional with a schema default (§3.2)
           outputAttribute — title "Output Attribute" is generic; suggest "Count Attribute" (§3.3)
           ordering — `groupBy` (optional) is defined between two required params; once countStart is made optional, correct order: outputAttribute → countStart → groupBy (§3.5)
  tags:    empty — suggest ["aggregation", "attribute"]; `debug` duplicates category (§6)

NoopProcessor
  name:    → "Noop Processor"
  desc:    noun phrase — "No-Operation Pass-Through Processor"; suggest "Pass features through unchanged."
  tags:    empty — `debug` duplicates category (§6); no other established vocabulary terms apply

NoopSink
  name:    → "Noop Sink"
  desc:    noun phrase with parenthetical — "No-Operation Sink (Discard Features)"; suggest "Discard all incoming features."
  tags:    empty — same constraint as NoopProcessor
```

---

## Input (9)

<!-- Session 2 -->

```
CityGmlReader
  name:    → "CityGML Reader"
  params:  flatten — missing title and description (§3.3)
           dataset — description example "data.csv" copy-pasted from CsvReader; should
             reference a .gml file (§3.3)
           ordering — `flatten` sits between `dataset` and `inline`; correct order:
             dataset → inline → flatten (§3.5)

CsvReader
  name:    → "CSV Reader"
  desc:    title-case — "Read Features from CSV or TSV File"; suggest "Reads features
             from CSV and TSV files."
  params:  ordering — `format` is required but is 3rd in properties (after dataset,
             encoding); move to first (§3.5)
           encoding — description is paragraph-length; §3.3 prefers ≤2 sentences
           headerRows — description exceeds 2 sentences (§3.3)
  tags:    ["csv"] — 1 tag; suggest adding `file`

FeatureCreator
  name:    → "Feature Creator"
  desc:    title-case — "Generate Custom Features Using Scripts"; suggest "Creates
             features from a script expression that returns one or more attribute maps."
  params:  creator — description opens with imperative "Write a script expression…";
             should describe, not instruct; suggest "Script expression that returns a map
             (single feature) or an array of maps (multiple features)."
  tags:    empty — no fitting established vocabulary terms; consider proposing `scripting`

FilePathExtractor
  name:    → "File Path Extractor"
  tags:    ["file-system"] — not in established vocabulary; replace with `file`; 1 tag
             acceptable (no strong second candidate)

GeoJsonReader
  name:    → "GeoJSON Reader"
  params:  dataset — description example "data.csv" copy-pasted; should reference a
             .geojson file (§3.3)
  tags:    ["geojson"] — 1 tag; suggest adding `vector`

GeoPackageReader
  name:    → "GeoPackage Reader"
  params:  schema-level description missing (§3.3)
           attributeFilter, batchSize, force2D, includeMetadata, layerName, readMode,
             spatialFilter, tileFormat — all missing title and/or description (§3.3)
           readMode enum variants ("features", "tiles", "all", "metadataOnly") — no
             per-variant descriptions (§3.4)
           tileFormat enum variants ("png", "jpeg", "webp") — no per-variant descriptions
             (§3.4)
           10 params — exceeds 8; justification required (§3.5); batchSize looks like
             implementation leakage — evaluate removal (§3.5)
           ordering — alphabetical, not usability-ordered; suggest: dataset → readMode →
             layerName → attributeFilter → spatialFilter → includeMetadata → force2D →
             tileFormat → batchSize (§3.5)
  tags:    ["geopackage"] — 1 tag; suggest adding `vector` (and `raster` if tile reading
             is a primary use case)

JsonReader
  name:    → "JSON Reader"
  tags:    ["json"] — 1 tag; acceptable (no strong second candidate)

ShapefileReader
  name:    → "Shapefile Reader"
  params:  allowEmptyPath — description mentions "Rhai `()`"; remove implementation
             detail; suggest "If true, a null dataset path produces zero features instead
             of an error."
           encoding — description is paragraph-length; §3.3 prefers ≤2 sentences
           force2d — should be `force2D` per camelCase (§3.1)
           dataset — description example "data.csv" copy-pasted; should reference a .zip
             file (§3.3)
           ordering — allowEmptyPath (edge-case) is first; correct order: dataset →
             inline → encoding → force2D → allowEmptyPath (§3.5)
  tags:    ["shapefile"] — 1 tag; suggest adding `vector`

SqlReader
  name:    → "SQL Reader"
  desc:    title-case — "Read Features from SQL Database"; suggest "Reads features from
             a SQL database."
  tags:    ["database"] — 1 tag; acceptable (no strong second candidate)
```

---

## Output (12)

<!-- Session 3 — EchoSink and NoopSink already reviewed in Debug section above -->

```
Cesium3DTilesWriter
  name:    → "Cesium 3D Tiles Writer"
  desc:    title-case — "Export Features as Cesium 3D Tiles for Web Visualization"; suggest
             "Writes features to Cesium 3D Tiles format for 3D web visualization."
  params:  schema-level description missing (§3.3)
           skipUnexposedAttributes — title "Skip unexposed Attributes" inconsistent
             capitalization; should be "Skip Unexposed Attributes"
           ordering — alphabetical; suggest: output → maxZoom → minZoom → attachTexture →
             dracoCompression → schemaKey → skipUnexposedAttributes → compressOutput (§3.5)
  tags:    ["3d-tiles", "3d"] — `3d-tiles` not in vocabulary; suggest ["3d", "tiling"]

CityGmlWriter
  name:    → "CityGML Writer"
  params:  schema-level description missing (§3.3)
           epsgCode, lodFilter, output, prettyPrint — all missing title (§3.3)
           ordering — required `output` is 3rd; correct order: output → prettyPrint →
             lodFilter → epsgCode (§3.5)

CsvWriter
  name:    → "CSV Writer"
  params:  format, output — missing title (§3.3)
           ordering — `geometry` (optional) sits between two required params; correct order:
             format → output → geometry (§3.5)
  tags:    ["csv"] — 1 tag; suggest adding `file`

GeoJsonWriter
  name:    → "GeoJSON Writer"
  params:  groupBy, output — missing title (§3.3)
           ordering — required `output` is last; correct order: output → groupBy (§3.5)
  tags:    ["geojson"] — 1 tag; suggest adding `vector`

GeoPackageWriter
  name:    → "GeoPackage Writer"
  desc:    "with proper SQLite structure, spatial indexing, and metadata tables" —
             implementation detail; suggest "Writes features to a GeoPackage (.gpkg) file."
  params:  createSpatialIndex, geometryColumn, geometryType, output, overwrite, srsId,
             tableName — all missing title (§3.3)
           ordering — alphabetical; suggest: output → tableName → geometryType →
             geometryColumn → srsId → overwrite → createSpatialIndex (§3.5)
  tags:    ["geopackage"] — 1 tag; suggest adding `vector`

JsonWriter
  name:    → "JSON Writer"
  params:  converter, output — missing title (§3.3)
           ordering — required `output` is last; correct order: output → converter (§3.5)

MVTWriter
  name:    → "MVT Writer"
  desc:    "with TileJSON 3.0.0 metadata" — version detail; suggest "Writes features to
             Mapbox Vector Tiles (MVT) format."
  params:  schema-level description contains implementation details (internal file paths,
             HTTP root note); replace with a plain summary (§3.3)
           extent — tile coordinate resolution (default 4096); users rarely adjust this;
             evaluate for removal as implementation leakage (§3.5)
           9 params — exceeds 8; justification required (§3.5)
           ordering — alphabetical; suggest: output → layerName → minZoom → maxZoom →
             compressOutput → schemaKey → skipUnexposedAttributes → colonToUnderscore →
             extent (§3.5)
  tags:    ["mvt"] — not in vocabulary; suggest ["vector", "tiling"]

ShapefileWriter
  name:    → "Shapefile Writer"
  params:  groupBy, output — missing title (§3.3)
           ordering — required `output` is last; correct order: output → groupBy (§3.5)
  tags:    ["shapefile"] — 1 tag; suggest adding `vector`

XmlWriter
  name:    → "XML Writer"
  params:  output — missing title (§3.3)

ZipFileWriter
  name:    → "Zip File Writer"
  params:  output — missing title (§3.3); description "Output path" too brief — should
             describe the accepted expression types
  tags:    ["file-system", "compression"] — `file-system` not in vocabulary; replace with
             `file`; `compression` now in vocabulary — keep as second tag
```

---

## Attribute (8)

<!-- Session 4 -->

```
AttributeAggregator
  name:    → "Attribute Aggregator"
  desc:    title-case — "Group and Aggregate Features by Attributes"; suggest "Groups
             features by attributes and aggregates values within each group."
  params:  aggregateAttributes, calculation, calculationAttribute, calculationValue, method
             — all have title but missing description (§3.3)
           ordering — required and optional interleaved; correct order: method →
             aggregateAttributes → calculationAttribute → calculationValue → calculation (§3.5)
  tags:    ["aggregate"] — not in vocabulary; replace with `aggregation`

AttributeConversionTable
  name:    → "Attribute Conversion Table"
  desc:    title-case — "Transform Feature Attributes Using Lookup Tables"; suggest
             "Transforms attributes using rules defined in a lookup table (CSV, TSV, or JSON)."
  params:  schema-level description missing (§3.3)
           ConversionTableFormat enum — no per-variant descriptions; property description
             covers them implicitly but borderline (§3.4)
           ordering — required params `format` and `rules` are not first; correct order:
             format → rules → dataset → inline (§3.5)
  tags:    ["mapping"] — now in vocabulary; 1 tag acceptable (no strong second candidate)

AttributeFlattener
  name:    → "Attribute Flattener"
  desc:    title-case — "Flatten Nested Object Attributes into Top-Level Attributes"; suggest
             "Flattens nested map attributes into individual top-level attributes."
  params:  schema-level description missing (§3.3)
  tags:    ["hierarchy"] — not in vocabulary; no established alternative; remove tag
             (0 tags acceptable — name and description provide sufficient discovery)

AttributeManager
  name:    → "Attribute Manager"
  desc:    title-case — "Create, Convert, Rename, and Remove Feature Attributes"; suggest
             "Creates, converts, renames, or removes feature attributes based on a
             configurable list of operations."
  params:  schema-level description missing (§3.3)
           Method enum ("convert", "create", "rename", "remove") — no per-variant
             descriptions; plain enum type cannot hold descriptions — restructure as oneOf
             or add variant explanations to the method property description (§3.4)
  tags:    empty — `attribute` duplicates category (§6); no other established vocabulary
             terms apply; 0 tags acceptable

AttributeMapper
  name:    → "Attribute Mapper"
  desc:    title-case — "Transform Feature Attributes Using Expressions and Mappings";
             suggest "Maps or transforms feature attributes using expressions and value
             assignments."
  params:  schema-level description missing (§3.3)
  tags:    ["mapping"] — now in vocabulary; 1 tag acceptable

BulkAttributeRenamer
  name:    → "Bulk Attribute Renamer"
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

NullAttributeMapper
  name:    → "Null Attribute Mapper"
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

StatisticsCalculator
  name:    → "Statistics Calculator"
  params:  groupBy — title "Group by" should be "Group By"
           groupId — title "Group id" should be "Group ID"
  tags:    ["statistics", "aggregate"] — `aggregate` not in vocabulary; replace with
             `aggregation`; `statistics` now in vocabulary; suggest ["aggregation",
             "statistics"]
```

---

## Filter (7)

<!-- Session 5 -->

---

## Merge (3)

<!-- Session 5 -->

---

## Feature (1) · File (2) · Transform (5)

<!-- Session 6 -->

---

## Geometry A (12)

<!-- Session 7 — AppearanceRemover through ImageRasterizer -->

---

## Geometry B (11)

<!-- Session 8 — PolygonNormalExtractor through VerticalReprojector -->
