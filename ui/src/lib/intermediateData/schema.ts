/**
 * The engine's intermediate-data schema, read directly and reduced to the few
 * maps the UI needs.
 *
 * `feature-intermediate.schema.json` beside this file is a committed copy of
 * `engine/schema/feature-intermediate.schema.json`, which `cargo make
 * schema-feature` generates. Do not hand-edit it; copy the original across when
 * the engine's changes, which `ci_ui.yml` fails the build over if forgotten.
 *
 * It is copied rather than imported because the production image builds with
 * `context: ui` and so has no engine tree at all. The reductions below run once
 * at module load over ~54 definitions.
 *
 * Two things come out of it. The titles are the names the engine intends a UI
 * to show ("Point (2D)", not "Point2D"), which exist only in the schema — the
 * `schemars(title = ...)` attributes are compiled out of the serde path, so no
 * JSONL line carries them. The other is the shape of the tagged enums, so a
 * geometry's type can be read off its discriminant key rather than guessed at.
 */
import rawSchema from "./feature-intermediate.schema.json";

/** Loosely typed schema node; the shapes are checked as they are read. */
type SchemaNode = Record<string, any>;

const DEF_PREFIX = "#/definitions/";

const definitions: Record<string, SchemaNode> =
  (rawSchema as SchemaNode).definitions ?? {};

/** Definition name a `$ref` points at, or null for anything else. */
function refTarget(node: SchemaNode | undefined): string | null {
  const ref = node?.$ref;
  return typeof ref === "string" && ref.startsWith(DEF_PREFIX)
    ? ref.slice(DEF_PREFIX.length)
    : null;
}

/**
 * Every definition reachable from a schema node without passing through
 * another definition. Refs hide behind four shapes here: a bare `$ref`, an
 * `allOf`/`anyOf`/`oneOf` branch (the nullable-property form is
 * `anyOf: [{$ref}, {type: "null"}]`), array `items`, and the properties of an
 * inline object.
 */
function directRefs(node: unknown, found = new Set<string>()): Set<string> {
  if (!node || typeof node !== "object") return found;
  if (Array.isArray(node)) {
    for (const item of node) directRefs(item, found);
    return found;
  }

  const record = node as SchemaNode;
  const target = refTarget(record);
  if (target) {
    found.add(target);
    return found;
  }

  for (const key of ["allOf", "anyOf", "oneOf", "items"]) {
    if (record[key]) directRefs(record[key], found);
  }
  if (record.properties) {
    for (const property of Object.values(record.properties)) {
      directRefs(property, found);
    }
  }
  if (
    record.additionalProperties &&
    typeof record.additionalProperties === "object"
  ) {
    directRefs(record.additionalProperties, found);
  }
  return found;
}

/**
 * The definition an enum variant's payload resolves to, when it resolves to
 * exactly one. Going through `directRefs` rather than reading `$ref` straight
 * off means a variant that wraps its target in an array or a tuple — `Csg::Union`
 * is a two-element tuple of `ThreeDimensional` — resolves the way a plain
 * `$ref` does instead of dead-ending.
 */
function soleTarget(node: unknown): string | null {
  const reachable = [...directRefs(node)];
  return reachable.length === 1 ? reachable[0] : null;
}

export type EnumSchema = {
  /** Discriminant key -> the definition it carries, null for a primitive. */
  variants: Record<string, string | null>;
  /** Unit variants, which serde writes as a bare string with no wrapper. */
  units: string[];
};

/** Human-facing name per definition, e.g. Point2D -> "Point (2D)". */
export const DEFINITION_TITLES: Record<string, string> = {};

/** Externally-tagged enums, keyed by definition name. */
export const ENUMS: Record<string, EnumSchema> = {};

for (const [name, definition] of Object.entries(definitions)) {
  if (definition.title) DEFINITION_TITLES[name] = definition.title;

  if (Array.isArray(definition.oneOf)) {
    const variants: Record<string, string | null> = {};
    const units: string[] = [];
    for (const branch of definition.oneOf as SchemaNode[]) {
      if (Array.isArray(branch.enum)) {
        units.push(...branch.enum);
        continue;
      }
      for (const [key, payload] of Object.entries(branch.properties ?? {})) {
        variants[key] = soleTarget(payload);
      }
    }
    ENUMS[name] = { variants, units };
  }
}
