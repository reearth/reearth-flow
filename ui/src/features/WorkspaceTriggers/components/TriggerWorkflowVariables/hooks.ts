import yaml from "js-yaml";
import { ChangeEvent, useCallback, useRef, useState } from "react";
import { WebsocketProvider } from "y-websocket";
import * as Y from "yjs";

import { config } from "@flow/config";
import { DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";
import { useAuth } from "@flow/lib/auth";
import { useProject, useProjectVariables } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { yWorkflowConstructor } from "@flow/lib/yjs/conversions";
import { YWorkflow } from "@flow/lib/yjs/types";
import { useCurrentWorkspace } from "@flow/stores";
import type { AnyProjectVariable } from "@flow/types";
import {
  validateWorkflowJson,
  validateWorkflowYaml,
} from "@flow/utils/engineWorkflowValidation";
import {
  deconstructedEngineWorkflow,
  isEngineWorkflow,
  type WorkflowVariable,
} from "@flow/utils/fromEngineWorkflow/deconstructedEngineWorkflow";

export default () => {
  const { getAccessToken } = useAuth();

  const t = useT();
  const [currentWorkspace] = useCurrentWorkspace();

  const fileInputRef = useRef<HTMLInputElement>(null);

  const [invalidFile, setInvalidFile] = useState<boolean>(false);
  const [isWorkflowImporting, setIsWorkflowImporting] =
    useState<boolean>(false);

  const handleWorkflowImportClick = useCallback(() => {
    fileInputRef.current?.click();
  }, []);

  const { createProject } = useProject();
  const { updateMultipleProjectVariables } = useProjectVariables();

  // State for variable mapping dialog
  const [showVariableMapping, setShowVariableMapping] =
    useState<boolean>(false);
  const [pendingWorkflowData, setPendingWorkflowData] = useState<{
    variables: WorkflowVariable[];
    workflowName: string;
    canvasReadyWorkflows: any;
    resultsObject: any;
  } | null>(null);

  const executeWorkflowImport = useCallback(
    async (
      canvasReadyWorkflows: any,
      resultsObject: any,
      projectVariables?: Omit<
        AnyProjectVariable,
        "id" | "createdAt" | "updatedAt" | "projectId"
      >[],
    ) => {
      if (!currentWorkspace) return;

      const { project } = await createProject({
        workspaceId: currentWorkspace.id,
        name: resultsObject.name + " " + t("(import)"),
        description: resultsObject.description,
      });

      if (!project) return console.error("Failed to create project");

      // Create project variables if provided
      if (projectVariables && projectVariables.length > 0) {
        await updateMultipleProjectVariables({
          projectId: project.id,
          creates: projectVariables.map((pv, index) => ({
            name: pv.name,
            defaultValue: pv.defaultValue,
            type: pv.type,
            required: pv.required,
            publicValue: pv.public,
            index,
            config: pv.config,
          })),
        });
      }

      const yDoc = new Y.Doc();
      const { websocket } = config();

      if (websocket && project) {
        const token = await getAccessToken();
        const yWebSocketProvider = new WebsocketProvider(
          websocket,
          `${project.id}:${DEFAULT_ENTRY_GRAPH_ID}`,
          yDoc,
          { params: { token } },
        );

        await new Promise<void>((resolve) => {
          yWebSocketProvider.once("sync", () => {
            const yWorkflows = yDoc.getMap<YWorkflow>("workflows");
            canvasReadyWorkflows.workflows.forEach((w: any) => {
              const yWorkflow = yWorkflowConstructor(
                w.id,
                w.name ?? "undefined",
                w.nodes,
                w.edges,
              );
              yWorkflows.set(w.id, yWorkflow);
            });

            setIsWorkflowImporting(false);
            resolve();
          });
        });
        yWebSocketProvider.destroy();
      }
    },
    [
      currentWorkspace,
      createProject,
      updateMultipleProjectVariables,
      getAccessToken,
      t,
    ],
  );

  const handleVariableMappingConfirm = useCallback(
    async (
      projectVariables: Omit<
        AnyProjectVariable,
        "id" | "createdAt" | "updatedAt" | "projectId"
      >[],
    ) => {
      if (pendingWorkflowData) {
        await executeWorkflowImport(
          pendingWorkflowData.canvasReadyWorkflows,
          pendingWorkflowData.resultsObject,
          projectVariables,
        );
        setPendingWorkflowData(null);
        setShowVariableMapping(false);
      }
    },
    [pendingWorkflowData, executeWorkflowImport],
  );

  const handleVariableMappingCancel = useCallback(() => {
    setShowVariableMapping(false);
    setIsWorkflowImporting(false);
    setPendingWorkflowData(null);
    // Reset file input so same file can be imported again
    if (fileInputRef.current) {
      fileInputRef.current.value = "";
    }
  }, []);

  const handleWorkflowFileUpload = useCallback(
    async (e: ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0];
      if (!file) return;

      setIsWorkflowImporting(true);

      const fileExtension = file.name.split(".").pop();

      const reader = new FileReader();

      reader.onload = async (e2) => {
        const results = e2.target?.result;

        if (results && typeof results === "string") {
          let resultsObject;
          if (
            fileExtension === "json" &&
            validateWorkflowJson(results).isValid
          ) {
            setInvalidFile(false);
            resultsObject = JSON.parse(results);
          } else if (
            (fileExtension === "yaml" || fileExtension === "yml") &&
            validateWorkflowYaml(results).isValid
          ) {
            setInvalidFile(false);
            resultsObject = yaml.load(results);
          } else {
            setInvalidFile(true);
          }

          if (currentWorkspace && isEngineWorkflow(resultsObject)) {
            const canvasReadyWorkflows = await deconstructedEngineWorkflow({
              engineWorkflow: resultsObject,
            });
            if (!canvasReadyWorkflows)
              return console.error("Failed to convert workflows");

            // Check if workflow has variables that need mapping
            if (
              canvasReadyWorkflows.variables &&
              canvasReadyWorkflows.variables.length > 0
            ) {
              // Show variable mapping dialog
              setPendingWorkflowData({
                variables: canvasReadyWorkflows.variables,
                workflowName: resultsObject.name,
                canvasReadyWorkflows,
                resultsObject,
              });
              setShowVariableMapping(true);
            } else {
              // No variables to map, proceed directly with import
              await executeWorkflowImport(canvasReadyWorkflows, resultsObject);
            }
          }
        }
      };

      reader.onerror = (e) => {
        console.error("Error reading file:", e.target?.error);
      };

      reader.readAsText(file);
    },
    [currentWorkspace, executeWorkflowImport],
  );

  return {
    fileInputRef,
    isWorkflowImporting,
    invalidFile,
    setIsWorkflowImporting,
    handleWorkflowImportClick,
    handleWorkflowFileUpload,
    showVariableMapping,
    pendingWorkflowData,
    handleVariableMappingConfirm,
    handleVariableMappingCancel,
  };
};
