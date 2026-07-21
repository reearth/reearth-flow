import * as Y from "yjs";

import { isActionNodeType } from "@flow/types";

import type { YWorkflow } from "../types";

// Base actions were renamed from PascalCase to spaced title case (engine PR #2240),
// e.g. "Cesium3DTilesWriter" → "Cesium 3D Tiles Writer". The engine resolves
// actions by exact name, so projects saved before the rename can no longer run.
// The remaining hidden (non-PLATEAU) actions were renamed the same way in engine
// PR #2272; PLATEAU3/4/6-prefixed actions were intentionally left untouched there
// and don't need an entry here yet.
export const LEGACY_ACTION_NAMES: Record<string, string> = {
  AppearanceRemover: "Appearance Remover",
  AreaCalculator: "Area Calculator",
  AreaOnAreaOverlayer: "Area On Area Overlayer",
  AttributeAggregator: "Attribute Aggregator",
  AttributeBulkArrayJoiner: "Attribute Bulk Array Joiner",
  AttributeConversionTable: "Attribute Conversion Table",
  AttributeDuplicateFilter: "Attribute Duplicate Filter",
  AttributeFilePathInfoExtractor: "Attribute File Path Info Extractor",
  AttributeFlattener: "Attribute Flattener",
  AttributeManager: "Attribute Manager",
  AttributeMapper: "Attribute Mapper",
  AttributeRangeMapper: "Attribute Range Mapper",
  BoundaryExtractor: "Boundary Extractor",
  BoundsExtractor: "Bounds Extractor",
  BulkAttributeRenamer: "Bulk Attribute Renamer",
  CSGBuilder: "CSG Builder",
  CSGEvaluator: "CSG Evaluator",
  CenterPointReplacer: "Center Point Replacer",
  Cesium3DTilesWriter: "Cesium 3D Tiles Writer",
  CityGmlReader: "CityGML Reader",
  CityGmlWriter: "CityGML Writer",
  ClosedCurveFilter: "Closed Curve Filter",
  ConvexHullAccumulator: "Convex Hull Accumulator",
  CoordinateExtractor: "Coordinate Extractor",
  CsvReader: "CSV Reader",
  CsvWriter: "CSV Writer",
  CzmlReader: "CZML Reader",
  CzmlWriter: "CZML Writer",
  DateTimeConverter: "Date Time Converter",
  DimensionFilter: "Dimension Filter",
  DirectoryDecompressor: "Directory Decompressor",
  EchoProcessor: "Echo Processor",
  EchoSink: "Echo Sink",
  ElevationExtractor: "Elevation Extractor",
  ExcelWriter: "Excel Writer",
  FeatureCityGml2Reader: "Feature CityGML 2 Reader",
  FeatureCityGml3Reader: "Feature CityGML 3 Reader",
  FeatureCityGmlReader: "Feature CityGML Reader",
  FeatureCounter: "Feature Counter",
  FeatureCreator: "Feature Creator",
  FeatureDuplicateFilter: "Feature Duplicate Filter",
  FeatureFilePathExtractor: "Feature File Path Extractor",
  FeatureFilter: "Feature Filter",
  FeatureJoiner: "Feature Joiner",
  FeatureLodFilter: "Feature LOD Filter",
  FeatureMerger: "Feature Merger",
  FeatureReader: "Feature Reader",
  FeatureSorter: "Feature Sorter",
  FeatureTransformer: "Feature Transformer",
  FeatureTypeFilter: "Feature Type Filter",
  FeatureWriter: "Feature Writer",
  FilePathExtractor: "File Path Extractor",
  FilePropertyExtractor: "File Property Extractor",
  FootprintReplacer: "Footprint Replacer",
  GeoJsonReader: "GeoJSON Reader",
  GeoJsonWriter: "GeoJSON Writer",
  GeoPackageReader: "GeoPackage Reader",
  GeoPackageWriter: "GeoPackage Writer",
  GeometryCoercer: "Geometry Coercer",
  GeometryExtractor: "Geometry Extractor",
  GeometryFilter: "Geometry Filter",
  GeometryPartExtractor: "Geometry Part Extractor",
  GeometryRemover: "Geometry Remover",
  GeometryReplacer: "Geometry Replacer",
  GeometrySplitter: "Geometry Splitter",
  GeometryValidator: "Geometry Validator",
  GeometryValueFilter: "Geometry Value Filter",
  GltfReader: "glTF Reader",
  GltfWriter: "glTF Writer",
  GridDivider: "Grid Divider",
  HTTPCaller: "HTTP Caller",
  HoleCounter: "Hole Counter",
  HoleExtractor: "Hole Extractor",
  HorizontalReprojector: "Horizontal Reprojector",
  ImageRasterizer: "Image Rasterizer",
  InputRouter: "Input Router",
  JPStandardGridAccumulator: "JP Standard Grid Accumulator",
  JSONFragmenter: "JSON Fragmenter",
  JsonReader: "JSON Reader",
  JsonWriter: "JSON Writer",
  LineOnLineOverlayer: "Line On Line Overlayer",
  ListConcatenator: "List Concatenator",
  ListExploder: "List Exploder",
  ListIndexer: "List Indexer",
  MVTWriter: "MVT Writer",
  NeighborFinder: "Neighbor Finder",
  NoopProcessor: "Noop Processor",
  NoopSink: "Noop Sink",
  NullAttributeMapper: "Null Attribute Mapper",
  ObjReader: "OBJ Reader",
  ObjWriter: "OBJ Writer",
  OrientationExtractor: "Orientation Extractor",
  OutputRouter: "Output Router",
  PlanarityFilter: "Planarity Filter",
  PolygonNormalExtractor: "Polygon Normal Extractor",
  PythonScriptProcessor: "Python Script Processor",
  RayIntersector: "Ray Intersector",
  Rotator3D: "Rotator 3D",
  ShapefileReader: "Shapefile Reader",
  ShapefileWriter: "Shapefile Writer",
  SolidBoundaryValidator: "Solid Boundary Validator",
  SpatialFilter: "Spatial Filter",
  SqlReader: "SQL Reader",
  StatisticsCalculator: "Statistics Calculator",
  ThreeDimensionBoxReplacer: "Three Dimension Box Replacer",
  ThreeDimensionForcer: "Three Dimension Forcer",
  ThreeDimensionPlanarityRotator: "Three Dimension Planarity Rotator",
  ThreeDimensionRotator: "Three Dimension Rotator",
  TwoDimensionForcer: "Two Dimension Forcer",
  VertexCounter: "Vertex Counter",
  VertexRemover: "Vertex Remover",
  VerticalReprojector: "Vertical Reprojector",
  XMLFragmenter: "XML Fragmenter",
  XMLValidator: "XML Validator",
  XmlWriter: "XML Writer",
  ZipFileWriter: "Zip File Writer",
};

/**
 * Counts nodes whose officialName is still a pre-rename action name; with
 * apply=true also rewrites them — call inside a transaction when applying.
 *
 * Only action nodes (reader/writer/transformer) are touched: on other node
 * types officialName is user space (e.g. a subworkflow named "FeatureFilter").
 */
export function scanLegacyActionNames(
  yWorkflows: Y.Map<YWorkflow>,
  apply: boolean,
): number {
  let count = 0;

  yWorkflows.forEach((yWorkflow) => {
    const yNodes = yWorkflow.get("nodes");
    if (!(yNodes instanceof Y.Map)) return;

    yNodes.forEach((yNode) => {
      const type = String((yNode as Y.Map<unknown>).get("type"));
      if (!isActionNodeType(type)) return;

      const yData = (yNode as Y.Map<unknown>).get("data");
      if (!(yData instanceof Y.Map)) return;

      const currentName =
        LEGACY_ACTION_NAMES[String(yData.get("officialName"))];
      if (!currentName) return;

      count++;
      if (apply) yData.set("officialName", new Y.Text(currentName));
    });
  });

  return count;
}

export const hasLegacyActionNames = (yWorkflows: Y.Map<YWorkflow>): boolean =>
  scanLegacyActionNames(yWorkflows, false) > 0;

export const migrateLegacyActionNames = (
  yWorkflows: Y.Map<YWorkflow>,
): number => scanLegacyActionNames(yWorkflows, true);
