export type ExpressionTemplate = {
  id: string;
  name: string;
  category: 'environment-access' | 'file-operations' | 'data-extraction' | 'conditional-logic' | 'array-processing' | 'validation';
  description: string;
  tags: string[];
  rhaiCode: string;
  placeholders: { key: string; description: string; defaultValue?: string }[];
  preview?: string;
  usageExample?: string;
};

export const EXPRESSION_TEMPLATES: ExpressionTemplate[] = [
  // Advanced File Operations (beyond Simple Builder capabilities)
  {
    id: 'file-extract-without-extension',
    name: 'Extract Filename Without Extension',
    category: 'file-operations',
    description: 'Get filename without its file extension',
    tags: ['file', 'filename', 'extension'],
    rhaiCode: 'file::extract_filename_without_ext({{filePath}})',
    placeholders: [
      { key: 'filePath', description: 'The file path expression', defaultValue: 'env.get("__value").cityGmlPath' }
    ],
    preview: 'file::extract_filename_without_ext(env.get("__value").cityGmlPath)',
    usageExample: 'Use base filename for creating related output files'
  },

  // Data Extraction - Complex CityGML/GML patterns
  {
    id: 'gml-attribute-access',
    name: 'Access GML Attributes Safely',
    category: 'data-extraction',
    description: 'Safely access CityGML attributes with null checking',
    tags: ['gml', 'citygml', 'attributes', 'null-safe'],
    rhaiCode: `let attributes = env.get("__value").cityGmlAttributes ?? #{};
attributes["{{attributeName}}"] ?? {{defaultValue}}`,
    placeholders: [
      { key: 'attributeName', description: 'CityGML attribute name', defaultValue: 'bldg:boundedBy' },
      { key: 'defaultValue', description: 'Default value if attribute missing', defaultValue: '[]' }
    ],
    preview: `let attributes = env.get("__value").cityGmlAttributes ?? #{};
attributes["bldg:boundedBy"] ?? []`,
    usageExample: 'Safely access CityGML building boundary information'
  },
  {
    id: 'regex-extract-mesh-code',
    name: 'Extract Mesh Code from Filename',
    category: 'data-extraction',
    description: 'Extract mesh code from CityGML filename using regex',
    tags: ['regex', 'mesh', 'code', 'filename'],
    rhaiCode: 'str::extract_single_by_regex("^(.+?)_.+$", file::extract_filename({{filePath}}))',
    placeholders: [
      { key: 'filePath', description: 'File path to extract from', defaultValue: 'env.get("__value").cityGmlPath' }
    ],
    preview: 'str::extract_single_by_regex("^(.+?)_.+$", file::extract_filename(env.get("__value").cityGmlPath))',
    usageExample: 'Get mesh code from PLATEAU CityGML filename format'
  },
  {
    id: 'building-id-construct',
    name: 'Construct Building ID',
    category: 'data-extraction',
    description: 'Build compound building ID from multiple parts',
    tags: ['building', 'id', 'concatenation'],
    rhaiCode: `let attributes = env.get("__value").cityGmlAttributes ?? #{};
let building_id_attribute = attributes["uro:buildingIDAttribute"] ?? [];
if building_id_attribute.len == 1 {
  building_id_attribute[0]["uro:buildingID"] ?? "" + "-" + building_id_attribute[0]["uro:branchID"] ?? "" + "-" + building_id_attribute[0]["uro:partID"] ?? ""
} else {
  ""
}`,
    placeholders: [],
    preview: 'building_id_attribute[0]["uro:buildingID"] + "-" + building_id_attribute[0]["uro:branchID"] + "-" + building_id_attribute[0]["uro:partID"]',
    usageExample: 'Create unique building identifier from URO attributes'
  },

  {
    id: 'geometry-type-filter',
    name: 'Filter by Geometry Type',
    category: 'conditional-logic',
    description: 'Check if geometry matches specific types',
    tags: ['geometry', 'type', 'filter', 'gml'],
    rhaiCode: 'env.get("__value").{{geometryAttribute}} in [{{allowedTypes}}]',
    placeholders: [
      { key: 'geometryAttribute', description: 'Geometry type attribute name', defaultValue: 'geomTag' },
      { key: 'allowedTypes', description: 'Comma-separated allowed types', defaultValue: '"gml:MultiSurface", "gml:Solid"' }
    ],
    preview: 'env.get("__value").geomTag in ["gml:MultiSurface", "gml:Solid"]',
    usageExample: 'Filter features to only include solid geometries'
  },
  {
    id: 'feature-type-classification',
    name: 'Classify Feature Type',
    category: 'conditional-logic',
    description: 'Classify features based on type hierarchy',
    tags: ['feature', 'type', 'classification'],
    rhaiCode: `let gml_name = env.get("__value").gmlName ?? "";
if gml_name == "{{primaryType}}" {
  "{{primaryLabel}}"
} else if gml_name == "{{secondaryType}}" {
  "{{secondaryLabel}}"
} else {
  "{{defaultLabel}}"
}`,
    placeholders: [
      { key: 'primaryType', description: 'Primary feature type to check', defaultValue: 'Building' },
      { key: 'primaryLabel', description: 'Label for primary type', defaultValue: 'building' },
      { key: 'secondaryType', description: 'Secondary feature type', defaultValue: 'BuildingPart' },
      { key: 'secondaryLabel', description: 'Label for secondary type', defaultValue: 'building_part' },
      { key: 'defaultLabel', description: 'Default label', defaultValue: 'other' }
    ],
    preview: `if gml_name == "Building" { "building" } else if gml_name == "BuildingPart" { "building_part" } else { "other" }`,
    usageExample: 'Classify building features into categories'
  },

  // Array Processing - Collection operations
  {
    id: 'join-array-comma',
    name: 'Join Array with Commas',
    category: 'array-processing',
    description: 'Join array elements into comma-separated string',
    tags: ['array', 'join', 'comma', 'collection'],
    rhaiCode: 'collection::join_array((env.get("__value").{{arrayAttribute}} ?? []), ",")',
    placeholders: [
      { key: 'arrayAttribute', description: 'Array attribute name', defaultValue: 'relatedXMLTags' }
    ],
    preview: 'collection::join_array((env.get("__value").relatedXMLTags ?? []), ",")',
    usageExample: 'Convert array of XML tags into comma-separated string for output'
  },
  {
    id: 'count-by-type',
    name: 'Count Items by Type',
    category: 'array-processing',
    description: 'Count array items that match a specific type',
    tags: ['array', 'count', 'filter', 'reduce'],
    rhaiCode: `let {{arrayName}} = {{arrayExpression}} ?? [];
{{arrayName}}.reduce(|sum| { if this["type"] == "{{targetType}}" { sum + 1 } else { sum } }, 0)`,
    placeholders: [
      { key: 'arrayName', description: 'Variable name for array', defaultValue: 'bounded_by' },
      { key: 'arrayExpression', description: 'Expression that returns array', defaultValue: 'attributes["bldg:boundedBy"]' },
      { key: 'targetType', description: 'Type to count', defaultValue: 'bldg:RoofSurface' }
    ],
    preview: `let bounded_by = attributes["bldg:boundedBy"] ?? [];
bounded_by.reduce(|sum| { if this["type"] == "bldg:RoofSurface" { sum + 1 } else { sum } }, 0)`,
    usageExample: 'Count the number of roof surfaces in a building'
  },
  {
    id: 'array-length-check',
    name: 'Check Array Length',
    category: 'array-processing',
    description: 'Get length of array with null safety',
    tags: ['array', 'length', 'count', 'null-safe'],
    rhaiCode: '(env.get("__value").{{arrayAttribute}} ?? []).len()',
    placeholders: [
      { key: 'arrayAttribute', description: 'Array attribute name', defaultValue: 'package' }
    ],
    preview: '(env.get("__value").package ?? []).len()',
    usageExample: 'Get the count of items in a package array'
  },

  // Validation - Common quality check patterns
  {
    id: 'validate-attribute-exists',
    name: 'Validate Attribute Exists',
    category: 'validation',
    description: 'Check if required attribute exists and has value',
    tags: ['validation', 'attribute', 'exists'],
    rhaiCode: `let attributes = env.get("__value").cityGmlAttributes ?? #{};
let {{attributeName}} = attributes["{{attributeKey}}"] ?? [];
{{attributeName}}.len() > 0`,
    placeholders: [
      { key: 'attributeName', description: 'Variable name for attribute', defaultValue: 'building_id' },
      { key: 'attributeKey', description: 'CityGML attribute key', defaultValue: 'uro:buildingIDAttribute' }
    ],
    preview: `let building_id = attributes["uro:buildingIDAttribute"] ?? [];
building_id.len() > 0`,
    usageExample: 'Check if building has required ID attribute'
  },
  {
    id: 'validate-multiple-values',
    name: 'Validate Against Multiple Values',
    category: 'validation',
    description: 'Check if value is one of several allowed values',
    tags: ['validation', 'multiple', 'allowed'],
    rhaiCode: `let value = env.get("__value").{{attributeName}} ?? "";
value in [{{allowedValues}}]`,
    placeholders: [
      { key: 'attributeName', description: 'Attribute to validate', defaultValue: 'featureType' },
      { key: 'allowedValues', description: 'Comma-separated allowed values', defaultValue: '"Building", "BuildingPart", "BuildingInstallation"' }
    ],
    preview: `let value = env.get("__value").featureType ?? "";
value in ["Building", "BuildingPart", "BuildingInstallation"]`,
    usageExample: 'Validate that feature type is one of allowed building types'
  }
];

// Template categories with metadata - focused on advanced patterns not covered by Simple Builder
export const TEMPLATE_CATEGORIES = {
  'file-operations': {
    name: 'Advanced File Operations',
    description: 'Complex file operations beyond simple path building',
    icon: '📁'
  },
  'data-extraction': {
    name: 'CityGML Data Extraction',
    description: 'Extract data from complex CityGML and GML structures',
    icon: '🔍'
  },
  'conditional-logic': {
    name: 'Advanced Conditional Logic',
    description: 'Complex feature classification and filtering patterns',
    icon: '🔀'
  },
  'array-processing': {
    name: 'Array Processing',
    description: 'Collection operations, counting, filtering, and joining',
    icon: '📊'
  },
  'validation': {
    name: 'Data Validation',
    description: 'Quality checks and attribute validation patterns',
    icon: '✅'
  }
} as const;

// Helper function to get templates by category
export function getTemplatesByCategory(category: keyof typeof TEMPLATE_CATEGORIES) {
  return EXPRESSION_TEMPLATES.filter(template => template.category === category);
}

// Helper function to search templates
export function searchTemplates(query: string) {
  const lowerQuery = query.toLowerCase();
  return EXPRESSION_TEMPLATES.filter(template =>
    template.name.toLowerCase().includes(lowerQuery) ||
    template.description.toLowerCase().includes(lowerQuery) ||
    template.tags.some(tag => tag.toLowerCase().includes(lowerQuery)) ||
    template.rhaiCode.toLowerCase().includes(lowerQuery)
  );
}