/**
 * Schemas for the SchemaForm stories.
 *
 * Hand-written in the dialect `schemars` emits into
 * `engine/schema/actions.json`, so each field kind can be looked at on its own
 * — the engine's real schemas mix several per action, and nothing under `src/`
 * may import from `engine/` (the UI image builds with `context: ui`).
 *
 * `storyFixtures.test.ts` compiles these, so a fixture that stops producing the
 * kind its story is named after fails rather than quietly drawing the wrong
 * control. New field kind, new property here.
 */
import type { JSONSchema7 } from "json-schema";

export const DRAFT = "http://json-schema.org/draft-07/schema#";

/** The `format: "code"` shape, as the engine spells it. */
export const exprProperty = (title: string, description: string): JSONSchema7 =>
  ({
    title,
    description,
    type: ["object", "null"],
    format: "code",
    required: ["type", "value"],
    properties: {
      type: { type: "string", enum: ["flowExpr", "string"] },
      value: { type: "string" },
    },
  }) as JSONSchema7;

export const FIELD_KINDS_SCHEMA: JSONSchema7 = {
  $schema: DRAFT,
  title: "Every Field Kind",
  description:
    "One property per FieldNode kind, so the whole renderer is visible at once.",
  type: "object",
  required: ["name", "mode"],
  properties: {
    name: {
      title: "Name (string)",
      description: "A required string.",
      type: "string",
      minLength: 1,
    },
    featureCount: {
      title: "Feature Count (integer)",
      description: "An integer with bounds.",
      type: "integer",
      format: "uint32",
      minimum: 0,
      maximum: 1000,
      default: 100,
    },
    tolerance: {
      title: "Tolerance (number)",
      type: "number",
      format: "double",
      default: 0.001,
    },
    keepOriginal: {
      title: "Keep Original (boolean)",
      type: "boolean",
      default: false,
    },
    mode: {
      title: "Mode (enum)",
      description: "A plain Rust enum: oneOf over bare constants.",
      allOf: [{ $ref: "#/definitions/Mode" }],
    },
    groupBy: {
      title: "Group By (array)",
      type: ["array", "null"],
      items: { type: "string" },
    },
    extent: {
      title: "Extent (object)",
      description: "A nested object section.",
      type: ["object", "null"],
      required: ["minX", "minY"],
      properties: {
        minX: { title: "Min X", type: "number" },
        minY: { title: "Min Y", type: "number" },
        maxX: { title: "Max X", type: ["number", "null"] },
        maxY: { title: "Max Y", type: ["number", "null"] },
      },
    },
    renames: {
      title: "Renames (map)",
      description: "additionalProperties — keys the user names.",
      type: ["object", "null"],
      additionalProperties: { type: "string" },
    },
    projection: {
      title: "Projection (union)",
      description: "A tagged union; the tag is implied by the choice.",
      allOf: [{ $ref: "#/definitions/Projection" }],
    },
    filter: exprProperty(
      "Filter (expr)",
      "A FlowExpr or a literal string — format: code.",
    ),
    strokeColor: {
      title: "Stroke Color (color)",
      type: ["string", "null"],
      format: "color",
      default: "#4f46e5",
    },
    notes: {
      title: "Notes (wysiwyg)",
      type: ["string", "null"],
      format: "wysiwyg",
    },
    legacyOption: {
      title: "Legacy Option (unsupported)",
      description:
        "Two concrete types at once. The compiler will not guess; it hands over a raw-JSON editor rather than dropping the value.",
      type: ["string", "number"],
    },
  },
  definitions: {
    Mode: {
      oneOf: [
        {
          title: "Fast",
          description: "Trades accuracy for throughput.",
          type: "string",
          enum: ["fast"],
        },
        {
          title: "Accurate",
          description: "Trades throughput for accuracy.",
          type: "string",
          enum: ["accurate"],
        },
      ],
    },
    Projection: {
      oneOf: [
        {
          title: "EPSG Code",
          description: "Identify the CRS by its EPSG code.",
          type: "object",
          required: ["type", "code"],
          properties: {
            type: { type: "string", enum: ["epsg"] },
            code: {
              title: "Code",
              type: "integer",
              minimum: 1024,
              maximum: 32767,
            },
          },
        },
        {
          title: "WKT",
          description: "Supply the CRS as well-known text.",
          type: "object",
          required: ["type", "wkt"],
          properties: {
            type: { type: "string", enum: ["wkt"] },
            wkt: { title: "WKT", type: "string" },
          },
        },
        {
          title: "Passthrough",
          description: "A variant with no fields of its own.",
          type: "object",
          required: ["type"],
          properties: { type: { type: "string", enum: ["passthrough"] } },
        },
      ],
    },
  },
};

export const VALIDATION_SCHEMA: JSONSchema7 = {
  $schema: DRAFT,
  title: "Validation",
  description: "AJV runs against this schema exactly as written.",
  type: "object",
  required: ["outputAttribute", "threshold"],
  properties: {
    outputAttribute: {
      title: "Output Attribute",
      description: "Required, and must not be empty.",
      type: "string",
      minLength: 1,
    },
    threshold: {
      title: "Threshold",
      description: "Required, 0 to 1 inclusive.",
      type: "number",
      minimum: 0,
      maximum: 1,
    },
    epsgCode: {
      title: "EPSG Code",
      description: "Optional, but must look like EPSG:6668 when given.",
      type: ["string", "null"],
      pattern: "^EPSG:[0-9]{4,5}$",
    },
    attributes: {
      title: "Attributes",
      description: "At least one entry.",
      type: "array",
      items: { type: "string" },
      minItems: 1,
    },
  },
};

export const NULLABILITY_SCHEMA: JSONSchema7 = {
  $schema: DRAFT,
  title: "Nullability",
  description: "Every field here admits being unset.",
  type: "object",
  properties: {
    attribute: { title: "Attribute", type: ["string", "null"] },
    limit: {
      title: "Limit",
      type: ["integer", "null"],
      format: "uint32",
      minimum: 0,
    },
    caseSensitive: { title: "Case Sensitive", type: ["boolean", "null"] },
    bounds: {
      title: "Bounds",
      description: "A whole optional section.",
      type: ["object", "null"],
      required: ["minX", "maxX"],
      properties: {
        minX: { title: "Min X", type: "number" },
        maxX: { title: "Max X", type: "number" },
      },
    },
    tags: {
      title: "Tags",
      type: ["array", "null"],
      items: { type: "string" },
    },
  },
};

export const UNIONS_SCHEMA: JSONSchema7 = {
  $schema: DRAFT,
  title: "Unions",
  type: "object",
  required: ["projection", "selector"],
  properties: {
    projection: {
      title: "Projection (tagged)",
      description: "Every branch agrees on a `type` tag.",
      allOf: [{ $ref: "#/definitions/Projection" }],
    },
    selector: {
      title: "Selector (untagged)",
      description: "No tag — the branch is matched by the value's own shape.",
      allOf: [{ $ref: "#/definitions/Selector" }],
    },
  },
  definitions: {
    Projection: FIELD_KINDS_SCHEMA.definitions?.Projection as JSONSchema7,
    Selector: {
      oneOf: [
        {
          title: "Single Attribute",
          type: "object",
          required: ["attribute"],
          properties: { attribute: { title: "Attribute", type: "string" } },
        },
        {
          title: "Multiple Attributes",
          type: "object",
          required: ["attributes"],
          properties: {
            attributes: {
              title: "Attributes",
              type: "array",
              items: { type: "string" },
            },
          },
        },
      ],
    },
  },
};

export const CONTAINERS_SCHEMA: JSONSchema7 = {
  $schema: DRAFT,
  title: "Arrays, Objects and Maps",
  type: "object",
  required: ["rules"],
  properties: {
    rules: {
      title: "Rules",
      description: "An array of objects — add, remove and reorder.",
      type: "array",
      minItems: 1,
      items: { $ref: "#/definitions/Rule" },
    },
    tables: {
      title: "Tables",
      description:
        "A map whose keys the user types. Keys containing dots and colons are real: the engine documents names like bldg:Building.",
      type: ["object", "null"],
      additionalProperties: { $ref: "#/definitions/Table" },
    },
  },
  definitions: {
    Rule: {
      title: "Rule",
      type: "object",
      required: ["attribute", "operator"],
      properties: {
        attribute: { title: "Attribute", type: "string" },
        operator: {
          title: "Operator",
          type: "string",
          enum: ["equals", "contains", "matches"],
        },
        value: { title: "Value", type: ["string", "null"] },
      },
    },
    Table: {
      title: "Table",
      type: "object",
      required: ["columns"],
      properties: {
        columns: {
          title: "Columns",
          type: "array",
          items: { type: "string" },
        },
        hasHeader: { title: "Has Header", type: ["boolean", "null"] },
      },
    },
  },
};

/**
 * Copied from the engine's `Python Script Processor`, which is what makes the
 * `script` field open the Python editor rather than the FlowExpr one — the
 * flavour comes from an override keyed by action name and path, not from the
 * schema.
 */

export const PYTHON_SCRIPT_SCHEMA: JSONSchema7 = {
  $schema: DRAFT,
  title: "PythonScriptProcessorParam",
  type: "object",
  properties: {
    script: exprProperty(
      "Inline Script",
      "Python script code to execute inline",
    ),
    pythonFile: exprProperty(
      "Python File",
      "Path to a Python script file (supports file://, http://, https://, gs://, etc.)",
    ),
    pythonPath: {
      title: "Python Path",
      description: "Path to Python interpreter executable (default: python3)",
      type: ["string", "null"],
    },
    timeoutSeconds: {
      title: "Timeout Seconds",
      description:
        "Maximum execution time for the Python script in seconds (default: 30)",
      type: ["integer", "null"],
      format: "uint64",
      minimum: 0,
    },
  },
};

export const UNSUPPORTED_SCHEMA: JSONSchema7 = {
  $schema: DRAFT,
  title: "Shapes The Compiler Does Not Recognise",
  description:
    "A custom action need not follow the schemars dialect. Each of these renders as a labelled raw-JSON editor, which costs the user a nicer control but never their data. Built-in actions are held to zero of them by corpus.test.ts.",
  type: "object",
  properties: {
    mixedType: { title: "Mixed Type", type: ["string", "number"] },
    looseArray: { title: "Loose Array", type: "array" },
    anythingGoes: { title: "Anything Goes" },
  },
};
