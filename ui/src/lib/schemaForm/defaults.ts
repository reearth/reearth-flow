/**
 * Seed a value with the defaults the schema actually states — and nothing else.
 *
 * The rule is deliberately narrow: a default is applied where the schema
 * declares one, and an optional section is never conjured into existence. The
 * pipeline this replaces did the opposite. Because patching had deleted the
 * `null` branch of every `anyOf: [X, null]`, RJSF's `getDefaultFormState` read
 * each optional section as mandatory and materialised it — filling a fresh HTTP
 * Caller with a half-built `authentication`, a `rateLimit` missing its own
 * required `requests`, and a `responseEncoding` of `"text"` that the schema
 * never mentions — then wrote the lot back through `onChange` on mount.
 *
 * A required field that is missing stays missing. That is a validation error
 * for the user to resolve, not a hole for the form to fill with a guess.
 */
import { selectVariant } from "./compile";
import type { FieldNode } from "./types";

const clone = <T>(value: T): T =>
  value === null || typeof value !== "object"
    ? value
    : (structuredClone(value) as T);

const isPlainObject = (value: unknown): value is Record<string, unknown> =>
  typeof value === "object" && value !== null && !Array.isArray(value);

export const applyDefaults = (node: FieldNode, value: unknown): unknown => {
  if (value === undefined) {
    if (node.default !== undefined) return clone(node.default);
    // Descend into a section that must exist anyway, so its own defaults are
    // picked up; stop at anything the schema says may be absent.
    if (node.kind !== "object" || node.nullable) return undefined;
  }

  switch (node.kind) {
    case "object": {
      const source = isPlainObject(value) ? value : {};
      const result: Record<string, unknown> = { ...source };
      let filled = false;

      for (const property of node.properties) {
        const seeded = applyDefaults(property, source[property.name]);
        if (seeded !== undefined) {
          result[property.name] = seeded;
          filled = true;
        }
      }

      if (value === undefined && !filled) return undefined;
      return result;
    }

    case "array": {
      if (!Array.isArray(value)) return value;
      return value.map((item) => applyDefaults(node.item, item) ?? item);
    }

    case "map": {
      if (!isPlainObject(value)) return value;
      return Object.fromEntries(
        Object.entries(value).map(([key, entry]) => [
          key,
          applyDefaults(node.value, entry) ?? entry,
        ]),
      );
    }

    case "union": {
      if (value === undefined || value === null) return value;
      const index = selectVariant(node, value);
      if (index < 0) return value;

      const variant = node.variants[index];
      const seeded = applyDefaults(variant.node, value);
      // The tag identifies the variant and is not a field, so it survives the
      // round trip here rather than through the rendered form.
      if (variant.discriminator && isPlainObject(seeded)) {
        return {
          ...seeded,
          [variant.discriminator.property]: variant.discriminator.value,
        };
      }
      return seeded;
    }

    default:
      return value;
  }
};
