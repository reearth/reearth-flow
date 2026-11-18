import { FileIcon } from "@phosphor-icons/react";

import {
  Dialog,
  DialogContent,
  DialogContentWrapper,
  DialogTitle,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { useCurrentWorkspace } from "@flow/stores";

import WorkspaceAssetsPanel from "./WorkspaceAssetsPanel";

export { default as WorkspaceAssetsPanel } from "./WorkspaceAssetsPanel";

type Props = {
  onDialogClose: () => void;
};

const AssetsDialog: React.FC<Props> = ({ onDialogClose }) => {
  const t = useT();
  const [currentWorkspace] = useCurrentWorkspace();

  return (
    <Dialog open onOpenChange={onDialogClose}>
      <DialogContent className="max-h-[800px] w-full max-w-4xl overflow-hidden">
        <DialogTitle className="flex items-center gap-2">
          <FileIcon />
          {t("Workspace Assets")}
        </DialogTitle>
        <DialogContentWrapper>
          <WorkspaceAssetsPanel workspaceId={currentWorkspace?.id ?? ""} />
        </DialogContentWrapper>
      </DialogContent>
    </Dialog>
  );
};

export default AssetsDialog;
