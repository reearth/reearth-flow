import type { DetailsBoxContent } from "./DetailsBox";
import { formatVariableValue } from "./formatVariableValue";

export type WorkflowVariableValue = {
  name: string;
  value: unknown;
};

// The workflow variables as one collapsible row of a details box, or nothing
// when there are none. `labels` are passed in so this stays free of hooks.
export const toWorkflowVariablesDetail = (
  variables: WorkflowVariableValue[] | undefined,
  labels: { title: string; empty: string },
): DetailsBoxContent[] =>
  variables?.length
    ? [
        {
          id: "workflowVariables",
          name: labels.title,
          value: "",
          type: "collapsible",
          items: variables.map(({ name, value }) => ({
            id: `workflowVariables.${name}`,
            name,
            value: formatVariableValue(value, labels.empty),
          })),
        },
      ]
    : [];
