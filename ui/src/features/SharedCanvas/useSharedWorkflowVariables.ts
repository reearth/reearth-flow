import { useParams } from "@tanstack/react-router";
import { useMemo } from "react";

import { useSharedProject } from "@flow/lib/gql";
import { redactPrivateWorkflowVariableValues } from "@flow/utils";

/**
 * Workflow variables for the shared project on the current share URL.
 *
 * They come from `sharedProject(token)`, which the server resolves through the
 * sharing token and which never sends the values of non-public variables. The
 * shared viewer usually holds no workspace role, so it must not use
 * `useGetWorkflowVariables` (the `parameters(projectId)` query): that returns
 * every value to anyone permitted to call it.
 *
 * The client-side redaction is kept so non-public variables still get a
 * type-appropriate empty value rather than `null` on import.
 */
export default () => {
  const { sharedToken } = useParams({ strict: false });
  const { useGetSharedProject } = useSharedProject();
  const { workflowVariables, ...rest } = useGetSharedProject(sharedToken);

  const redactedWorkflowVariables = useMemo(
    () => redactPrivateWorkflowVariableValues(workflowVariables),
    [workflowVariables],
  );

  return { workflowVariables: redactedWorkflowVariables, ...rest };
};
