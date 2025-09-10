import { DownloadIcon, WarningIcon } from "@phosphor-icons/react";
import { useEffect, useState } from "react";

import {
  Button,
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogContentWrapper,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";

type Props = {
  isOpen: boolean;
  fileName: string;
  fileUrl: string;
  onConfirm: () => void;
  onCancel: () => void;
};

const DownloadConfirmDialog: React.FC<Props> = ({
  isOpen,
  fileName,
  fileUrl,
  onConfirm,
  onCancel,
}) => {
  const t = useT();
  const [fileSize, setFileSize] = useState<string | null>(null);
  const [isLoadingSize, setIsLoadingSize] = useState(false);

  const getRemoteSize = async (url: string): Promise<number | null> => {
    // Try HEAD first
    try {
      const head = await fetch(url, { method: "HEAD" });
      const len = head.headers.get("Content-Length");
      if (len) return parseInt(len, 10);
    } catch (_) {
      console.warn("HEAD request failed, trying GET with Range");
    }

    // Fallback: 1-byte range GET
    try {
      const res = await fetch(url, { headers: { Range: "bytes=0-0" } });
      const cr = res.headers.get("Content-Range"); // e.g., "bytes 0-0/12345"
      if (cr && cr.includes("/")) return parseInt(cr.split("/")[1], 10);
    } catch (_) {
      console.warn("Range GET request failed, unable to determine file size");
    }

    return null;
  };

  useEffect(() => {
    if (isOpen && fileUrl) {
      setIsLoadingSize(true);
      setFileSize(null);

      getRemoteSize(fileUrl)
        .then((bytes) => {
          if (bytes !== null) {
            setFileSize(formatFileSize(bytes));
          } else {
            setFileSize("Size unavailable");
          }
        })
        .catch(() => {
          setFileSize("Size unavailable");
        })
        .finally(() => {
          setIsLoadingSize(false);
        });
    }
  }, [isOpen, fileUrl]);

  const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return "0 Bytes";

    const k = 1024;
    const sizes = ["Bytes", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));

    return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
  };

  return (
    <Dialog open={isOpen} onOpenChange={onCancel}>
      <DialogContent size="sm">
        <DialogHeader className="text-warning">
          <DialogTitle className="flex items-center gap-2">
            <WarningIcon size={20} />
            {t("Confirm Download")}
          </DialogTitle>
        </DialogHeader>
        <DialogContentWrapper>
          <DialogContentSection>
            <p className="text-sm">
              {t("Are you sure you want to download this file?")}
            </p>
          </DialogContentSection>
          <DialogContentSection className="rounded bg-muted/30 p-3">
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <span className="text-sm font-medium">{t("File name")}:</span>
                <span className="font-mono text-sm">{fileName}</span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-sm font-medium">{t("File size")}:</span>
                <span className="text-sm">
                  {isLoadingSize ? t("Loading...") : fileSize || "Unknown"}
                </span>
              </div>
            </div>
          </DialogContentSection>
          <DialogContentSection>
            <p className="text-xs text-muted-foreground">
              {t("Large files may take some time to download.")}
            </p>
          </DialogContentSection>
        </DialogContentWrapper>
        <DialogFooter className="flex justify-end gap-2">
          <Button variant="outline" size="sm" onClick={onCancel}>
            {t("Cancel")}
          </Button>
          <Button size="sm" onClick={onConfirm}>
            <DownloadIcon size={16} />
            {t("Download")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export default DownloadConfirmDialog;