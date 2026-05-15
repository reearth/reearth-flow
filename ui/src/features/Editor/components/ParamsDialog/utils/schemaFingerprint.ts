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

export function computeSchemaFingerprint(
  schema?: RJSFSchema,
): string | undefined {
  if (!schema?.properties) return undefined;
  const sortedKeys = Object.keys(schema.properties).sort();
  const properties = schema.properties;
  const normalized = sortedKeys.reduce(
    (acc, key) => {
      acc[key] = properties[key];
      return acc;
    },
    {} as Record<string, unknown>,
  );
  return simpleHash(JSON.stringify(normalized));
}

// Returns true (no migration needed) when:
// - storedSchema is undefined (first-time node, never been saved with a schema)
// - schemas produce the same fingerprint
export function schemasMatch(
  storedSchema: RJSFSchema | undefined,
  currentSchema?: RJSFSchema,
): boolean {
  if (!storedSchema) return true;
  const storedHash = computeSchemaFingerprint(storedSchema);
  const currentHash = computeSchemaFingerprint(currentSchema);
  if (!currentHash) return true;
  return storedHash === currentHash;
}
