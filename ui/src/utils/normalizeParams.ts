import type { NodeParams } from "@flow/types";

const EMPTY = Symbol("emptyParamValue");

type CodeValue = { type: string; value: unknown };

const isCodeValue = (value: unknown): value is CodeValue =>
  typeof value === "object" &&
  value !== null &&
  !Array.isArray(value) &&
  "value" in value &&
  "type" in value;

const isEmptyCodeValue = (codeValue: CodeValue): boolean => {
  const { type, value } = codeValue;
  if (value === undefined || value === null) return true;
  if (typeof value !== "string") return false;
  return type === "flowExpr" ? value.trim() === "" : value === "";
};

const normalizeValue = (value: unknown): unknown | typeof EMPTY => {
  if (value === undefined || value === null || value === "") return EMPTY;

  if (isCodeValue(value)) return isEmptyCodeValue(value) ? EMPTY : value;

  if (Array.isArray(value)) {
    const items = value.map(normalizeValue).filter((item) => item !== EMPTY);
    return items.length ? items : EMPTY;
  }

  if (typeof value === "object") {
    const entries = Object.entries(value)
      .map(([key, entryValue]) => [key, normalizeValue(entryValue)] as const)
      .filter(([, entryValue]) => entryValue !== EMPTY);
    return entries.length ? Object.fromEntries(entries) : EMPTY;
  }

  return value;
};

/**
 * Drops empty params before they are persisted.
 *
 * The engine treats an absent param as "not set" (`None`, or the serde default),
 * but a param that is present and blank is a value it has to use: an empty
 * FlowExpr (`{ type: "flowExpr", value: "" }`) fails to compile and takes the
 * whole run down, and an empty string is never a meaningful engine value.
 *
 * Applied on save so the saved and exported workflow is clean too, not just the
 * engine-ready conversion. Runs recursively, since blank fields also show up
 * inside nested objects and rows of array params. A container left with nothing
 * meaningful inside is dropped as well, so an all-blank row never reaches the
 * engine as `{}`.
 */
export const normalizeParams = (params: unknown): NodeParams | undefined => {
  if (params === undefined || params === null) return undefined;
  if (typeof params !== "object" || Array.isArray(params)) return undefined;

  const normalized = normalizeValue(params);
  return normalized === EMPTY ? {} : (normalized as NodeParams);
};
