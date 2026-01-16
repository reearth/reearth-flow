# Quality Check 10 - Other Structures

This workflow performs quality checks on generic city objects and other structures in 3D city models. It validates geometric consistency, surface integrity, solid boundaries, and gml:name attributes according to PLATEAU specifications.

## Purpose

The workflow checks for the following quality issues in 3D city models:
- Surface geometry errors (intersections, self-intersections, duplicate points)
- Incorrect surface orientations
- Non-waterproof (non-manifold) solids
- Solid boundary validation issues
- Missing or improperly coded gml:name attributes

## Subgraphs (components)

### 1. Main Workflow (`main.yml`)
The main workflow orchestrates the entire quality checking process. It reads CityGML data, routes features by LOD (Level of Detail), and coordinates the various validation subgraphs.

### 2. LOD Splitter (`lod-splitter.yml`)
Splits incoming features by Level of Detail (LOD) and routes them to appropriate validation modules based on their LOD values (LOD0, LOD1, LOD2, LOD3, etc.).

### 3. Surface Validator 2D (`surface-validator-2d.yml`)
Validates 2D surfaces (typically LOD0) for:
- Self-intersection issues
- Ring intersection problems
- Duplicate points
- Geometric validity

### 4. Surface Validator 3D (`surface-validator-3d.yml`)
Validates 3D surfaces (typically LOD2/LOD3) for:
- Angular tolerance issues
- Ring intersection problems
- Self-intersection detection
- Thickness tolerance validation
- Orientation correctness

### 5. Solid Boundary Validator (`solid-boundary-validator.yml`)
Validates solid geometry boundaries for:
- Closed solid validation
- Boundary consistency
- Topological correctness
- Water tightness verification

### 6. GML Name Validation (`gml-name-validation.yml`)
Checks gml:name attributes for:
- Missing codeSpace values
- Improperly coded gml:name elements
- Compliance with naming conventions

### 7. Result Aggregation (`result-aggregation.yml`)
Combines all validation results and generates:
- Summary reports with error counts
- Detailed error reports by file and LOD
- Excel output with validation results
- Shapefile output for spatial visualization
- JSON summary files

## Data Flow

1. CityGML data is read and routed by LOD
2. LOD0 geometries go through 2D surface validation
3. LOD2/LOD3 geometries go through 3D surface validation
4. Solid geometries undergo boundary validation
5. GML name validation occurs separately
6. All results are aggregated and reported

## Output

The workflow produces several outputs:
- Excel reports with detailed error counts by file, LOD, and feature type
- Shapefiles with error locations for spatial analysis
- CSV summary files
- JSON result files with aggregated error counts