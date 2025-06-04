import { saveAs } from "file-saver";
import JSZip from "jszip";
import { useCallback, useState } from "react";
import * as Y from "yjs";
import { Doc } from "yjs";

import { Project } from "@flow/types";
import { generateUUID } from "@flow/utils";

export default () => {
  const [isExporting, setIsExporting] = useState<boolean>(false);

  const handleProjectExport = useCallback(
    async ({ yDoc, project }: { yDoc: Doc | null; project?: Project }) => {
      if (!project || !yDoc) return;

      setIsExporting(true);

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
      const timestamp = [
        date.getFullYear(),
        String(date.getMonth() + 1).padStart(2, "0"),
        String(date.getDate()).padStart(2, "0"),
        String(date.getHours()).padStart(2, "0"),
        String(date.getMinutes()).padStart(2, "0"),
        String(date.getSeconds()).padStart(2, "0"),
      ].join("");
      const zipName = `${project.name}_${timestamp}.flow.zip`;
      saveAs(zipBlob, zipName);
      setIsExporting(false);
    },
    [],
  );

  return {
    isExporting,
    setIsExporting,
    handleProjectExport,
  };
};
