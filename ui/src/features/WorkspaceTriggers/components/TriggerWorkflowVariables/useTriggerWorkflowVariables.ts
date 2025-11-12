import { useCallback, useState } from "react";

import { WorkflowVariable } from "@flow/utils/fromEngineWorkflow/deconstructedEngineWorkflow";

export const useTriggerWorkflowVariables = (
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
  >(undefined);

  // Store the original deployment workflow variables for comparison
  const [deploymentDefaultVariables, setDeploymentDefaultVariables] = useState<
    Record<string, any> | undefined
  >(undefined);

  // Store initial trigger custom variables separately
  const [triggerCustomVariables] = useState<Record<string, any> | undefined>(
    initialVariables,
  );

  const handleWorkflowFetch = useCallback(
    async (workflowUrl?: string, hasInitialVariables = false) => {
      if (!workflowUrl) return;
      try {
        const response = await fetch(workflowUrl);
        if (!response.ok) {
          return;
        }
        const jsonData = await response.json();
        const deploymentVars = jsonData.with || {};

        // Store deployment default variables for comparison
        setDeploymentDefaultVariables(deploymentVars);

        // Merge deployment defaults with trigger custom values
        // This ensures new variables from deployment are visible while preserving custom values
        let mergedVariables = { ...deploymentVars };

        if (hasInitialVariables && triggerCustomVariables) {
          // Override deployment defaults with trigger custom values where they exist
          mergedVariables = { ...deploymentVars, ...triggerCustomVariables };
        }

        setWorkflowVariablesObject(mergedVariables);

        const variablesArray: WorkflowVariable[] = Object.entries(
          mergedVariables,
        ).map(([name, value]) => ({
          name,
          value,
        }));

        // Return early if there are no variables
        if (variablesArray.length === 0) {
          return;
        }

        setPendingWorkflowData({
          variables: variablesArray,
          workflowName: jsonData.name,
        });
      } catch (error) {
        console.error("Failed to fetch workflow:", error);
        return;
      }
    },
    [triggerCustomVariables],
  );

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

  // Compare current variables with deployment defaults
  // Returns undefined if they match (don't save), or only the customized variables if they differ
  const getVariablesToSave = useCallback(():
    | Record<string, any>
    | undefined => {
    if (!workflowVariablesObject || !deploymentDefaultVariables) {
      return workflowVariablesObject;
    }

    const customizedVars: Record<string, any> = {};
    let hasCustomizations = false;

    // Only include variables that differ from deployment defaults
    Object.entries(workflowVariablesObject).forEach(([key, value]) => {
      const defaultValue = deploymentDefaultVariables[key];

      // Include this variable if:
      // 1. It doesn't exist in deployment defaults (removed from deployment)
      // 2. The value differs from deployment default
      if (
        defaultValue === undefined ||
        JSON.stringify(value) !== JSON.stringify(defaultValue)
      ) {
        customizedVars[key] = value;
        hasCustomizations = true;
      }
    });

    if (!hasCustomizations) {
      return undefined;
    }

    return customizedVars;
  }, [workflowVariablesObject, deploymentDefaultVariables]);

  return {
    pendingWorkflowData,
    workflowVariablesObject,
    openTriggerProjectVariablesDialog,
    setOpenTriggerProjectVariablesDialog,
    handleWorkflowFetch,
    handleVariablesConfirm,
    initializeVariables,
    getVariablesToSave,
    deploymentDefaultVariables,
  };
};
