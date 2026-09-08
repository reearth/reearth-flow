import type { AnyWorkflowVariable } from "@flow/types";

import type { AutocompleteSuggestion } from "./flowExprConstants";

/**
 * Workflow variables as `variables["…"]` autocomplete entries — the variables counterpart
 * to the reader-schema attribute suggestions. `variables` reads a workflow variable
 * by name, so the project's variables are exactly the valid keys.
 */
export const toVariableAutocompleteSuggestions = (
  workflowVariables?: AnyWorkflowVariable[],
): AutocompleteSuggestion[] =>
  workflowVariables?.map((variable) => ({
    label: variable.name,
    insertText: variable.name,
    type: "variable",
    detail: variable.type,
  })) ?? [];
