import { useCallback, useMemo, useRef, useState } from "react";

import { inferWorkflowVariableType } from "@flow/features/WorkspaceProjects/components/WorkflowImport/inferVariableType";
import type { AnyWorkflowVariable, Deployment, Variable } from "@flow/types";
import type { WorkflowVariable } from "@flow/utils/fromEngineWorkflow/deconstructedEngineWorkflow";

import {
  definitionDefaults,
  missingRequiredVariables,
  staleVariableNames,
} from "./triggerVariables";

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
 * Convert Record to Variable array (domain type)
 */
const recordToVariables = (
  record?: Record<string, any>,
  definitions?: AnyWorkflowVariable[],
): Variable[] | undefined => {
  // A cleared field is no override: the deployment's default applies. Sent as
  // null it would fail the server's conversion to the variable's type.
  const entries = Object.entries(record ?? {}).filter(
    ([, value]) => value !== null && value !== undefined,
  );
  if (entries.length === 0) return undefined;
  return entries.map(([key, value]) => {
    // A variable the deployment defines keeps its type; only one it does not
    // know has its type guessed from the value.
    const type =
      definitions?.find((d) => d.name === key)?.type ??
      inferWorkflowVariableType(value, key);
    return { key, type, value };
  });
};

export const useTriggerWorkflowVariables = (initialVariables?: Variable[]) => {
  // Convert Variable[] to Record for internal manipulation
  const initialRecord = variablesToRecord(initialVariables);

  const [
    openTriggerWorkflowVariablesDialog,
    setOpenTriggerWorkflowVariablesDialog,
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

  // The deployment's saved variable definitions. Unset for a deployment
  // uploaded as a file, whose variables are read from the workflow file.
  const [definitions, setDefinitions] = useState<
    AnyWorkflowVariable[] | undefined
  >(undefined);

  // Bumped on every deployment selection, so a fetch begun for an earlier
  // selection can tell it has been superseded and drop its result.
  const selectionRef = useRef(0);

  // True while the workflow file of a deployment uploaded as a file is being
  // read; its variables are unknown until it settles.
  const [isLoadingVariables, setIsLoadingVariables] = useState(false);

  const handleWorkflowFetch = useCallback(
    async (workflowUrl?: string, hasInitialVariables = false) => {
      const selection = ++selectionRef.current;
      const isStale = () => selection !== selectionRef.current;
      // Nothing from the previously selected deployment may linger to be
      // saved as an override for this one; only the trigger's own values
      // survive until (and unless) this deployment's variables arrive.
      setDeploymentDefaultVariables(undefined);
      setWorkflowVariablesObject(
        hasInitialVariables ? triggerCustomVariables : undefined,
      );
      setPendingWorkflowData(null);
      if (!workflowUrl) {
        setIsLoadingVariables(false);
        return;
      }
      setIsLoadingVariables(true);
      try {
        const response = await fetch(workflowUrl);
        if (isStale() || !response.ok) return;
        const jsonData = await response.json();
        if (isStale()) return;
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
      } finally {
        if (!isStale()) setIsLoadingVariables(false);
      }
    },
    [triggerCustomVariables],
  );

  // Loads the variables of the deployment the trigger runs. A deployment made
  // from a project carries their definitions; one uploaded as a file does
  // not, so its workflow file is read instead.
  const loadDeploymentVariables = useCallback(
    (deployment: Deployment, hasInitialVariables = false) => {
      if (!deployment.parameters?.length) {
        setDefinitions(undefined);
        return handleWorkflowFetch(deployment.workflowUrl, hasInitialVariables);
      }
      selectionRef.current++;
      setIsLoadingVariables(false);
      const defaults = definitionDefaults(deployment.parameters);
      const merged =
        hasInitialVariables && triggerCustomVariables
          ? { ...defaults, ...triggerCustomVariables }
          : defaults;
      setDefinitions(deployment.parameters);
      setDeploymentDefaultVariables(defaults);
      setWorkflowVariablesObject(merged);
      setPendingWorkflowData({
        variables: Object.entries(merged).map(([name, value]) => ({
          name,
          value,
        })),
        workflowName: deployment.projectName ?? deployment.description ?? "",
      });
    },
    [handleWorkflowFetch, triggerCustomVariables],
  );

  const handleVariablesConfirm = useCallback((workflowVariables: any[]) => {
    const variablesObj: Record<string, any> = {};
    workflowVariables.forEach((variable) => {
      variablesObj[variable.name] = variable.defaultValue;
    });
    setWorkflowVariablesObject(variablesObj);
    setOpenTriggerWorkflowVariablesDialog(false);
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

  // Compare current variables with deployment defaults. Returns only the
  // customized variables, an empty list when none remain, or undefined while
  // the variables are not loaded.
  const getVariablesToSave = useCallback((): Variable[] | undefined => {
    if (!workflowVariablesObject || !deploymentDefaultVariables) {
      return recordToVariables(workflowVariablesObject, definitions);
    }

    const customizedVars: Record<string, any> = {};

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
      }
    });

    // An empty list, unlike undefined, tells the server to drop the trigger's
    // saved overrides; undefined leaves them in place.
    return recordToVariables(customizedVars, definitions) ?? [];
  }, [workflowVariablesObject, deploymentDefaultVariables, definitions]);

  const missingRequired = useMemo(
    () =>
      definitions
        ? missingRequiredVariables(definitions, workflowVariablesObject)
        : [],
    [definitions, workflowVariablesObject],
  );
  const staleVariables = useMemo(
    () =>
      definitions
        ? staleVariableNames(definitions, workflowVariablesObject)
        : [],
    [definitions, workflowVariablesObject],
  );

  return {
    definitions,
    missingRequired,
    staleVariables,
    loadDeploymentVariables,
    isLoadingVariables,
    pendingWorkflowData,
    workflowVariablesObject,
    openTriggerWorkflowVariablesDialog,
    setOpenTriggerWorkflowVariablesDialog,
    handleWorkflowFetch,
    handleVariablesConfirm,
    initializeVariables,
    getVariablesToSave,
    deploymentDefaultVariables,
  };
};
