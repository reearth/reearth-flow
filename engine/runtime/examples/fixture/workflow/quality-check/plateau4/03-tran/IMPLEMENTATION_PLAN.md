# PLATEAU4 Transportation Quality Check Implementation Plan

## Overview
Implementation of quality inspection workflow for PLATEAU4 transportation models (道路等の交通モデル) based on FME workflow analysis.

**Target File**: `runtime/examples/fixture/workflow/quality-check/plateau4/03-tran/workflow.yml`

**Reference FME Template**: `PLATEAU4 品質検査03 道路等の交通モデル.fmwt`

---

## Implementation Strategy

### Phase 1: New Action Development
**ONE dedicated action** will be created:
- `PLATEAU4.TransportationSurfaceReferenceValidator`

### Phase 2: Workflow Implementation
**Existing actions** will be used with workflow configuration:
- L-tran-01: Adjacent Road instance overlap detection
- L-tran-02: TrafficArea/AuxiliaryTrafficArea overlap within same Road
- Surface orientation validation

### Phase 3: Action Enhancement (Optional)
**Modifications to existing actions** may be needed:
- Enhanced grouping parameters in `LineOnLineOverlayer`
- LOD-based filtering improvements
- Vertical surface handling in orientation validation

---

## Quality Check Requirements (From PLATEAU Standard)

### 1. XLink Reference Validation (L-tran-03)
**PLATEAU Standard**: "Road.lod2MultiSurface/lod3MultiSurface must reference all child TrafficArea and AuxiliaryTrafficArea surfaces"

**FME Implementation**: Uses custom XLink checker transformer

**Re:Earth Flow Implementation**:
- ✅ **NEW ACTION REQUIRED**: `PLATEAU4.TransportationSurfaceReferenceValidator`
- Location: `runtime/action-plateau-processor/src/plateau4/transportation_surface_reference_validator.rs`

### 2. Adjacent Road Instance Overlap (L-tran-01)
**PLATEAU Standard**: "Adjacent Road instances must share boundaries (no gaps/overlaps) within 10cm tolerance"

**FME Implementation**:
- `LineOnLineOverlayer` with `GROUP_BY { _lod _folder_package }`
- `CLEANING_TOLERANCE { 0.1 }` (10cm)
- **Excludes `gml_parent_id`** → checks across different Road instances

**Re:Earth Flow Implementation**:
- ✅ **USE EXISTING ACTION**: `LineOnLineOverlayer`
- Configuration:
  ```yaml
  groupBy:
    - lod
    - package  # tran/rwy/trk/squr/wwy
  # NOTE: Do NOT include gml_parent_id for L-tran-01
  outputAttribute: lineOverlap
  tolerance: 0.1  # 10cm
  ```

### 3. TrafficArea/AuxiliaryTrafficArea Overlap Within Same Road (L-tran-02)
**PLATEAU Standard**: "TrafficArea and AuxiliaryTrafficArea within the same Road instance must share boundaries (LOD2/3 only)"

**FME Implementation**:
- `LineOnLineOverlayer_2` with `GROUP_BY { _lod gml_parent_id _folder_package }`
- `CLEANING_TOLERANCE { 0.1 }` (10cm)
- **Includes `gml_parent_id`** → checks within same Road instance only

**Re:Earth Flow Implementation**:
- ✅ **USE EXISTING ACTION**: `LineOnLineOverlayer`
- Configuration:
  ```yaml
  groupBy:
    - lod
    - gmlParentId  # Key difference from L-tran-01
    - package
  outputAttribute: lineOverlap
  tolerance: 0.1  # 10cm
  ```

### 4. Surface Orientation Validation
**PLATEAU Standard**: "Surfaces must have correct orientation (exterior faces outward)"

**FME Implementation**:
- `OrientationExtractor` → `_orientation` attribute
- `Tester` → Flag if `_orientation NOT_BEGINS_WITH "left"`
- Mark as `_issue = "Incorrect Orientation"`

**Known Limitation (From FME Workflow Note 1)**:
> LOD3の道路・鉄道・徒歩道・広場、交通領域、交通補助領域の面のうち、路肩立ち上がり部などの鉛直の面については、面の向きが識別できずに「面の向き不正」エラーが検出されることがあります。

**Re:Earth Flow Implementation**:
- ✅ **USE EXISTING ACTION**: `OrientationExtractor`
- ⚠️ **POTENTIAL ENHANCEMENT**: Add vertical surface detection to reduce false positives
  - Check if surface normal Z-component ≈ 0
  - For LOD3 + vertical surfaces → Report as WARNING instead of ERROR
  - Improves upon FME workflow's manual review requirement

### 5. General Geometry Validation
**Standard validations** from PLATEAU specification 6.3.2 (論理一貫性):
- Duplicate points
- Self-intersection
- Ring closure
- Planarity
- Solid boundary validation

**Re:Earth Flow Implementation**:
- ✅ **USE EXISTING ACTIONS**:
  - `GeometryValidator`
  - `SolidBoundaryValidator`
  - `PlanarityFilter`

---

## New Action Specification

### PLATEAU4.TransportationSurfaceReferenceValidator

**Purpose**: Validate that Road instances correctly reference all child TrafficArea and AuxiliaryTrafficArea surfaces via XLink.

**Input**:
- CityGML features (Road, TrafficArea, AuxiliaryTrafficArea)
- LOD level (lod2, lod3)

**Processing Logic**:
1. Extract Road instances
2. Parse `lod2MultiSurface` and `lod3MultiSurface` XLink references
3. Collect all child TrafficArea/AuxiliaryTrafficArea surface IDs
4. Compare referenced IDs vs. actual child IDs
5. Detect orphaned surfaces (not referenced by parent Road)

**Output Ports**:
- `passed`: Roads with correct references
- `failed`: Roads with missing/incorrect references
- `orphaned`: TrafficArea/AuxiliaryTrafficArea not referenced by any Road

**Output Attributes**:
- `unreferencedSurfaceNum`: Count of orphaned surfaces
- `missingReferenceIds`: List of surface IDs not referenced
- `orphanedSurfaceIds`: List of orphaned child surfaces

**Implementation File**:
- `runtime/action-plateau-processor/src/plateau4/transportation_surface_reference_validator.rs`

**Registration**:
- Add to `runtime/action-plateau-processor/src/plateau4/mapping.rs`

**Schema/i18n**:
- `schema/i18n/actions/plateau4/transportation_surface_reference_validator.yml`

---

## Workflow Structure

### Input Parameters
```yaml
with:
  cityGmlPath: <path_to_citygml>
  cityCode: <5_digit_code>
  codelists: <path_to_codelists>
  schemas: <path_to_schemas>
  outputPath: <output_directory>
  targetPackages:
    - tran
    - rwy
    - trk
    - squr
    - wwy
```

### Main Workflow Sections

#### 1. File Reading & Preparation
- `FeatureCreator` → Initialize workflow
- `DirectoryDecompressor` → Extract archives
- `FeatureFilePathExtractor` → Extract GML file paths
- `FeatureCounter` → Assign file index
- `PLATEAU4.UDXFolderExtractor` → Extract package structure

#### 2. XLink Reference Validation (L-tran-03)
```yaml
- name: TransportationSurfaceReferenceValidator
  type: action
  action: PLATEAU4.TransportationSurfaceReferenceValidator
  with:
    # Validates Road → TrafficArea/AuxiliaryTrafficArea references

- name: AttributeAggregator-xlink-errors
  type: action
  action: AttributeAggregator
  with:
    aggregateAttributes:
      - newAttribute: Index
        attribute: fileIndex
      - newAttribute: Folder
        attribute: package
      - newAttribute: LOD
        attribute: lod
      - newAttribute: gmlID
        attribute: gmlId
      - newAttribute: FeatureType
        attribute: featureType
    calculation: |
      env.get("__value").unreferencedSurfaceNum
    calculationAttribute: "xlink参照エラー数"
    method: count

- name: FileWriter-xlink-errors
  type: action
  action: FileWriter
  with:
    format: tsv
    output: |
      file::join_path(env.get("outputPath"), "xlink参照エラー.tsv")
```

#### 3. CityGML Reading & LOD Splitting
```yaml
- name: FeatureCityGmlReader
  type: action
  action: FeatureCityGmlReader
  with:
    format: citygml
    flatten: true
    dataset: |
      env.get("__value")["gmlPath"]

- name: LodSplitter
  type: subGraph
  subGraphId: lod_splitter_with_dm
  # Splits features by LOD level → lod0, lod1, lod2, lod3 ports
```

#### 4. Surface Orientation Validation
```yaml
- name: OrientationExtractor
  type: action
  action: OrientationExtractor
  with:
    outputAttribute: _orientation

- name: Tester-IncorrectOrientation
  type: action
  action: FeatureFilter
  with:
    conditions:
      - expr: |
          !env.get("__value")._orientation.starts_with("left")
        outputPort: incorrectOrientation

- name: AttributeCreator-OrientationIssue
  type: action
  action: AttributeMapper
  with:
    mappers:
      - attribute: _issue
        expr: |
          "Incorrect Orientation"
      - attribute: _orientation
        expr: |
          env.get("__value")._orientation

# Optional Enhancement: Vertical Surface Handler
- name: VerticalSurfaceDetector
  type: action
  action: RhaiCaller
  with:
    isTarget: |
      env.get("__value").lod == "3"
    process: |
      # Check if surface is vertical (normal Z ≈ 0)
      let normal_z = env.get("__value").surfaceNormalZ ?? 1.0;
      let is_vertical = normal_z.abs() < 0.1;
      #{
        "is_vertical_surface": is_vertical,
        "_issue_severity": if is_vertical { "WARNING" } else { "ERROR" }
      }
```

#### 5. L-tran-01: Adjacent Road Instance Overlap (LOD1)
```yaml
# Convert surfaces to 2D
- name: TwoDimensionForcer
  type: action
  action: TwoDimensionForcer

# Dissolve by instance to get boundaries
- name: GeometryDissolver
  type: action
  action: GeometryDissolver
  with:
    groupBy:
      - gmlId
      - lod
      - fileIndex
      - package

# Extract boundaries as LineStrings
- name: GeometryCoercer-LineString
  type: action
  action: GeometryCoercer
  with:
    coercerType: lineString

# Filter LOD1 only
- name: FeatureFilter-LOD1
  type: action
  action: FeatureFilter
  with:
    conditions:
      - expr: |
          env.get("__value").lod == "1"
        outputPort: lod1

# Detect overlaps ACROSS Road instances (no gml_parent_id grouping)
- name: LineOnLineOverlayer-L-tran-01
  type: action
  action: LineOnLineOverlayer
  with:
    groupBy:
      - lod
      - package
    # NOTE: Excluding gmlParentId → checks across instances
    outputAttribute: lineOverlap
    tolerance: 0.1  # 10cm

- name: FeatureFilter-HasOverlap
  type: action
  action: FeatureFilter
  with:
    conditions:
      - expr: |
          env.get("__value").lineOverlap > 1
        outputPort: overlap
```

#### 6. L-tran-02: TrafficArea Overlap Within Same Road (LOD2/3)
```yaml
# Filter LOD2/3 only
- name: FeatureFilter-LOD23
  type: action
  action: FeatureFilter
  with:
    conditions:
      - expr: |
          env.get("__value").lod != "1"
        outputPort: lod23

# Detect overlaps WITHIN same Road instance (with gml_parent_id grouping)
- name: LineOnLineOverlayer-L-tran-02
  type: action
  action: LineOnLineOverlayer
  with:
    groupBy:
      - lod
      - gmlParentId  # KEY DIFFERENCE: Groups by parent Road
      - package
    outputAttribute: lineOverlap
    tolerance: 0.1  # 10cm

- name: FeatureFilter-HasOverlap-LOD23
  type: action
  action: FeatureFilter
  with:
    conditions:
      - expr: |
          env.get("__value").lineOverlap > 1
        outputPort: overlap
```

#### 7. Results Aggregation & Output
```yaml
# Merge all validation results
- name: FeatureMerger-Results
  type: action
  action: FeatureMerger
  with:
    requestorAttribute:
      - fileIndex
      - featureType
      - lod
    supplierAttribute:
      - fileIndex
      - featureType
      - lod

# Create summary report
- name: AttributeMapper-Summary
  type: action
  action: AttributeMapper
  with:
    mappers:
      - attribute: Index
        expr: env.get("__value").fileIndex
      - attribute: Folder
        expr: env.get("__value")["package"]
      - attribute: Filename
        expr: file::extract_filename(env.get("__value").cityGmlPath)
      - attribute: "フィーチャータイプ"
        expr: env.get("__value").featureType
      - attribute: "LOD"
        expr: env.get("__value").lod
      - attribute: "インスタンス数"
        expr: env.get("__value").totalCount
      - attribute: "面のエラー"
        expr: env.get("__value").numInvalidSurface ?? 0
      - attribute: "面の向き不正"
        expr: env.get("__value").numInvalidOrientation ?? 0
      - attribute: "隣接面重複"
        expr: env.get("__value").adjacentSurfaceOverlap ?? 0
      - attribute: "xlink参照エラー"
        expr: env.get("__value")["xlink参照エラー数"] ?? 0

# Write Excel output
- name: FileWriter-Summary
  type: action
  action: FileWriter
  with:
    format: tsv
    output: |
      file::join_path(env.get("outputPath"), "検査結果一覧.tsv")
```

---

## TODO Checklist

### Phase 1: New Action Development
- [ ] **Create action implementation file**
  - [ ] File: `runtime/action-plateau-processor/src/plateau4/transportation_surface_reference_validator.rs`
  - [ ] Implement `TransportationSurfaceReferenceValidatorFactory`
  - [ ] Define parameter struct with schema
  - [ ] Implement XLink reference parsing logic
  - [ ] Implement surface-to-parent matching logic
  - [ ] Add output ports: `passed`, `failed`, `orphaned`
  - [ ] Add output attributes: `unreferencedSurfaceNum`, `missingReferenceIds`, `orphanedSurfaceIds`

- [ ] **Register action in mapping**
  - [ ] Add to `runtime/action-plateau-processor/src/plateau4/mapping.rs`
  - [ ] Export factory in module

- [ ] **Add i18n schema**
  - [ ] Create `schema/i18n/actions/plateau4/transportation_surface_reference_validator.yml`
  - [ ] Add Japanese/English descriptions

- [ ] **Write unit tests**
  - [ ] Test valid Road with all surfaces referenced
  - [ ] Test Road with missing surface references
  - [ ] Test orphaned TrafficArea/AuxiliaryTrafficArea
  - [ ] Test multiple Roads with cross-references

- [ ] **Generate action schema**
  - [ ] Run `cargo make doc-action`
  - [ ] Verify schema generation

### Phase 2: Workflow Implementation
- [ ] **Create workflow YAML structure**
  - [ ] File: `runtime/examples/fixture/workflow/quality-check/plateau4/03-tran/workflow.yml`
  - [ ] Define input parameters
  - [ ] Add entryGraphId

- [ ] **Implement file reading section**
  - [ ] FeatureCreator
  - [ ] DirectoryDecompressor
  - [ ] FeatureFilePathExtractor
  - [ ] FeatureCounter
  - [ ] PLATEAU4.UDXFolderExtractor

- [ ] **Implement XLink validation section**
  - [ ] PLATEAU4.TransportationSurfaceReferenceValidator
  - [ ] AttributeAggregator for error counting
  - [ ] FileWriter for xlink error TSV

- [ ] **Implement CityGML reading & LOD splitting**
  - [ ] FeatureCityGmlReader
  - [ ] LodSplitter subgraph reference

- [ ] **Implement surface orientation validation**
  - [ ] OrientationExtractor
  - [ ] FeatureFilter for incorrect orientation
  - [ ] AttributeMapper for issue marking
  - [ ] (Optional) Vertical surface detector

- [ ] **Implement L-tran-01 validation**
  - [ ] TwoDimensionForcer
  - [ ] GeometryDissolver
  - [ ] GeometryCoercer to LineString
  - [ ] FeatureFilter for LOD1
  - [ ] LineOnLineOverlayer (without gmlParentId)
  - [ ] FeatureFilter for overlaps
  - [ ] AttributeMapper for error details
  - [ ] FileWriter for overlap TSV

- [ ] **Implement L-tran-02 validation**
  - [ ] FeatureFilter for LOD2/3
  - [ ] LineOnLineOverlayer (with gmlParentId)
  - [ ] FeatureFilter for overlaps
  - [ ] Merge with L-tran-01 results

- [ ] **Implement results aggregation**
  - [ ] FeatureMerger for all validation results
  - [ ] AttributeMapper for summary report
  - [ ] FileWriter for summary TSV

- [ ] **Define edges (data flow connections)**
  - [ ] Connect all nodes with proper ports
  - [ ] Ensure proper data flow from sources to sinks

### Phase 3: Testing & Validation
- [ ] **Create test data**
  - [ ] Valid PLATEAU4 transportation CityGML
  - [ ] Invalid data with various error types:
    - [ ] Missing XLink references
    - [ ] Adjacent Road overlaps (L-tran-01)
    - [ ] TrafficArea overlaps (L-tran-02)
    - [ ] Incorrect surface orientation
    - [ ] Geometry errors

- [ ] **Run workflow tests**
  - [ ] Test with valid data → expect no errors
  - [ ] Test with each error type → expect proper detection
  - [ ] Test with mixed errors → verify all detected
  - [ ] Compare output against FME workflow results

- [ ] **Performance testing**
  - [ ] Test with large datasets (1000+ features)
  - [ ] Verify memory usage is acceptable
  - [ ] Ensure processing completes within reasonable time

- [ ] **Output validation**
  - [ ] Verify TSV format matches FME template output
  - [ ] Check all required columns present
  - [ ] Verify error counts accurate
  - [ ] Ensure Japanese characters display correctly

### Phase 4: Documentation
- [ ] **Update workflow documentation**
  - [ ] Add usage instructions to README
  - [ ] Document input parameters
  - [ ] Document output file formats
  - [ ] Add example command-line invocation

- [ ] **Add code comments**
  - [ ] Document complex logic in action implementation
  - [ ] Add comments explaining L-tran-01 vs L-tran-02 difference
  - [ ] Document known limitations (vertical surfaces)

- [ ] **Create example data**
  - [ ] Provide sample input CityGML
  - [ ] Provide sample output TSV files

### Phase 5: Optional Enhancements
- [ ] **Vertical surface handler implementation**
  - [ ] Calculate surface normal vectors
  - [ ] Detect vertical surfaces (Z ≈ 0)
  - [ ] Differentiate ERROR vs WARNING for LOD3
  - [ ] Update output to show severity level

- [ ] **Grade separation overlap filter**
  - [ ] Detect 3D road intersections (overpasses/underpasses)
  - [ ] Calculate Z-distance between overlapping segments
  - [ ] Flag as WARNING if Z-distance > threshold (e.g., 3m)
  - [ ] Reduce false positives for L-tran-01

- [ ] **Action parameter enhancements**
  - [ ] Add `parentIdAttribute` parameter to LineOnLineOverlayer
  - [ ] Add `tolerance` parameter validation
  - [ ] Add LOD-specific filtering options

---

## Expected Output Files

### 1. 検査結果一覧.tsv (Inspection Results Summary)
**Columns**:
- Folder (package type)
- Index (file number)
- Filename
- フィーチャータイプ (Feature Type)
- LOD
- インスタンス数 (Instance Count)
- 面のエラー (Surface Errors)
- 面の向き不正 (Incorrect Orientation)
- 隣接面重複 (Adjacent Surface Overlap)
- xlink参照エラー (XLink Reference Errors)

### 2. xlink参照エラー.tsv (XLink Reference Errors)
**Columns**:
- Folder
- Index
- Filename
- FeatureType
- LOD
- gml:id
- 未参照 (Unreferenced Count)

### 3. 隣接面重複.tsv (Adjacent Surface Overlap)
**Columns**:
- Folder
- Index
- Filename
- FeatureType
- LOD
- gml:id
- 隣接ID (Adjacent ID)

### 4. 面の向き不正.tsv (Incorrect Orientation)
**Columns**:
- Folder
- Index
- Filename
- FeatureType
- LOD
- gml:id
- エラー数 (Error Count)
- エラー内容 (Error Content)
- 面の向き (Orientation)

### 5. 面のエラー.tsv (Surface Errors)
**Columns**:
- Folder
- Index
- Filename
- FeatureType
- LOD
- gml:id
- エラー数 (Error Count)
- エラー内容 (Error Content)

---

## Known Limitations & Design Decisions

### 1. Vertical Surface False Positives (LOD3)
**Issue**: Vertical road shoulders cannot have orientation determined reliably, causing false positive "Incorrect Orientation" errors for LOD3.

**FME Approach**: Report all errors, document limitation, require manual review.

**Our Approach Options**:
- **Option A** (Match FME): Report all, document limitation
- **Option B** (Enhancement): Detect vertical surfaces, report as WARNING not ERROR
- **Decision**: Start with Option A, implement Option B as Phase 5 enhancement

### 2. Grade Separation False Positives (L-tran-01)
**Issue**: 3D road intersections (overpasses/underpasses) trigger overlap detection despite being valid geometry.

**FME Approach**: Report all overlaps, document limitation, require manual review.

**Our Approach Options**:
- **Option A** (Match FME): Report all, document limitation
- **Option B** (Enhancement): Calculate Z-distance, filter 3D intersections
- **Decision**: Start with Option A, implement Option B as Phase 5 enhancement

### 3. Tolerance Values
**10cm tolerance** (0.1m) used for adjacency detection is defined in PLATEAU specification.
- L-tran-01: 10cm tolerance for Road instance boundaries
- L-tran-02: 10cm tolerance for TrafficArea/AuxiliaryTrafficArea boundaries

### 4. Feature Type Coverage
Supports PLATEAU transportation packages:
- `tran`: 道路 (Roads)
- `rwy`: 鉄道 (Railways)
- `trk`: 徒歩道 (Pedestrian paths)
- `squr`: 広場 (Plazas)
- `wwy`: 航路 (Waterways)

---

## References

### PLATEAU Documentation
- **Standard Product Specification v4.x**: Section 4.3 (交通（道路）モデルの応用スキーマ)
- **Quality Requirements**: Section 6.3.2 (論理一貫性)
- **L-tran-01**: Adjacent Road instance boundary consistency
- **L-tran-02**: TrafficArea/AuxiliaryTrafficArea boundary consistency within Road
- **L-tran-03**: Road surface XLink reference completeness

### FME Workflow Analysis
- **Template**: `PLATEAU4 品質検査03 道路等の交通モデル.fmwt`
- **Key Transformers**:
  - LineOnLineOverlayer (lines 30553, 30557)
  - AreaOnAreaOverlayer (line 30595)
  - OrientationExtractor (lines 30010, 30075, 30430)
  - PlanarityFilter (line 30708)
- **Vertical Surface Note**: Lines 18, 42 (documentation)
- **Grade Separation Note**: Lines 18, 42 (documentation)

### Re:Earth Flow Actions
- `LineOnLineOverlayer`: `runtime/action-processor/src/geometry/line_on_line_overlayer.rs`
- `AreaOnAreaOverlayer`: `runtime/action-processor/src/geometry/area_on_area_overlayer.rs`
- `OrientationExtractor`: `runtime/action-processor/src/geometry/orientation_extractor.rs`
- `GeometryValidator`: `runtime/action-processor/src/geometry/validator.rs`
- `PLATEAU4.UDXFolderExtractor`: `runtime/action-plateau-processor/src/plateau4/udx_folder_extractor.rs`

---

## Timeline Estimate

### Phase 1: New Action Development (3-5 days)
- Day 1-2: Implement `TransportationSurfaceReferenceValidator`
- Day 3: Write tests and documentation
- Day 4-5: Integration and debugging

### Phase 2: Workflow Implementation (5-7 days)
- Day 1-2: File reading and XLink validation section
- Day 3-4: L-tran-01 and L-tran-02 validation sections
- Day 5: Surface orientation validation
- Day 6-7: Results aggregation and output

### Phase 3: Testing & Validation (3-5 days)
- Day 1-2: Create test data
- Day 3-4: Run tests and fix issues
- Day 5: Performance testing

### Phase 4: Documentation (2-3 days)
- Day 1-2: Write documentation
- Day 3: Review and finalize

### Phase 5: Optional Enhancements (5-7 days)
- As needed based on user feedback

**Total Estimated Time**: 18-27 days

---

## Success Criteria

### Must Have (MVP)
- ✅ Workflow successfully reads PLATEAU4 transportation CityGML
- ✅ XLink reference validation detects missing references (L-tran-03)
- ✅ Adjacent Road overlap detection works for LOD1 (L-tran-01)
- ✅ TrafficArea overlap detection works for LOD2/3 (L-tran-02)
- ✅ Surface orientation validation detects incorrect orientations
- ✅ Output TSV files match FME template format
- ✅ All tests pass with sample data

### Nice to Have (Enhancements)
- ✅ Vertical surface detection reduces LOD3 false positives
- ✅ Grade separation filter reduces L-tran-01 false positives
- ✅ Performance optimized for large datasets (>10,000 features)
- ✅ Detailed error messages with spatial locations
- ✅ Visualization output (GeoJSON/KML) for errors

---

## Notes for Future Development

1. **Coordinate with Building QC**: Surface orientation logic shared with building quality checks
2. **Reusable Components**: Consider abstracting XLink validation for other feature types
3. **Performance**: Consider parallel processing for large datasets
4. **Extensibility**: Design workflow to easily add new transportation-specific checks
5. **User Feedback**: Prioritize Phase 5 enhancements based on actual user needs

---

**Document Version**: 1.0
**Last Updated**: 2025-10-20
**Author**: Implementation Plan based on FME workflow analysis
