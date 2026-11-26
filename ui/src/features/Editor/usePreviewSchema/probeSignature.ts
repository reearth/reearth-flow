import type { AnyWorkflowVariable } from "@flow/types";

const VARIABLE_REFERENCE = /\bvariables\s*(?:\[|\.get\()\s*r?"([^"]*)"/g;

const collectStrings = (value: unknown, out: string[]) => {
  if (typeof value === "string") {
    out.push(value);
  } else if (Array.isArray(value)) {
    value.forEach((item) => collectStrings(item, out));
  } else if (value && typeof value === "object") {
    Object.values(value).forEach((item) => collectStrings(item, out));
  }
};

export const referencedVariableNames = (params: unknown): string[] => {
  const strings: string[] = [];
  collectStrings(params, strings);
  const names = new Set<string>();
  for (const text of strings) {
    for (const match of text.matchAll(VARIABLE_REFERENCE)) {
      names.add(match[1]);
    }
  }
  return [...names].sort();
};

export const probeSignature = (
  params: unknown,
  variables: AnyWorkflowVariable[] | undefined,
): string =>
  JSON.stringify([
    params,
    referencedVariableNames(params).map((name) => [
      name,
      variables?.find((variable) => variable.name === name)?.defaultValue ??
        null,
    ]),
  ]);
