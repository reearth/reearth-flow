import { Segregated } from "@flow/types";

const byCategory: Segregated = {
  Attribute: [
    {
      name: "AttributeAggregator",
      description: "Aggregates features by attributes",
      type: "processor",
      categories: ["Attribute"],
    },
    {
      name: "AttributeDuplicateFilter",
      description: "Filters features by duplicate attributes",
      type: "processor",
      categories: ["Attribute"],
    },
    {
      name: "AttributeFilePathInfoExtractor",
      description: "Extracts file path information from attributes",
      type: "processor",
      categories: ["Attribute"],
    },
    {
      name: "AttributeKeeper",
      description: "Keeps only specified attributes",
      type: "processor",
      categories: ["Attribute"],
    },
    {
      name: "AttributeManager",
      description: "Manages attributes",
      type: "processor",
      categories: ["Attribute"],
    },
    {
      name: "StatisticsCalculator",
      description: "Calculates statistics of features",
      type: "processor",
      categories: ["Attribute"],
    },
  ],
  Debug: [
    {
      name: "Echo",
      description: "Echo features",
      type: "sink",
      categories: ["Debug"],
    },
  ],
  Feature: [
    {
      name: "FeatureCounter",
      description: "Counts features",
      type: "processor",
      categories: ["Feature"],
    },
    {
      name: "FeatureFilter",
      description: "Filters features based on conditions",
      type: "processor",
      categories: ["Feature"],
    },
    {
      name: "FeatureMerger",
      description: "Merges features by attributes",
      type: "processor",
      categories: ["Feature"],
    },
    {
      name: "FeatureReader",
      description: "Filters features based on conditions",
      type: "processor",
      categories: ["Feature"],
    },
    {
      name: "FeatureSorter",
      description: "Sorts features by attributes",
      type: "processor",
      categories: ["Feature"],
    },
    {
      name: "FeatureTransformer",
      description: "Transforms features by expressions",
      type: "processor",
      categories: ["Feature"],
    },
    {
      name: "RhaiCaller",
      description: "Calls Rhai script",
      type: "processor",
      categories: ["Feature"],
    },
  ],
  File: [
    {
      name: "FilePathExtractor",
      description: "Extracts files from a directory or an archive",
      type: "source",
      categories: ["File"],
    },
    {
      name: "FileReader",
      description: "Reads features from a file",
      type: "source",
      categories: ["File"],
    },
    {
      name: "FileWriter",
      description: "Writes features to a file",
      type: "sink",
      categories: ["File"],
    },
  ],
  Geometry: [
    {
      name: "AreaOnAreaOverlayer",
      description: "Overlays an area on another area",
      type: "processor",
      categories: ["Geometry"],
    },
    {
      name: "BoundsExtractor",
      description: "Bounds Extractor",
      type: "processor",
      categories: ["Geometry"],
    },
    {
      name: "Bufferer",
      description: "Buffers a geometry",
      type: "processor",
      categories: ["Geometry"],
    },
    {
      name: "CenterPointReplacer",
      description:
        "Replaces the geometry of the feature with a point that is either in the center of the feature's bounding box, at the center of mass of the feature, or somewhere guaranteed to be inside the feature's area.",
      type: "processor",
      categories: ["Geometry"],
    },
    {
      name: "Clipper",
      description:
        "Divides Candidate features using Clipper features, so that Candidates and parts of Candidates that are inside or outside of the Clipper features are output separately",
      type: "processor",
      categories: ["Geometry"],
    },
    {
      name: "ClosedCurveFilter",
      description: "Checks if curves form closed loops",
      type: "processor",
      categories: ["Geometry"],
    },
    {
      name: "CoordinateSystemSetter",
      description: "Sets the coordinate system of a feature",
      type: "processor",
      categories: ["Geometry"],
    },
    {
      name: "Extruder",
      description: "Extrudes a polygon by a distance",
      type: "processor",
      categories: ["Geometry"],
    },
    {
      name: "GeometryCoercer",
      description: "Coerces the geometry of a feature to a specific geometry",
      type: "processor",
      categories: ["Geometry"],
    },
    {
      name: "GeometryExtractor",
      description: "Extracts geometry from a feature and adds it as an attribute.",
      type: "processor",
      categories: ["Geometry"],
    },
    {
      name: "GeometryFilter",
      description: "Filter geometry by type",
      type: "processor",
      categories: ["Geometry"],
    },
    {
      name: "GeometryReplacer",
      description: "Replaces the geometry of a feature with a new geometry.",
      type: "processor",
      categories: ["Geometry"],
    },
    {
      name: "GeometrySplitter",
      description: "Split geometry by type",
      type: "processor",
      categories: ["Geometry"],
    },
    {
      name: "GeometryValidator",
      description: "Validates the geometry of a feature",
      type: "processor",
      categories: ["Geometry"],
    },
    {
      name: "HoleCounter",
      description: "Counts the number of holes in a geometry and adds it as an attribute.",
      type: "processor",
      categories: ["Geometry"],
    },
    {
      name: "HoleExtractor",
      description: "Extracts holes in a geometry and adds it as an attribute.",
      type: "processor",
      categories: ["Geometry"],
    },
    {
      name: "LineOnLineOverlayer",
      description:
        "Intersection points are turned into point features that can contain the merged list of attributes of the original intersected lines.",
      type: "processor",
      categories: ["Geometry"],
    },
    {
      name: "OrientationExtractor",
      description:
        "Extracts the orientation of a geometry from a feature and adds it as an attribute.",
      type: "processor",
      categories: ["Geometry"],
    },
    {
      name: "PlanarityFilter",
      description: "Filter geometry by type",
      type: "processor",
      categories: ["Geometry"],
    },
    {
      name: "Refiner",
      description: "Geometry Refiner",
      type: "processor",
      categories: ["Geometry"],
    },
    {
      name: "Reprojector",
      description: "Reprojects the geometry of a feature to a specified coordinate system",
      type: "processor",
      categories: ["Geometry"],
    },
    {
      name: "ThreeDimentionBoxReplacer",
      description: "Replaces a three dimention box with a polygon.",
      type: "processor",
      categories: ["Geometry"],
    },
    {
      name: "ThreeDimentionRotator",
      description: "Replaces a three dimention box with a polygon.",
      type: "processor",
      categories: ["Geometry"],
    },
    {
      name: "TwoDimentionForcer",
      description: "Forces a geometry to be two dimentional.",
      type: "processor",
      categories: ["Geometry"],
    },
    {
      name: "VertexRemover",
      description: "Removes specific vertices from a feature’s geometry",
      type: "processor",
      categories: ["Geometry"],
    },
  ],
  PLATEAU: [
    {
      name: "PLATEAU.AttributeFlattener",
      description: "AttributeFlattener",
      type: "processor",
      categories: ["PLATEAU"],
    },
    {
      name: "PLATEAU.BuildingInstallationGeometryTypeExtractor",
      description: "Extracts BuildingInstallationGeometryType",
      type: "processor",
      categories: ["PLATEAU"],
    },
    {
      name: "PLATEAU.DictionariesInitiator",
      description: "Initializes dictionaries for PLATEAU",
      type: "processor",
      categories: ["PLATEAU"],
    },
    {
      name: "PLATEAU.DomainOfDefinitionValidator",
      description: "Validates domain of definition of CityGML features",
      type: "processor",
      categories: ["PLATEAU"],
    },
    {
      name: "PLATEAU.MaxLodExtractor",
      description: "Extracts maxLod",
      type: "processor",
      categories: ["PLATEAU"],
    },
    {
      name: "PLATEAU.UDXFolderExtractor",
      description: "Extracts UDX folders from cityGML path",
      type: "processor",
      categories: ["PLATEAU"],
    },
    {
      name: "PLATEAU.UnmatchedXlinkDetector",
      description: "Detect unmatched xlink for PLATEAU",
      type: "processor",
      categories: ["PLATEAU"],
    },
    {
      name: "PLATEAU.XMLAttributeExtractor",
      description: "Extracts attributes from XML fragments based on a schema definition",
      type: "processor",
      categories: ["PLATEAU"],
    },
    {
      name: "XMLValidator",
      description: "Validates XML content",
      type: "processor",
      categories: ["PLATEAU"],
    },
  ],
  Uncategorized: [
    {
      name: "Router",
      description: "Action for last port forwarding for sub-workflows.",
      type: "processor",
      categories: [],
    },
  ],
  XML: [
    {
      name: "XMLFragmenter",
      description: "Fragment XML",
      type: "processor",
      categories: ["XML"],
    },
  ],
};

const byType: Segregated = {
  processor: [
    {
      name: "AreaOnAreaOverlayer",
      description: "Overlays an area on another area",
      type: "processor",
      categories: ["Geometry"],
    },
    {
      name: "AttributeAggregator",
      description: "Aggregates features by attributes",
      type: "processor",
      categories: ["Attribute"],
    },
    {
      name: "AttributeDuplicateFilter",
      description: "Filters features by duplicate attributes",
      type: "processor",
      categories: ["Attribute"],
    },
    {
      name: "AttributeFilePathInfoExtractor",
      description: "Extracts file path information from attributes",
      type: "processor",
      categories: ["Attribute"],
    },
    {
      name: "AttributeKeeper",
      description: "Keeps only specified attributes",
      type: "processor",
      categories: ["Attribute"],
    },
    {
      name: "AttributeManager",
      description: "Manages attributes",
      type: "processor",
      categories: ["Attribute"],
    },
    {
      name: "BoundsExtractor",
      description: "Bounds Extractor",
      type: "processor",
      categories: ["Geometry"],
    },
    {
      name: "Bufferer",
      description: "Buffers a geometry",
      type: "processor",
      categories: ["Geometry"],
    },
    {
      name: "CenterPointReplacer",
      description:
        "Replaces the geometry of the feature with a point that is either in the center of the feature's bounding box, at the center of mass of the feature, or somewhere guaranteed to be inside the feature's area.",
      type: "processor",
      categories: ["Geometry"],
    },
    {
      name: "Clipper",
      description:
        "Divides Candidate features using Clipper features, so that Candidates and parts of Candidates that are inside or outside of the Clipper features are output separately",
      type: "processor",
      categories: ["Geometry"],
    },
    {
      name: "ClosedCurveFilter",
      description: "Checks if curves form closed loops",
      type: "processor",
      categories: ["Geometry"],
    },
    {
      name: "CoordinateSystemSetter",
      description: "Sets the coordinate system of a feature",
      type: "processor",
      categories: ["Geometry"],
    },
    {
      name: "Extruder",
      description: "Extrudes a polygon by a distance",
      type: "processor",
      categories: ["Geometry"],
    },
    {
      name: "FeatureCounter",
      description: "Counts features",
      type: "processor",
      categories: ["Feature"],
    },
    {
      name: "FeatureFilter",
      description: "Filters features based on conditions",
      type: "processor",
      categories: ["Feature"],
    },
    {
      name: "FeatureMerger",
      description: "Merges features by attributes",
      type: "processor",
      categories: ["Feature"],
    },
    {
      name: "FeatureReader",
      description: "Filters features based on conditions",
      type: "processor",
      categories: ["Feature"],
    },
    {
      name: "FeatureSorter",
      description: "Sorts features by attributes",
      type: "processor",
      categories: ["Feature"],
    },
    {
      name: "FeatureTransformer",
      description: "Transforms features by expressions",
      type: "processor",
      categories: ["Feature"],
    },
    {
      name: "GeometryCoercer",
      description: "Coerces the geometry of a feature to a specific geometry",
      type: "processor",
      categories: ["Geometry"],
    },
    {
      name: "GeometryExtractor",
      description: "Extracts geometry from a feature and adds it as an attribute.",
      type: "processor",
      categories: ["Geometry"],
    },
    {
      name: "GeometryFilter",
      description: "Filter geometry by type",
      type: "processor",
      categories: ["Geometry"],
    },
    {
      name: "GeometryReplacer",
      description: "Replaces the geometry of a feature with a new geometry.",
      type: "processor",
      categories: ["Geometry"],
    },
    {
      name: "GeometrySplitter",
      description: "Split geometry by type",
      type: "processor",
      categories: ["Geometry"],
    },
    {
      name: "GeometryValidator",
      description: "Validates the geometry of a feature",
      type: "processor",
      categories: ["Geometry"],
    },
    {
      name: "HoleCounter",
      description: "Counts the number of holes in a geometry and adds it as an attribute.",
      type: "processor",
      categories: ["Geometry"],
    },
    {
      name: "HoleExtractor",
      description: "Extracts holes in a geometry and adds it as an attribute.",
      type: "processor",
      categories: ["Geometry"],
    },
    {
      name: "LineOnLineOverlayer",
      description:
        "Intersection points are turned into point features that can contain the merged list of attributes of the original intersected lines.",
      type: "processor",
      categories: ["Geometry"],
    },
    {
      name: "OrientationExtractor",
      description:
        "Extracts the orientation of a geometry from a feature and adds it as an attribute.",
      type: "processor",
      categories: ["Geometry"],
    },
    {
      name: "PLATEAU.AttributeFlattener",
      description: "AttributeFlattener",
      type: "processor",
      categories: ["PLATEAU"],
    },
    {
      name: "PLATEAU.BuildingInstallationGeometryTypeExtractor",
      description: "Extracts BuildingInstallationGeometryType",
      type: "processor",
      categories: ["PLATEAU"],
    },
    {
      name: "PLATEAU.DictionariesInitiator",
      description: "Initializes dictionaries for PLATEAU",
      type: "processor",
      categories: ["PLATEAU"],
    },
    {
      name: "PLATEAU.DomainOfDefinitionValidator",
      description: "Validates domain of definition of CityGML features",
      type: "processor",
      categories: ["PLATEAU"],
    },
    {
      name: "PLATEAU.MaxLodExtractor",
      description: "Extracts maxLod",
      type: "processor",
      categories: ["PLATEAU"],
    },
    {
      name: "PLATEAU.UDXFolderExtractor",
      description: "Extracts UDX folders from cityGML path",
      type: "processor",
      categories: ["PLATEAU"],
    },
    {
      name: "PLATEAU.UnmatchedXlinkDetector",
      description: "Detect unmatched xlink for PLATEAU",
      type: "processor",
      categories: ["PLATEAU"],
    },
    {
      name: "PLATEAU.XMLAttributeExtractor",
      description: "Extracts attributes from XML fragments based on a schema definition",
      type: "processor",
      categories: ["PLATEAU"],
    },
    {
      name: "PlanarityFilter",
      description: "Filter geometry by type",
      type: "processor",
      categories: ["Geometry"],
    },
    {
      name: "Refiner",
      description: "Geometry Refiner",
      type: "processor",
      categories: ["Geometry"],
    },
    {
      name: "Reprojector",
      description: "Reprojects the geometry of a feature to a specified coordinate system",
      type: "processor",
      categories: ["Geometry"],
    },
    {
      name: "RhaiCaller",
      description: "Calls Rhai script",
      type: "processor",
      categories: ["Feature"],
    },
    {
      name: "Router",
      description: "Action for last port forwarding for sub-workflows.",
      type: "processor",
      categories: [],
    },
    {
      name: "StatisticsCalculator",
      description: "Calculates statistics of features",
      type: "processor",
      categories: ["Attribute"],
    },
    {
      name: "ThreeDimentionBoxReplacer",
      description: "Replaces a three dimention box with a polygon.",
      type: "processor",
      categories: ["Geometry"],
    },
    {
      name: "ThreeDimentionRotator",
      description: "Replaces a three dimention box with a polygon.",
      type: "processor",
      categories: ["Geometry"],
    },
    {
      name: "TwoDimentionForcer",
      description: "Forces a geometry to be two dimentional.",
      type: "processor",
      categories: ["Geometry"],
    },
    {
      name: "VertexRemover",
      description: "Removes specific vertices from a feature’s geometry",
      type: "processor",
      categories: ["Geometry"],
    },
    {
      name: "XMLFragmenter",
      description: "Fragment XML",
      type: "processor",
      categories: ["XML"],
    },
    {
      name: "XMLValidator",
      description: "Validates XML content",
      type: "processor",
      categories: ["PLATEAU"],
    },
  ],
  sink: [
    {
      name: "Echo",
      description: "Echo features",
      type: "sink",
      categories: ["Debug"],
    },
    {
      name: "FileWriter",
      description: "Writes features to a file",
      type: "sink",
      categories: ["File"],
    },
  ],
  source: [
    {
      name: "FilePathExtractor",
      description: "Extracts files from a directory or an archive",
      type: "source",
      categories: ["File"],
    },
    {
      name: "FileReader",
      description: "Reads features from a file",
      type: "source",
      categories: ["File"],
    },
  ],
};

export default {
  byCategory,
  byType,
};
