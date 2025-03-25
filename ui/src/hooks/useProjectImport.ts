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
        const { websocket } = config();

        if (websocket && projectMeta) {
          const token = await getAccessToken();

          const yWebSocketProvider = new WebsocketProvider(
            websocket,
            `${project.id}:${DEFAULT_ENTRY_GRAPH_ID}`,
            yDoc,
            { params: { token } },
          );

          await new Promise<void>((resolve) => {
            yWebSocketProvider.once("sync", () => {
              yDoc.transact(() => {
                Y.applyUpdate(yDoc, yDocBinary);
              });

              const yWorkflows = yDoc.getMap<YWorkflow>("workflows");
              if (!yWorkflows.get(DEFAULT_ENTRY_GRAPH_ID)) {
                console.warn("Imported project has no workflows");
              }

              setIsProjectImporting(false);
              resolve();
            });
          });
          yWebSocketProvider?.destroy();
        }
      } catch (error) {
        console.error("Failed to import project:", error);
        setIsProjectImporting(false);
      }
    },
    [createProject, currentWorkspace, getAccessToken, t],
  );

  return {
    isProjectImporting,
    handleProjectImportClick,
    handleProjectFileUpload,
    fileInputRef,
  };
};
