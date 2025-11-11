import { useCallback, useEffect, useState } from "react";

import { WorkflowVariable } from "@flow/utils/fromEngineWorkflow/deconstructedEngineWorkflow";

export const useDeploymentWorkflowVariables = (
  initialVariables?: Record<string, any>,
) => {
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
  >(initialVariables);

  // Initialize variables from trigger when editing
  useEffect(() => {
    if (initialVariables) {
      const variablesArray: WorkflowVariable[] = Object.entries(
        initialVariables,
      ).map(([name, value]) => ({
        name,
        value,
      }));

      if (variablesArray.length > 0) {
        setPendingWorkflowData({
          variables: variablesArray,
          workflowName: "",
        });
      }
    }
  }, [initialVariables]);

  const handleWorkflowFetch = useCallback(async (workflowUrl?: string) => {
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

  const initializeVariables = useCallback((variables: Record<string, any>) => {
    setWorkflowVariablesObject(variables);
    const variablesArray: WorkflowVariable[] = Object.entries(variables).map(
      ([name, value]) => ({
        name,
        value,
      }),
    );
    if (variablesArray.length > 0) {
      setPendingWorkflowData({
        variables: variablesArray,
        workflowName: "",
      });
    }
  }, []);

  return {
    pendingWorkflowData,
    workflowVariablesObject,
    openTriggerProjectVariablesDialog,
    setOpenTriggerProjectVariablesDialog,
    handleWorkflowFetch,
    handleVariablesConfirm,
    initializeVariables,
  };
};
