import { ChangeEvent, useCallback, useRef, useState } from "react";
import { WebsocketProvider } from "y-websocket";
import * as Y from "yjs";

import { config } from "@flow/config";
import { DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";
import { useAuth } from "@flow/lib/auth";
import { useProject } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { yWorkflowConstructor } from "@flow/lib/yjs/conversions";
import { YWorkflow } from "@flow/lib/yjs/types";
import { useCurrentWorkspace } from "@flow/stores";
import {
  validateWorkflowJson,
  validateWorkflowYaml,
} from "@flow/utils/engineWorkflowValidation";
import {
  deconstructedEngineWorkflow,
  isEngineWorkflow,
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
          if (
            (fileExtension === "json" &&
              validateWorkflowJson(results).isValid) ||
            (fileExtension === "yaml" && validateWorkflowYaml(results).isValid)
          ) {
            setInvalidFile(false);
          } else {
            setInvalidFile(true);
          }
          const resultsObject = JSON.parse(results as string);

          if (currentWorkspace && isEngineWorkflow(resultsObject)) {
            const canvasReadyWorkflows = await deconstructedEngineWorkflow({
              engineWorkflow: resultsObject,
            });
            if (!canvasReadyWorkflows)
              return console.error("Failed to convert workflows");

            const { project } = await createProject({
              workspaceId: currentWorkspace.id,
              name: resultsObject.name + t("(import)"),
              description: resultsObject.description,
            });

            if (!project) return console.error("Failed to create project");

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
                  const yWorkflows = yDoc.getArray<YWorkflow>("workflows");
                  yWorkflows.insert(
                    0,
                    canvasReadyWorkflows.workflows.map((w) =>
                      yWorkflowConstructor(
                        w.id,
                        w.name ?? "undefined",
                        w.nodes,
                        w.edges,
                      ),
                    ),
                  );

                  setIsWorkflowImporting(false);
                  resolve();
                });
              });
              yWebSocketProvider.destroy();
            }
          }
        }
      };

      reader.onerror = (e) => {
        console.error("Error reading file:", e.target?.error);
      };

      reader.readAsText(file);
    },
    [currentWorkspace, t, createProject, getAccessToken],
  );

  return {
    fileInputRef,
    isWorkflowImporting,
    invalidFile,
    setIsWorkflowImporting,
    handleWorkflowImportClick,
    handleWorkflowFileUpload,
  };
};
