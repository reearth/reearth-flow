import { useMemo } from "react";

import { useWorkflowVariables } from "@flow/lib/gql";
import { redactPrivateWorkflowVariableValues } from "@flow/utils";

/**
 * Workflow variables for a shared project, with the values of non-public ones
 * withheld. Shared-canvas code should always read variables through this rather
 * than `useGetWorkflowVariables`, so a private value cannot leak by omission.
 */
export default (projectId?: string) => {
  const { useGetWorkflowVariables } = useWorkflowVariables();
  const { workflowVariables, ...rest } = useGetWorkflowVariables(projectId);

  const redactedWorkflowVariables = useMemo(
    () => redactPrivateWorkflowVariableValues(workflowVariables),
    [workflowVariables],
  );

  return { workflowVariables: redactedWorkflowVariables, ...rest };
};
