import type { AnyWorkflowVariable } from "@flow/types";

import { getDefaultValueForWorkflowVar } from "./getDefaultValueForWorkflowVar";

// Withholds the stored values of non-public workflow variables.

export const redactPrivateWorkflowVariableValues = (
  workflowVariables?: AnyWorkflowVariable[],
): AnyWorkflowVariable[] =>
  workflowVariables?.map((variable) =>
    variable.public === true
      ? variable
      : {
          ...variable,
          defaultValue: getDefaultValueForWorkflowVar(variable.type),
        },
  ) ?? [];
