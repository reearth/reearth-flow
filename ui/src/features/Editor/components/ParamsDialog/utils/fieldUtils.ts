/**
 * Utility functions for handling field paths and value manipulation in RJSF forms
 */

/**
 * Extracts field path from RJSF field ID
 * @param id - RJSF field ID (e.g., "root_param1_subparam")
 * @returns Array of path segments (e.g., ["param1", "subparam"])
 */
export function extractFieldPath(id: string): string[] {
  return id
    .replace(/^root_/, "")
    .split("_")
    .filter(Boolean);
}

/**
 * Gets value at a specific path in a nested object
 * @param obj - The object to traverse
 * @param path - Array of keys representing the path
 * @returns The value at the specified path, or undefined if not found
 */
export function getValueAtPath(obj: any, path: string[]): any {
  return path.reduce((current, key) => current?.[key], obj);
}

/**
 * Sets value at a specific path in a nested object (immutably)
 * @param obj - The object to update
 * @param path - Array of keys representing the path
 * @param value - The value to set
 * @returns A new object with the value set at the specified path
 */
export function setValueAtPath(obj: any, path: string[], value: any): any {
  if (path.length === 0) {
    return value;
  }

  const newObj = { ...obj };
  let current = newObj;

  // Navigate to the parent of the target field
  for (let i = 0; i < path.length - 1; i++) {
    const key = path[i];
    if (!(key in current)) {
      current[key] = {};
    } else {
      current[key] = { ...current[key] };
    }
    current = current[key];
  }

  // Set the final value
  const finalKey = path[path.length - 1];
  current[finalKey] = value;

  return newObj;
}

/**
 * Represents field context information for the value editor
 */
export type FieldContext = {
  id: string;
  name: string;
  path: string[];
  value: any;
  schema: any;
  fieldName: string;
};

/**
 * Creates field context from RJSF field props
 * @param props - RJSF field props
 * @returns Field context object
 */
export function createFieldContext(props: {
  id: string;
  name: string;
  value: any;
  schema: any;
}): FieldContext {
  const path = extractFieldPath(props.id);
  const fieldName = path[path.length - 1] || props.name;

  return {
    id: props.id,
    name: props.name,
    path,
    value: props.value,
    schema: props.schema,
    fieldName,
  };
}
