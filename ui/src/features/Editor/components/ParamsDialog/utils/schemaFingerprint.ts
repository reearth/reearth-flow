import { RJSFSchema } from "@rjsf/utils";

function simpleHash(str: string): string {
  let hash = 0;
  for (let i = 0; i < str.length; i++) {
    const char = str.charCodeAt(i);
    hash = (hash << 5) - hash + char;
    hash = hash & hash;
  }
  return hash.toString(36);
}

function sortDeep(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(sortDeep);
  if (value !== null && typeof value === "object") {
    return Object.keys(value as object)
      .sort()
      .reduce(
        (acc, key) => {
          acc[key] = sortDeep((value as Record<string, unknown>)[key]);
          return acc;
        },
        {} as Record<string, unknown>,
      );
  }
  return value;
}

export function computeSchemaFingerprint(
  schema?: RJSFSchema,
): string | undefined {
  if (!schema?.properties) return undefined;
  return simpleHash(JSON.stringify(sortDeep(schema.properties)));
}

// Returns true (no migration needed) when:
// - storedSchema is undefined (first-time node, never been saved with a schema)
// - schemas produce the same fingerprint
export function schemasMatch(
  storedSchema: RJSFSchema | undefined,
  currentSchema: RJSFSchema,
): boolean {
  if (!storedSchema) return true;
  const storedHash = computeSchemaFingerprint(storedSchema);
  const currentHash = computeSchemaFingerprint(currentSchema);
  return storedHash === currentHash;
}
