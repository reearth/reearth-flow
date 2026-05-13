export type ExpressionTemplate = {
  id: string;
  name: string;
  category:
    | "environment-access"
    | "file-operations"
    | "data-extraction"
    | "conditional-logic"
    | "array-processing"
    | "validation";
  description: string;
  tags: string[];
  rhaiCode: string;
  placeholders: { key: string; description: string; defaultValue?: string }[];
  preview?: string;
  usageExample?: string;
};

export const getExpressionTemplates = (
  t: (key: string) => string,
): ExpressionTemplate[] => [
  // File Operations
  {
    id: "create-output-path",
    name: t("Create Output File Path"),
    category: "file-operations",
    description: t("Generate output file path to write a file"),
    tags: ["file", "path", "output"],
    rhaiCode: `str(Url(env("workerArtifactPath")) / "{{fileNameWithExtension}}")`,
    placeholders: [
      {
        key: "fileNameWithExtension",
        description: t("Output filename with extension"),
        defaultValue: "example.geojson",
      },
    ],
    preview: `str(Url(env("workerArtifactPath")) / "example.geojson")`,
    usageExample: t("Create output path for processed data files"),
  },
  {
    id: "file-extract-stem",
    name: t("Extract Filename Without Extension"),
    category: "file-operations",
    description: t("Get filename without its file extension"),
    tags: ["file", "filename", "extension"],
    rhaiCode: "Url({{filePath}}).stem()",
    placeholders: [
      {
        key: "filePath",
        description: t("The file path expression"),
        defaultValue: 'value("cityGmlPath")',
      },
    ],
    preview: 'Url(value("cityGmlPath")).stem()',
    usageExample: t("Use base filename for creating related output files"),
  },
  {
    id: "file-extract-name",
    name: t("Extract Filename With Extension"),
    category: "file-operations",
    description: t("Get filename including its file extension"),
    tags: ["file", "filename"],
    rhaiCode: "Url({{filePath}}).name()",
    placeholders: [
      {
        key: "filePath",
        description: t("The file path expression"),
        defaultValue: 'value("cityGmlPath")',
      },
    ],
    preview: 'Url(value("cityGmlPath")).name()',
    usageExample: t("Get the full filename from a path"),
  },
  {
    id: "file-parent-dir",
    name: t("Get Parent Directory"),
    category: "file-operations",
    description: t("Get the directory containing a file"),
    tags: ["file", "directory", "path"],
    rhaiCode: "str(Url({{filePath}}).parent())",
    placeholders: [
      {
        key: "filePath",
        description: t("The file path expression"),
        defaultValue: 'value("cityGmlPath")',
      },
    ],
    preview: 'str(Url(value("cityGmlPath")).parent())',
    usageExample: t("Get directory of input file to write output alongside it"),
  },

  // Data Extraction
  {
    id: "gml-attribute-access",
    name: t("Access GML Attributes"),
    category: "data-extraction",
    description: t("Access a CityGML attribute by key"),
    tags: ["gml", "citygml", "attributes"],
    rhaiCode: `let attributes = value("cityGmlAttributes");
attributes["{{attributeName}}"]`,
    placeholders: [
      {
        key: "attributeName",
        description: t("CityGML attribute name"),
        defaultValue: "bldg:boundedBy",
      },
    ],
    preview: `let attributes = value("cityGmlAttributes");
attributes["bldg:boundedBy"]`,
    usageExample: t("Access CityGML building boundary information"),
  },
  {
    id: "building-id-construct",
    name: t("Construct Building ID"),
    category: "data-extraction",
    description: t("Build compound building ID from multiple parts"),
    tags: ["building", "id", "concatenation"],
    rhaiCode: `let attributes = value("cityGmlAttributes");
let ids = attributes["uro:buildingIDAttribute"];
if ids.len() == 1 {
  str(ids[0]["uro:buildingID"]) + "-" + str(ids[0]["uro:branchID"]) + "-" + str(ids[0]["uro:partID"])
} else {
  ""
}`,
    placeholders: [],
    preview: `let ids = value("cityGmlAttributes")["uro:buildingIDAttribute"];
if ids.len() == 1 { str(ids[0]["uro:buildingID"]) + "-" + str(ids[0]["uro:branchID"]) } else { "" }`,
    usageExample: t("Create unique building identifier from URO attributes"),
  },

  // Conditional Logic
  {
    id: "geometry-type-filter",
    name: t("Filter by Geometry Type"),
    category: "conditional-logic",
    description: t("Check if geometry matches specific types"),
    tags: ["geometry", "type", "filter", "gml"],
    rhaiCode: `value("{{geometryAttribute}}") in [{{allowedTypes}}]`,
    placeholders: [
      {
        key: "geometryAttribute",
        description: t("Geometry type attribute name"),
        defaultValue: "geomTag",
      },
      {
        key: "allowedTypes",
        description: t("Comma-separated allowed types"),
        defaultValue: '"gml:MultiSurface", "gml:Solid"',
      },
    ],
    preview: 'value("geomTag") in ["gml:MultiSurface", "gml:Solid"]',
    usageExample: t("Filter features to only include solid geometries"),
  },
  {
    id: "feature-type-classification",
    name: t("Classify Feature Type"),
    category: "conditional-logic",
    description: t("Classify features based on type hierarchy"),
    tags: ["feature", "type", "classification"],
    rhaiCode: `let gml_name = value("gmlName");
if gml_name == "{{primaryType}}" {
  "{{primaryLabel}}"
} else if gml_name == "{{secondaryType}}" {
  "{{secondaryLabel}}"
} else {
  "{{defaultLabel}}"
}`,
    placeholders: [
      {
        key: "primaryType",
        description: t("Primary feature type to check"),
        defaultValue: "Building",
      },
      {
        key: "primaryLabel",
        description: t("Label for primary type"),
        defaultValue: "building",
      },
      {
        key: "secondaryType",
        description: t("Secondary feature type"),
        defaultValue: "BuildingPart",
      },
      {
        key: "secondaryLabel",
        description: t("Label for secondary type"),
        defaultValue: "building_part",
      },
      {
        key: "defaultLabel",
        description: t("Default label"),
        defaultValue: "other",
      },
    ],
    preview: `if value("gmlName") == "Building" { "building" } else if value("gmlName") == "BuildingPart" { "building_part" } else { "other" }`,
    usageExample: t("Classify building features into categories"),
  },
  {
    id: "attribute-range-check",
    name: t("Check Attribute in Range"),
    category: "conditional-logic",
    description: t("Return a label based on a numeric attribute range"),
    tags: ["range", "numeric", "classify"],
    rhaiCode: `if value("{{attribute}}") >= {{high}} {
  "{{highLabel}}"
} else if value("{{attribute}}") >= {{low}} {
  "{{midLabel}}"
} else {
  "{{lowLabel}}"
}`,
    placeholders: [
      {
        key: "attribute",
        description: t("Numeric attribute name"),
        defaultValue: "lod",
      },
      { key: "high", description: t("High threshold"), defaultValue: "3" },
      { key: "highLabel", description: t("High label"), defaultValue: "high" },
      { key: "low", description: t("Low threshold"), defaultValue: "1" },
      { key: "midLabel", description: t("Mid label"), defaultValue: "medium" },
      { key: "lowLabel", description: t("Low label"), defaultValue: "low" },
    ],
    preview: `if value("lod") >= 3 { "high" } else if value("lod") >= 1 { "medium" } else { "low" }`,
    usageExample: t("Classify LOD levels into low / medium / high"),
  },

  // Array Processing
  {
    id: "array-length-check",
    name: t("Check Array Length"),
    category: "array-processing",
    description: t("Get length of an array attribute"),
    tags: ["array", "length", "count"],
    rhaiCode: 'value("{{arrayAttribute}}").len()',
    placeholders: [
      {
        key: "arrayAttribute",
        description: t("Array attribute name"),
        defaultValue: "package",
      },
    ],
    preview: 'value("package").len()',
    usageExample: t("Get the count of items in a package array"),
  },
  {
    id: "array-first-element",
    name: t("Get First Array Element"),
    category: "array-processing",
    description: t("Access the first element of an array attribute"),
    tags: ["array", "index", "first"],
    rhaiCode: 'value("{{arrayAttribute}}")[0]',
    placeholders: [
      {
        key: "arrayAttribute",
        description: t("Array attribute name"),
        defaultValue: "coordinates",
      },
    ],
    preview: 'value("coordinates")[0]',
    usageExample: t("Get first coordinate from a geometry array"),
  },
  {
    id: "array-last-element",
    name: t("Get Last Array Element"),
    category: "array-processing",
    description: t("Access the last element of an array attribute"),
    tags: ["array", "index", "last"],
    rhaiCode: 'value("{{arrayAttribute}}")[-1]',
    placeholders: [
      {
        key: "arrayAttribute",
        description: t("Array attribute name"),
        defaultValue: "coordinates",
      },
    ],
    preview: 'value("coordinates")[-1]',
    usageExample: t("Get last coordinate from a geometry array"),
  },
  {
    id: "array-slice",
    name: t("Slice Array"),
    category: "array-processing",
    description: t("Extract a sub-array using slice notation"),
    tags: ["array", "slice", "range"],
    rhaiCode: 'value("{{arrayAttribute}}")[{{start}}:{{end}}]',
    placeholders: [
      {
        key: "arrayAttribute",
        description: t("Array attribute name"),
        defaultValue: "items",
      },
      {
        key: "start",
        description: t("Start index (inclusive)"),
        defaultValue: "0",
      },
      {
        key: "end",
        description: t("End index (exclusive)"),
        defaultValue: "3",
      },
    ],
    preview: 'value("items")[0:3]',
    usageExample: t("Get the first three items from an array"),
  },

  // Validation
  {
    id: "validate-attribute-exists",
    name: t("Validate Attribute Exists"),
    category: "validation",
    description: t("Check if a required attribute exists and is non-empty"),
    tags: ["validation", "attribute", "exists"],
    rhaiCode: `let items = value("cityGmlAttributes")["{{attributeKey}}"];
items.len() > 0`,
    placeholders: [
      {
        key: "attributeKey",
        description: t("CityGML attribute key"),
        defaultValue: "uro:buildingIDAttribute",
      },
    ],
    preview: `let items = value("cityGmlAttributes")["uro:buildingIDAttribute"];
items.len() > 0`,
    usageExample: t("Check if building has required ID attribute"),
  },
  {
    id: "validate-multiple-values",
    name: t("Validate Against Allowed Values"),
    category: "validation",
    description: t("Check if value is one of several allowed values"),
    tags: ["validation", "multiple", "allowed"],
    rhaiCode: `value("{{attributeName}}") in [{{allowedValues}}]`,
    placeholders: [
      {
        key: "attributeName",
        description: t("Attribute to validate"),
        defaultValue: "featureType",
      },
      {
        key: "allowedValues",
        description: t("Comma-separated allowed values"),
        defaultValue: '"Building", "BuildingPart", "BuildingInstallation"',
      },
    ],
    preview: `value("featureType") in ["Building", "BuildingPart", "BuildingInstallation"]`,
    usageExample: t(
      "Validate that feature type is one of allowed building types",
    ),
  },
  {
    id: "validate-string-non-empty",
    name: t("Check Non-Empty String"),
    category: "validation",
    description: t("Verify a string attribute has content"),
    tags: ["validation", "string", "empty"],
    rhaiCode: `value("{{attributeName}}").len() > 0`,
    placeholders: [
      {
        key: "attributeName",
        description: t("String attribute to check"),
        defaultValue: "name",
      },
    ],
    preview: `value("name").len() > 0`,
    usageExample: t("Check that name field is not blank"),
  },
];

export const getTemplateCategories = (t: (key: string) => string) =>
  ({
    "file-operations": {
      name: t("File Operations"),
      description: t("Build and manipulate file paths using the Url type"),
      icon: "📁",
    },
    "data-extraction": {
      name: t("Data Extraction"),
      description: t(
        "Extract data from feature attributes and CityGML structures",
      ),
      icon: "🔍",
    },
    "conditional-logic": {
      name: t("Conditional Logic"),
      description: t("Feature classification and filtering patterns"),
      icon: "🔀",
    },
    "array-processing": {
      name: t("Array Processing"),
      description: t("Index, slice, and measure arrays"),
      icon: "📊",
    },
    validation: {
      name: t("Data Validation"),
      description: t("Quality checks and attribute validation patterns"),
      icon: "✅",
    },
  }) as const;

export function getTemplatesByCategory(
  category: keyof ReturnType<typeof getTemplateCategories>,
  t: (key: string) => string,
) {
  return getExpressionTemplates(t).filter(
    (template) => template.category === category,
  );
}

export function searchTemplates(query: string, t: (key: string) => string) {
  const lowerQuery = query.toLowerCase();
  return getExpressionTemplates(t).filter(
    (template) =>
      template.name.toLowerCase().includes(lowerQuery) ||
      template.description.toLowerCase().includes(lowerQuery) ||
      template.tags.some((tag) => tag.toLowerCase().includes(lowerQuery)) ||
      template.rhaiCode.toLowerCase().includes(lowerQuery),
  );
}
