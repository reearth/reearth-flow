import JSZip from "jszip";
import { ChangeEvent, useCallback, useRef, useState } from "react";
import { WebsocketProvider } from "y-websocket";
import * as Y from "yjs";

import { config } from "@flow/config";
import { DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";
import { useAuth } from "@flow/lib/auth";
import { useProject } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { YWorkflow } from "@flow/lib/yjs/types";
import { useCurrentWorkspace } from "@flow/stores";
import { ProjectToImport } from "@flow/types";

export default () => {
  const { getAccessToken } = useAuth();
  const t = useT();

  const [currentWorkspace] = useCurrentWorkspace();

  const fileInputRef = useRef<HTMLInputElement>(null);

  const [isProjectImporting, setIsProjectImporting] = useState<boolean>(false);

  const handleProjectImportClick = useCallback(() => {
    fileInputRef.current?.click();
  }, []);

  const { createProject } = useProject();

  const handleProjectFileUpload = useCallback(
    async (e: ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0];
      if (!file) return;

      try {
        setIsProjectImporting(true);

        const zip = await JSZip.loadAsync(file);

        const yDocBinary = await zip.file("ydoc.bin")?.async("uint8array");
        if (!yDocBinary) {
          throw new Error("Missing Y.doc binary data");
        }

        const projectMetaJson = await zip
          .file("projectMeta.json")
          ?.async("string");
        if (!projectMetaJson) {
          throw new Error("Missing project metadata");
        }

        const projectMeta: ProjectToImport = JSON.parse(projectMetaJson);

        if (!projectMeta) return console.error("Missing project metadata");
        if (!currentWorkspace)
          return console.error("Missing current workspace");

        const { project } = await createProject({
          workspaceId: currentWorkspace?.id,
          name: projectMeta.name + t("(import)"),
          description: projectMeta.description,
        });

        if (!project) return console.error("Failed to create project");

        const yDoc = new Y.Doc();
        Y.applyUpdate(yDoc, yDocBinary);

        const { websocket } = config();
        if (websocket && projectMeta) {
          (async () => {
            const token = await getAccessToken();

            const yWebSocketProvider = new WebsocketProvider(
              websocket,
              `${project.id}:${DEFAULT_ENTRY_GRAPH_ID}`,
              yDoc,
              { params: { token } },
            );

            yWebSocketProvider.once("sync", () => {
              const yWorkflows = yDoc.getArray<YWorkflow>("workflows");
              if (!yWorkflows.length) {
                console.warn("Imported project has no workflows");
              }

              setIsProjectImporting(false);
              yWebSocketProvider?.destroy();
            });
          })();
        }
      } catch (error) {
        console.error("Error importing project:", error);
        setIsProjectImporting(false);
      }
    },
    [currentWorkspace, t, createProject, getAccessToken],
  );

  return {
    fileInputRef,
    isProjectImporting,
    setIsProjectImporting,
    handleProjectImportClick,
    handleProjectFileUpload,
  };
};
