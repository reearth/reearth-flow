import type { AnyWorkflowVariable } from "@flow/types";

import type { AutocompleteSuggestion } from "./flowExprConstants";

/**
 * Workflow variables as `env["…"]` autocomplete entries — the env counterpart
 * to the reader-schema attribute suggestions. `env` reads a workflow variable
 * by name, so the project's variables are exactly the valid keys.
 */
export const toEnvAutocompleteSuggestions = (
  workflowVariables?: AnyWorkflowVariable[],
): AutocompleteSuggestion[] =>
  workflowVariables?.map((variable) => ({
    label: variable.name,
    insertText: variable.name,
    type: "variable",
    detail: variable.type,
  })) ?? [];
