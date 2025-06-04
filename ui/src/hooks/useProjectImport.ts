import { useCallback, useState } from "react";
import { WebsocketProvider } from "y-websocket";
import * as Y from "yjs";

import { config } from "@flow/config";
import { DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";
import { useProject } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { YWorkflow } from "@flow/lib/yjs/types";
import type { Workspace } from "@flow/types";

export default () => {
  const t = useT();

  const [isProjectImporting, setIsProjectImporting] = useState<boolean>(false);

  const { createProject } = useProject();

  const handleProjectImport = useCallback(
    async ({
      projectName,
      projectDescription,
      workspace,
      yDocBinary,
      accessToken,
    }: {
      projectName: string;
      projectDescription: string;
      workspace: Workspace;
      yDocBinary: Uint8Array<ArrayBufferLike>;
      accessToken: string;
    }) => {
      try {
        setIsProjectImporting(true);

        const { project } = await createProject({
          workspaceId: workspace.id,
          name: projectName + t("(import)"),
          description: projectDescription,
        });

        if (!project) return console.error("Failed to create project");

        const yDoc = new Y.Doc();
        const { websocket } = config();

        if (websocket) {
          const yWebSocketProvider = new WebsocketProvider(
            websocket,
            `${project.id}:${DEFAULT_ENTRY_GRAPH_ID}`,
            yDoc,
            { params: { token: accessToken } },
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
    [createProject, t],
  );

  return {
    isProjectImporting,
    handleProjectImport,
  };
};
