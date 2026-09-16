import { saveAs } from "file-saver";
import JSZip from "jszip";
import { useCallback, useRef, useState } from "react";

import type { ArtifactFile } from "@flow/utils";

const CONCURRENCY = 6;

type ArtifactZipProgress = {
  completed: number;
  total: number;
};

export default () => {
  const [progress, setProgress] = useState<ArtifactZipProgress>();
  const [failedPaths, setFailedPaths] = useState<string[]>();
  const inFlight = useRef(false);

  const downloadAsZip = useCallback(
    async (files: ArtifactFile[], zipName: string) => {
      if (!files.length || inFlight.current) return;
      inFlight.current = true;
      setFailedPaths(undefined);
      setProgress({ completed: 0, total: files.length });

      const zip = new JSZip();
      const failed: string[] = [];
      const queue = [...files];
      let completed = 0;

      const worker = async () => {
        for (let file = queue.shift(); file; file = queue.shift()) {
          try {
            const response = await fetch(file.url);
            if (!response.ok) {
              throw new Error(`${response.status} ${response.statusText}`);
            }
            // JSZip creates the intermediate folders from the path itself.
            zip.file(file.path, await response.blob());
          } catch (error) {
            console.error(`Failed to fetch artifact ${file.path}:`, error);
            failed.push(file.path);
          }
          completed++;
          setProgress({ completed, total: files.length });
        }
      };

      try {
        await Promise.all(
          Array.from({ length: Math.min(CONCURRENCY, files.length) }, worker),
        );

        // Nothing came back — save an empty archive and the failure is silent.
        if (failed.length === files.length) {
          setFailedPaths(failed);
          return;
        }

        const blob = await zip.generateAsync({
          type: "blob",
          compression: "DEFLATE",
        });
        saveAs(blob, zipName);
        setFailedPaths(failed.length ? failed : undefined);
      } finally {
        inFlight.current = false;
        setProgress(undefined);
      }
    },
    [],
  );

  const downloadOne = useCallback(async (file: ArtifactFile) => {
    setFailedPaths(undefined);
    try {
      const response = await fetch(file.url);
      if (!response.ok) {
        throw new Error(`${response.status} ${response.statusText}`);
      }
      saveAs(await response.blob(), file.name);
    } catch (error) {
      console.error(`Failed to download artifact ${file.path}:`, error);
      setFailedPaths([file.path]);
    }
  }, []);

  return {
    downloadAsZip,
    downloadOne,
    progress,
    failedPaths,
    isDownloading: !!progress,
  };
};
