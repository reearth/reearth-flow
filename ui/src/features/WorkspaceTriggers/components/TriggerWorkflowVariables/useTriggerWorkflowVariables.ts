import { useCallback, useState } from "react";

import { inferProjectVariableType } from "@flow/features/WorkspaceProjects/components/WorkflowImport/inferVariableType";
import { toGqlParameterType } from "@flow/lib/gql/convert";
import { Variable } from "@flow/types";
import { WorkflowVariable } from "@flow/utils/fromEngineWorkflow/deconstructedEngineWorkflow";

/**
 * Convert Variable array to Record for internal use
 */
const variablesToRecord = (
  variables?: Variable[],
): Record<string, any> | undefined => {
  if (!variables || variables.length === 0) return undefined;
  return variables.reduce(
    (acc, v) => {
      acc[v.key] = v.value;
      return acc;
    },
    {} as Record<string, any>,
  );
};

/**
 * Convert Record to Variable array for API
 */
const recordToVariables = (
  record?: Record<string, any>,
): Variable[] | undefined => {
  if (!record || Object.keys(record).length === 0) return undefined;
  return Object.entries(record).map(([key, value]) => {
    const inferredVarType = inferProjectVariableType(value, key);
    const type = toGqlParameterType(
      inferredVarType,
    ) as unknown as Variable["type"];
    if (!type) {
      throw new Error(`Unable to infer type for variable "${key}"`);
    }
    return { key, type, value };
  });
};

export const useTriggerWorkflowVariables = (initialVariables?: Variable[]) => {
  // Convert Variable[] to Record for internal manipulation
  const initialRecord = variablesToRecord(initialVariables);
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
    initialRecord,
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
  const getVariablesToSave = useCallback((): Variable[] | undefined => {
    if (!workflowVariablesObject || !deploymentDefaultVariables) {
      return recordToVariables(workflowVariablesObject);
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

    return recordToVariables(customizedVars);
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
