import { useCallback, useState } from "react";
import { WebsocketProvider } from "y-websocket";
import * as Y from "yjs";
import { Doc } from "yjs";

import { config } from "@flow/config";
import { DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";
import { useProject } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { YWorkflow } from "@flow/lib/yjs/types";
import { Project, ProjectToImport } from "@flow/types";

import { useToast } from "../NotificationSystem/useToast";

type Props = {
  sharedYdoc: Doc | null;
  sharedProject?: Project;
  token?: string;
};
export default ({ sharedYdoc, sharedProject, token }: Props) => {
  const t = useT();
  const { toast } = useToast();

  const [isProjectImporting, setIsProjectImporting] = useState<boolean>(false);
  const [selectedWorkspaceId, setSelectedWorkspaceId] = useState<string | null>(
    null,
  );
  const { createProject } = useProject();
  const handleSelectWorkspace = useCallback((workspaceId: string) => {
    setSelectedWorkspaceId(workspaceId);
  }, []);
  const handleProjectImport = useCallback(async () => {
    try {
      setIsProjectImporting(true);

      if (!sharedYdoc || !sharedProject || !token || !selectedWorkspaceId) {
        throw new Error(
          "Missing either sharedYdoc, sharedProject, token, or selectedWorkspaceId",
        );
      }

      const convertedUpdates = Y.encodeStateAsUpdate(sharedYdoc);
      const projectMeta: ProjectToImport = sharedProject;

      if (!projectMeta) return console.error("Missing project metadata");

      const { project } = await createProject({
        workspaceId: selectedWorkspaceId,
        name: projectMeta.name + t("(import)"),
        description: projectMeta.description,
      });

      if (!project)
        return console.error("Failed to create project from shared project");

      const yDoc = new Y.Doc();
      const { websocket } = config();

      if (websocket && projectMeta) {
        const yWebSocketProvider = new WebsocketProvider(
          websocket,
          `${project.id}:${DEFAULT_ENTRY_GRAPH_ID}`,
          yDoc,
          { params: { token } },
        );

        await new Promise<void>((resolve) => {
          yWebSocketProvider.once("sync", () => {
            yDoc.transact(() => {
              Y.applyUpdate(yDoc, convertedUpdates);
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
      toast({
        title: t("Project Imported"),
        description: t(
          "{{project}} has successfully been imported into {{workspace}}",
          {
            project: projectMeta.name,
            workspace: selectedWorkspaceId,
          },
        ),
      });
    } catch (error) {
      console.error("Failed to import project into selected workspace:", error);
      setIsProjectImporting(false);
    }
  }, [
    createProject,
    sharedYdoc,
    sharedProject,
    selectedWorkspaceId,
    token,
    t,
    toast,
  ]);

  return {
    selectedWorkspaceId,
    isProjectImporting,
    handleProjectImport,
    handleSelectWorkspace,
  };
};
