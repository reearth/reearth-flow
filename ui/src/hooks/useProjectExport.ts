import { saveAs } from "file-saver";
import JSZip from "jszip";
import { useCallback, useState } from "react";
import { WebsocketProvider } from "y-websocket";
import * as Y from "yjs";

import { config } from "@flow/config";
import { DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";
import { useProject } from "@flow/lib/gql";
import { yWorkflowConstructor } from "@flow/lib/yjs/conversions";
import { YWorkflow } from "@flow/lib/yjs/types";
import { generateUUID } from "@flow/utils";

export default (projectId?: string) => {
  const [isExporting, setIsExporting] = useState<boolean>(false);

  const { useGetProject } = useProject();

  const { project } = useGetProject(projectId);

  const handleProjectExport = useCallback(async () => {
    if (!project) return;
    setIsExporting(true);
    const yDoc = new Y.Doc();
    const yWorkflows = yDoc.getArray<YWorkflow>("workflows");

    const { websocket } = config();
    let yWebSocketProvider: WebsocketProvider | null = null;

    if (websocket && project.id) {
      yWebSocketProvider = new WebsocketProvider(
        websocket,
        `${project.id}:${DEFAULT_ENTRY_GRAPH_ID}`,
        yDoc,
      );

      yWebSocketProvider.once("sync", async () => {
        if (yWorkflows.length === 0) {
          yDoc.transact(() => {
            const yWorkflow = yWorkflowConstructor(
              DEFAULT_ENTRY_GRAPH_ID,
              "Main Workflow",
            );
            yWorkflows.insert(0, [yWorkflow]);
          });
        }

        const zip = new JSZip();

        const yDocBinary = Y.encodeStateAsUpdate(yDoc);
        zip.file("ydoc.bin", yDocBinary);

        const projectData = {
          id: generateUUID(),
          name: project.name,
          description: project.description,
        };
        zip.file("projectMeta.json", JSON.stringify(projectData, null, 2));

        const zipBlob = await zip.generateAsync({ type: "blob" });
        const date = new Date();
        const timeStamp = [
          date.getFullYear(),
          String(date.getMonth() + 1).padStart(2, "0"),
          String(date.getDate()).padStart(2, "0"),
          String(date.getHours()).padStart(2, "0"),
          String(date.getMinutes()).padStart(2, "0"),
          String(date.getSeconds()).padStart(2, "0"),
        ].join("");
        const zipName = `${project.name}_${timeStamp}.flow.zip`;
        saveAs(zipBlob, zipName);
        setIsExporting(false);

        yWebSocketProvider?.destroy();
      });
    }
  }, [project]);

  return {
    isExporting,
    handleProjectExport,
  };
};
