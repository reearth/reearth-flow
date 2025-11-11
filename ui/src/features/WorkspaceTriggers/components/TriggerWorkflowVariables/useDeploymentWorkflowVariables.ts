import { useCallback, useState } from "react";

import { WorkflowVariable } from "@flow/utils/fromEngineWorkflow/deconstructedEngineWorkflow";

export const useDeploymentWorkflowVariables = () => {
  const [
    openTriggerProjectVariablesDialog,
    setOpenTriggerProjectVariablesDialog,
  ] = useState<boolean>(false);

  const [pendingWorkflowData, setPendingWorkflowData] = useState<{
    variables: WorkflowVariable[];
    workflowName: string;
  } | null>(null);

  const [workflowVariablesObject, setWorkflowVariablesObject] = useState<
    Record<string, any> | undefined
  >(undefined);

  const handleWorkflowFileRead = useCallback(async (workflowUrl?: string) => {
    if (!workflowUrl) return;
    const response = await fetch(workflowUrl);

    const jsonData = await response.json();

    const variablesObj = jsonData.with || {};

    setWorkflowVariablesObject(variablesObj);
    const variablesArray: WorkflowVariable[] = Object.entries(variablesObj).map(
      ([name, value]) => ({
        name,
        value,
      }),
    );
    // Return early if there are no variables
    if (variablesArray.length === 0) {
      return;
    }

    setPendingWorkflowData({
      variables: variablesArray,
      workflowName: jsonData.name,
    });
  }, []);

  const handleVariablesConfirm = useCallback((projectVariables: any[]) => {
    const variablesObj: Record<string, any> = {};
    projectVariables.forEach((variable) => {
      variablesObj[variable.name] = variable.defaultValue;
    });
    setWorkflowVariablesObject(variablesObj);
    setOpenTriggerProjectVariablesDialog(false);
  }, []);

  return {
    pendingWorkflowData,
    workflowVariablesObject,
    openTriggerProjectVariablesDialog,
    setOpenTriggerProjectVariablesDialog,
    handleWorkflowFileRead,
    handleVariablesConfirm,
  };
};
