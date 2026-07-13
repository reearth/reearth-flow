import * as Y from "yjs";

import { isActionNodeType } from "@flow/types";

import type { YWorkflow } from "../types";

// Base actions were renamed from concatenated PascalCase to space-separated
// title case (engine v0.0.430, PR #2240), e.g. "Cesium3DTilesWriter" →
// "Cesium 3D Tiles Writer". The engine resolves actions by exact name, so
// projects saved before the rename can no longer run. This map is the exact
// old → new name diff of engine/schema/actions.json across that rename.
export const LEGACY_ACTION_NAMES: Record<string, string> = {
  AppearanceRemover: "Appearance Remover",
  AreaCalculator: "Area Calculator",
  AttributeAggregator: "Attribute Aggregator",
  AttributeConversionTable: "Attribute Conversion Table",
  AttributeFlattener: "Attribute Flattener",
  AttributeManager: "Attribute Manager",
  AttributeMapper: "Attribute Mapper",
  BoundsExtractor: "Bounds Extractor",
  BulkAttributeRenamer: "Bulk Attribute Renamer",
  Cesium3DTilesWriter: "Cesium 3D Tiles Writer",
  CityGmlReader: "CityGML Reader",
  CityGmlWriter: "CityGML Writer",
  CsvReader: "CSV Reader",
  CsvWriter: "CSV Writer",
  DimensionFilter: "Dimension Filter",
  DirectoryDecompressor: "Directory Decompressor",
  EchoProcessor: "Echo Processor",
  EchoSink: "Echo Sink",
  ElevationExtractor: "Elevation Extractor",
  FeatureCityGmlReader: "Feature CityGML Reader",
  FeatureCounter: "Feature Counter",
  FeatureCreator: "Feature Creator",
  FeatureFilePathExtractor: "Feature File Path Extractor",
  FeatureFilter: "Feature Filter",
  FeatureJoiner: "Feature Joiner",
  FeatureLodFilter: "Feature LOD Filter",
  FeatureMerger: "Feature Merger",
  FeatureSorter: "Feature Sorter",
  FeatureTransformer: "Feature Transformer",
  FeatureTypeFilter: "Feature Type Filter",
  FilePathExtractor: "File Path Extractor",
  FilePropertyExtractor: "File Property Extractor",
  FootprintReplacer: "Footprint Replacer",
  GeoJsonReader: "GeoJSON Reader",
  GeoJsonWriter: "GeoJSON Writer",
  GeoPackageReader: "GeoPackage Reader",
  GeoPackageWriter: "GeoPackage Writer",
  GeometryExtractor: "Geometry Extractor",
  GeometryPartExtractor: "Geometry Part Extractor",
  GeometryRemover: "Geometry Remover",
  GeometryReplacer: "Geometry Replacer",
  GeometrySplitter: "Geometry Splitter",
  GeometryValidator: "Geometry Validator",
  GridDivider: "Grid Divider",
  HorizontalReprojector: "Horizontal Reprojector",
  ImageRasterizer: "Image Rasterizer",
  InputRouter: "Input Router",
  JsonReader: "JSON Reader",
  JsonWriter: "JSON Writer",
  ListExploder: "List Exploder",
  MVTWriter: "MVT Writer",
  NoopProcessor: "Noop Processor",
  NoopSink: "Noop Sink",
  NullAttributeMapper: "Null Attribute Mapper",
  OutputRouter: "Output Router",
  PolygonNormalExtractor: "Polygon Normal Extractor",
  RayIntersector: "Ray Intersector",
  ShapefileReader: "Shapefile Reader",
  ShapefileWriter: "Shapefile Writer",
  SpatialFilter: "Spatial Filter",
  SqlReader: "SQL Reader",
  StatisticsCalculator: "Statistics Calculator",
  ThreeDimensionForcer: "Three Dimension Forcer",
  TwoDimensionForcer: "Two Dimension Forcer",
  VerticalReprojector: "Vertical Reprojector",
  XMLFragmenter: "XML Fragmenter",
  XMLValidator: "XML Validator",
  XmlWriter: "XML Writer",
  ZipFileWriter: "Zip File Writer",
};

/**
 * Walks every workflow in the doc counting nodes whose officialName is still
 * a pre-rename action name. With apply=true it also rewrites them to the
 * current names — call inside a transaction when applying.
 *
 * Only action nodes (reader/writer/transformer) are touched: officialName is
 * stamped from the action definition there, and the engine resolves it by
 * exact name. On other node types (subworkflow, batch, note) officialName is
 * user space — a subworkflow the user happened to name "FeatureFilter" must
 * not be renamed. User customizations (customName) are never touched either.
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
