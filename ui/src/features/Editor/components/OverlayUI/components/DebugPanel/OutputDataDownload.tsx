import {
  CaretDownIcon,
  DownloadIcon,
  FileZipIcon,
  FolderIcon,
} from "@phosphor-icons/react";
import { useCallback, useMemo, useState } from "react";

import {
  Button,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@flow/components";
import { ArchiveNameDialog } from "@flow/features/common";
import { useArtifactZipDownload } from "@flow/hooks";
import { useT } from "@flow/lib/i18n";
import { folderArchiveName, groupArtifactsByFolder } from "@flow/utils";
import type { ArtifactFile } from "@flow/utils";

import DownloadConfirmDialog from "./DownloadConfirmDialog";

type Props = {
  outputData?: ArtifactFile[];
  /** Name "Download all" suggests for the archive, without the extension. */
  archiveName: string;
};

const OutputDataDownload: React.FC<Props> = ({ outputData, archiveName }) => {
  const t = useT();
  const { downloadAsZip, downloadOne, progress, isDownloading } =
    useArtifactZipDownload();
  const [confirmDialog, setConfirmDialog] = useState<{
    isOpen: boolean;
    fileName: string;
    fileUrl: string;
    onConfirm: () => void;
  }>({
    isOpen: false,
    fileName: "",
    fileUrl: "",
    onConfirm: () => {},
  });

  // A writer with `groupBy` set puts one file per group in a folder, so the
  // folders are shown rather than listing every group under a bare hash.
  const folders = useMemo(
    () => (outputData ? groupArtifactsByFolder(outputData) : []),
    [outputData],
  );

  const handleDownload = useCallback(
    (file: ArtifactFile) => {
      setConfirmDialog({
        isOpen: true,
        fileName: file.path,
        fileUrl: file.url,
        onConfirm: () => {
          downloadOne(file);
          setConfirmDialog((prev) => ({ ...prev, isOpen: false }));
        },
      });
    },
    [downloadOne],
  );

  // A ZIP asks for its name instead of confirming a size: a batch has no
  // single URL to size up front.
  const [archiveRequest, setArchiveRequest] = useState<{
    isOpen: boolean;
    files: ArtifactFile[];
    defaultName: string;
  }>({ isOpen: false, files: [], defaultName: "" });

  const handleDownloadZip = useCallback(
    (files: ArtifactFile[], defaultName: string) =>
      setArchiveRequest({ isOpen: true, files, defaultName }),
    [],
  );

  const handleCloseArchiveRequest = useCallback(
    () => setArchiveRequest((prev) => ({ ...prev, isOpen: false })),
    [],
  );

  const handleConfirmArchive = useCallback(
    (fileName: string) => {
      handleCloseArchiveRequest();
      downloadAsZip(archiveRequest.files, fileName);
    },
    [archiveRequest.files, downloadAsZip, handleCloseArchiveRequest],
  );

  const handleCancelDownload = useCallback(() => {
    setConfirmDialog((prev) => ({ ...prev, isOpen: false }));
  }, []);

  const count = outputData?.length || 0;

  return (
    <>
      <DropdownMenu>
        <DropdownMenuTrigger
          disabled={count === 0}
          render={
            <Button
              className="h-[25px] gap-1 px-2 text-xs font-light hover:bg-primary dark:font-thin"
              variant="ghost"
              disabled={count === 0}>
              <FolderIcon size={14} />
              {progress
                ? t("Preparing {{completed}} / {{total}}…", {
                    completed: progress.completed,
                    total: progress.total,
                  })
                : `${t("Output data")} (${count})`}
              <CaretDownIcon size={10} />
            </Button>
          }
        />
        <DropdownMenuContent
          align="start"
          className="max-h-[400px] overflow-y-auto">
          {count > 1 && (
            <>
              <DropdownMenuItem
                disabled={isDownloading}
                onClick={() =>
                  handleDownloadZip(outputData ?? [], archiveName)
                }>
                <FileZipIcon size={16} />
                {t("Download all as ZIP")}
              </DropdownMenuItem>
              <div className="my-1 h-px bg-border" />
            </>
          )}
          {folders.map((folder) => (
            <div key={folder.path || "__root__"}>
              {folder.path && (
                <DropdownMenuItem
                  disabled={isDownloading}
                  onClick={() =>
                    handleDownloadZip(
                      folder.files,
                      folderArchiveName(folder.path),
                    )
                  }>
                  <FileZipIcon size={16} />
                  <span className="truncate">{folder.path}</span>
                  <span className="text-muted-foreground">
                    ({folder.files.length})
                  </span>
                </DropdownMenuItem>
              )}
              {folder.files.map((file) => (
                <DropdownMenuItem
                  key={file.url}
                  className={folder.path ? "pl-6" : undefined}
                  onClick={() => handleDownload(file)}>
                  <DownloadIcon size={16} />
                  <span className="truncate">{file.name}</span>
                </DropdownMenuItem>
              ))}
            </div>
          ))}
        </DropdownMenuContent>
      </DropdownMenu>

      <DownloadConfirmDialog
        isOpen={confirmDialog.isOpen}
        fileName={confirmDialog.fileName}
        fileUrl={confirmDialog.fileUrl}
        onConfirm={confirmDialog.onConfirm}
        onCancel={handleCancelDownload}
      />

      <ArchiveNameDialog
        isOpen={archiveRequest.isOpen}
        defaultName={archiveRequest.defaultName}
        fileCount={archiveRequest.files.length}
        onConfirm={handleConfirmArchive}
        onCancel={handleCloseArchiveRequest}
      />
    </>
  );
};

export default OutputDataDownload;
