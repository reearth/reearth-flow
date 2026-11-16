import type { VarType } from "@flow/types";

// Workflow variables reach the engine keyed by name, so a name must be unique
// within the project and, by convention, camelCase with no whitespace.

const camelCase = (snake: string) =>
  snake.replace(/_([a-z])/g, (_, letter: string) => letter.toUpperCase());

// A name for a newly added variable, from its type: "choiceVariable", then
// "choiceVariable2" and so on, taking the lowest number not in use. Not
// translated, since it is an identifier rather than a label.
export const defaultVariableName = (
  type: VarType,
  existingNames: Iterable<string>,
) => {
  const base = `${camelCase(type)}Variable`;
  const taken = new Set(existingNames);
  if (!taken.has(base)) return base;
  let n = 2;
  while (taken.has(`${base}${n}`)) n++;
  return `${base}${n}`;
};

export type VariableNameProblems = {
  hasEmpty: boolean;
  withWhitespace: string[];
  duplicates: string[];
};

// What stops a set of variables from being saved, by name.
export const findVariableNameProblems = (
  variables: { name: string }[],
): VariableNameProblems => {
  const seen = new Set<string>();
  const duplicates = new Set<string>();
  const withWhitespace = new Set<string>();
  let hasEmpty = false;
  for (const { name } of variables) {
    if (name === "") hasEmpty = true;
    else if (/\s/.test(name)) withWhitespace.add(name);
    if (seen.has(name) && name !== "") duplicates.add(name);
    seen.add(name);
  }
  return {
    hasEmpty,
    withWhitespace: [...withWhitespace],
    duplicates: [...duplicates],
  };
};

export const hasVariableNameProblems = (problems: VariableNameProblems) =>
  problems.hasEmpty ||
  problems.withWhitespace.length > 0 ||
  problems.duplicates.length > 0;

// Whether this name is one of the problems found.
export const isProblemVariableName = (
  name: string,
  problems: VariableNameProblems,
) =>
  name === ""
    ? problems.hasEmpty
    : problems.withWhitespace.includes(name) ||
      problems.duplicates.includes(name);
