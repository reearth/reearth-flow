/**
 * Carry a stored value across a schema change, keeping whatever the new schema
 * can still hold.
 *
 * What this replaces kept the top-level keys that the new schema also declared,
 * which assumed every schema is an object with `properties`. The compiler
 * supports a union or a map at the root — four actions have one today — and for
 * those the assumption held nothing, so the whole stored value was discarded
 * before the migration form was shown.
 *
 * The walk is conservative in one direction only: a value is kept when the new
 * schema can represent it, and dropped when it cannot. Nothing is invented, and
 * nothing that would not validate is carried through.
 */
import { selectVariant } from "./compile";
import type { FieldNode } from "./types";

const isPlainObject = (value: unknown): value is Record<string, unknown> =>
  typeof value === "object" && value !== null && !Array.isArray(value);

export const migrateValue = (node: FieldNode, value: unknown): unknown => {
  if (value === undefined) return undefined;
  // An explicit "unset" survives only where the new schema still allows it.
  if (value === null) return node.nullable ? null : undefined;

  switch (node.kind) {
    case "object": {
      if (!isPlainObject(value)) return undefined;
      const result: Record<string, unknown> = {};
      for (const property of node.properties) {
        const kept = migrateValue(property, value[property.name]);
        if (kept !== undefined) result[property.name] = kept;
      }
      // An optional section that kept nothing is better absent than empty.
      if (Object.keys(result).length === 0 && node.nullable) return undefined;
      return result;
    }

    case "union": {
      const index = selectVariant(node, value);
      if (index < 0) return undefined;
      const variant = node.variants[index];
      if (variant.constant !== undefined) return variant.constant;

      const kept = migrateValue(variant.node, value);
      if (kept === undefined) return undefined;
      if (variant.discriminator && isPlainObject(kept)) {
        return {
          ...kept,
          [variant.discriminator.property]: variant.discriminator.value,
        };
      }
      return kept;
    }

    case "array": {
      if (!Array.isArray(value)) return undefined;
      const items = value
        .map((item) => migrateValue(node.item, item))
        // A row that kept nothing carries no intent, and keeping it as `{}`
        // would ask the user to fill in required fields for a row they never
        // had. An empty row the user did have is not worth preserving either.
        .filter(
          (item) =>
            item !== undefined &&
            !(isPlainObject(item) && Object.keys(item).length === 0),
        );
      return items.length > 0 || !node.nullable ? items : undefined;
    }

    case "map": {
      if (!isPlainObject(value)) return undefined;
      const entries = Object.entries(value)
        .map(([key, entry]) => [key, migrateValue(node.value, entry)] as const)
        .filter(([, entry]) => entry !== undefined);
      return entries.length > 0 || !node.nullable
        ? Object.fromEntries(entries)
        : undefined;
    }

    case "enum":
      return node.options.some((option) => option.value === value)
        ? value
        : undefined;

    case "expr": {
      // A code field is `{ type, value }`; anything else cannot be carried.
      if (!isPlainObject(value) || typeof value.value !== "string")
        return undefined;
      const type = node.allowedTypes.includes(value.type as never)
        ? value.type
        : node.allowedTypes[0];
      return { type, value: value.value };
    }

    case "string":
    case "color":
    case "wysiwyg":
      return typeof value === "string" ? value : undefined;

    case "number":
      return typeof value === "number" ? value : undefined;

    case "boolean":
      return typeof value === "boolean" ? value : undefined;

    case "unsupported":
      // Nothing is known about the shape, so nothing can be judged about it.
      return value;
  }
};
