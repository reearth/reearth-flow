import { DownloadIcon } from "@phosphor-icons/react";
import { useState } from "react";

import {
  Button,
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogContentWrapper,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Input,
  Label,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { toArchiveBaseName, toSuggestedArchiveName } from "@flow/utils";

type Props = {
  isOpen: boolean;
  /**
   * Name to suggest, without the extension. The field starts with it reduced
   * to a name that needs no quoting; whatever the user types is kept.
   */
  defaultName: string;
  fileCount: number;
  /** Called with the file name to save under, extension included. */
  onConfirm: (fileName: string) => void;
  onCancel: () => void;
};

/**
 * Asks what to call a ZIP of output files before it is built. Whatever is
 * typed is made safe to save under; the archive keeps the files' folders
 * whatever it is called.
 */
const ArchiveNameDialog: React.FC<Props> = ({
  isOpen,
  defaultName,
  fileCount,
  onConfirm,
  onCancel,
}) => {
  const t = useT();
  const [name, setName] = useState(() => toSuggestedArchiveName(defaultName));

  // Each opening starts from the suggested name, not the last one typed.
  const [wasOpen, setWasOpen] = useState(isOpen);
  if (isOpen !== wasOpen) {
    setWasOpen(isOpen);
    if (isOpen) setName(toSuggestedArchiveName(defaultName));
  }

  const baseName = toArchiveBaseName(name);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (baseName) onConfirm(`${baseName}.zip`);
  };

  return (
    <Dialog open={isOpen} onOpenChange={(open) => !open && onCancel()}>
      <DialogContent size="sm">
        <form className="flex flex-col" onSubmit={handleSubmit}>
          <DialogHeader>
            <DialogTitle>{t("Download as ZIP")}</DialogTitle>
          </DialogHeader>
          <DialogContentWrapper>
            <DialogContentSection>
              <Label htmlFor="archive-name">{t("File name")}</Label>
              <div className="flex items-center gap-2">
                <Input
                  id="archive-name"
                  autoFocus
                  value={name}
                  placeholder={t("File name")}
                  onChange={(e) => setName(e.target.value)}
                  onFocus={(e) => e.currentTarget.select()}
                />
                <span className="text-sm text-muted-foreground">.zip</span>
              </div>
            </DialogContentSection>
            <DialogContentSection>
              <p className="text-xs text-muted-foreground">
                {t(
                  "{{total}} file(s). Folders created by the workflow are kept inside the ZIP.",
                  { total: fileCount },
                )}
              </p>
            </DialogContentSection>
          </DialogContentWrapper>
          <DialogFooter className="flex justify-end gap-2">
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={onCancel}>
              {t("Cancel")}
            </Button>
            <Button type="submit" size="sm" disabled={!baseName}>
              <DownloadIcon size={16} />
              {t("Download")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
};

export { ArchiveNameDialog };
