import JSZip from "jszip";
import { ChangeEvent, useCallback, useRef } from "react";

import { useProjectImport } from "@flow/hooks";
import { useAuth } from "@flow/lib/auth";
import { useCurrentWorkspace } from "@flow/stores";
import { ProjectToImport } from "@flow/types";

export default () => {
  const { getAccessToken } = useAuth();

  const [currentWorkspace] = useCurrentWorkspace();

  const fileInputRefProject = useRef<HTMLInputElement>(null);

  const handleProjectImportClick = useCallback(() => {
    fileInputRefProject.current?.click();
  }, []);

  const { isProjectImporting, handleProjectImport } = useProjectImport();

  const handleProjectFileUpload = useCallback(
    async (e: ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0];
      if (!file) return;

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
      if (!currentWorkspace) return console.error("Missing current workspace");

      try {
        await handleProjectImport({
          projectName: projectMeta.name + " " + "(import)",
          projectDescription: projectMeta.description,
          workspace: currentWorkspace,
          yDocBinary,
          accessToken: await getAccessToken(),
        });
      } catch (error) {
        console.error("Failed to import project:", error);
      }
    },
    [currentWorkspace, getAccessToken, handleProjectImport],
  );

  return {
    isProjectImporting,
    fileInputRefProject,
    handleProjectImportClick,
    handleProjectFileUpload,
  };
};
