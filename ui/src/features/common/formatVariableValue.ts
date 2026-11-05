const isEmpty = (value: unknown) =>
  value === null || value === undefined || value === "";

// One value as a line of text; anything structured is shown as JSON.
const toLine = (value: unknown, empty: string): string => {
  if (isEmpty(value)) return empty;
  if (typeof value === "object") return JSON.stringify(value);
  return String(value);
};

// A variable's value as a line of text, or one line per item for an array.
export const formatVariableValue = (
  value: unknown,
  empty: string,
): string | string[] => {
  if (Array.isArray(value)) {
    return value.length ? value.map((v) => toLine(v, empty)) : empty;
  }
  return toLine(value, empty);
};
