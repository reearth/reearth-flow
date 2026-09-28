import type { FlowSchema } from "@flow/lib/schemaForm";

function simpleHash(str: string): string {
  let hash = 0;
  for (let i = 0; i < str.length; i++) {
    const char = str.charCodeAt(i);
    hash = (hash << 5) - hash + char;
    hash = hash & hash;
  }
  return hash.toString(36);
}

// Action schemas are fetched per-language, so translated text must not
// affect the fingerprint — otherwise collaborators using different UI
// languages would trigger false migrations for each other.
const I18N_KEYS = new Set(["title", "description"]);

function normalizeSchema(value: any): any {
  if (Array.isArray(value)) return value.map(normalizeSchema);
  if (value && typeof value === "object") {
    const result: Record<string, any> = {};
    for (const key of Object.keys(value).sort()) {
      if (I18N_KEYS.has(key)) continue;
      result[key] = normalizeSchema(value[key]);
    }
    return result;
  }
  return value;
}

// Structural fingerprint of a params schema: covers property keys (at any
// depth), types, enums, required, format, defaults, oneOf/anyOf variants —
// everything except translated title/description text.
export function computeSchemaFingerprint(
  schema?: FlowSchema,
): string | undefined {
  // Only an absent schema has no fingerprint. Returning undefined for a schema
  // without top-level `properties` gave the four union-rooted actions no
  // fingerprint at all, so `schemasMatch` compared undefined to undefined and
  // reported "unchanged" however much their schema had moved.
  if (!schema) return undefined;
  return simpleHash(JSON.stringify(normalizeSchema(schema)));
}

// Returns true (no migration needed) when:
// - storedSchema is undefined (legacy node saved before schemas were
//   stamped at creation — no baseline to compare against)
// - schemas produce the same structural fingerprint
export function schemasMatch(
  storedSchema: FlowSchema | undefined,
  currentSchema: FlowSchema,
): boolean {
  if (!storedSchema) return true;
  const storedHash = computeSchemaFingerprint(storedSchema);
  const currentHash = computeSchemaFingerprint(currentSchema);
  return storedHash === currentHash;
}
