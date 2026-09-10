import {
  DownloadIcon,
  FileIcon,
  FolderIcon,
  WarningIcon,
} from "@phosphor-icons/react";
import { useMemo } from "react";

import { Button } from "@flow/components";
import { useArtifactZipDownload } from "@flow/hooks";
import { useT } from "@flow/lib/i18n";
import { folderArchiveName, groupArtifactsByFolder } from "@flow/utils";
import type { ArtifactFile } from "@flow/utils";

type Props = {
  files: ArtifactFile[];
  /** Base name of the archive "Download all" produces, without the extension. */
  archiveName: string;
};

/**
 * A job's outputs, grouped back into the folders the writers created. A writer
 * with `groupBy` set emits one file per group inside a folder, and every file
 * carries the same shape of name, so without the folder and a way to take them
 * all at once the list is unusable.
 */
const OutputArtifacts: React.FC<Props> = ({ files, archiveName }) => {
  const t = useT();
  const { downloadAsZip, downloadOne, progress, failedPaths, isDownloading } =
    useArtifactZipDownload();

  const folders = useMemo(() => groupArtifactsByFolder(files), [files]);

  if (!files.length) return null;

  return (
    <div className="rounded-md border dark:font-thin">
      <div className="flex items-center justify-between gap-4 px-4 py-2">
        <div className="flex items-center gap-2">
          <p className="text-xl">{t("Output Data")}</p>
          <span className="text-sm font-light text-muted-foreground">
            {t("({{total}} files)", { total: files.length })}
          </span>
        </div>
        <div className="flex items-center gap-3">
          {progress && (
            <span className="text-xs font-light text-muted-foreground">
              {t("Preparing {{completed}} / {{total}}…", {
                completed: progress.completed,
                total: progress.total,
              })}
            </span>
          )}
          <Button
            variant="outline"
            size="sm"
            disabled={isDownloading}
            onClick={() => downloadAsZip(files, `${archiveName}.zip`)}>
            <DownloadIcon />
            {t("Download all as ZIP")}
          </Button>
        </div>
      </div>

      {failedPaths?.length ? (
        <div className="flex items-center gap-2 border-t px-4 py-2 text-warning">
          <WarningIcon />
          <p className="text-sm font-light">
            {t("{{failed}} file(s) could not be downloaded", {
              failed: failedPaths.length,
            })}
          </p>
        </div>
      ) : null}

      <div className="flex flex-col gap-2 border-t p-4">
        {folders.map((folder) => (
          <div key={folder.path || "__root__"} className="flex flex-col">
            {folder.path && (
              <div className="flex items-center justify-between gap-2 py-1">
                <div className="flex min-w-0 items-center gap-2">
                  <FolderIcon className="shrink-0" />
                  <p className="truncate">{folder.path}</p>
                  <span className="shrink-0 text-sm font-light text-muted-foreground">
                    ({folder.files.length})
                  </span>
                </div>
                {/* One writer's grouped output is the unit worth taking: the
                    whole-job archive would drag every other writer along. */}
                <Button
                  variant="ghost"
                  size="sm"
                  disabled={isDownloading}
                  onClick={() =>
                    downloadAsZip(
                      folder.files,
                      folderArchiveName(archiveName, folder.path),
                    )
                  }>
                  <DownloadIcon />
                  ZIP
                </Button>
              </div>
            )}
            <div
              className={folder.path ? "ml-6 flex flex-col" : "flex flex-col"}>
              {folder.files.map((file) => (
                <div
                  key={file.url}
                  className="flex items-center justify-between gap-2 rounded py-1 hover:bg-accent">
                  <div className="flex min-w-0 items-center gap-2">
                    <FileIcon className="shrink-0" />
                    <a
                      href={file.url}
                      target="_blank"
                      rel="noopener noreferrer"
                      title={file.path}
                      className="truncate text-sm font-light text-blue-400 hover:text-blue-300 hover:underline focus-visible:ring-1 focus-visible:ring-ring focus-visible:outline-hidden">
                      {file.name}
                    </a>
                  </div>
                  <Button
                    variant="ghost"
                    size="icon"
                    aria-label={t("Download {{name}}", { name: file.path })}
                    onClick={() => downloadOne(file)}>
                    <DownloadIcon />
                  </Button>
                </div>
              ))}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};

export { OutputArtifacts };
