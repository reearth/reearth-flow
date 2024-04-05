import { DialogContent, Dialog as DialogWrapper } from "@flow/components";
import { useDialogAtom } from "@flow/stores";

import {
  KeyboardDialogContent,
  AccountDialogContent,
  SettingsDialogContent,
  WorkspacesDialogContent,
  WorkflowDialogContent,
} from "./components";

const Dialog: React.FC = () => {
  const [dialogType, setDialogType] = useDialogAtom();
  console.log("dialogType", dialogType);
  return (
    dialogType && (
      <DialogWrapper open={!!dialogType} onOpenChange={o => !o && setDialogType(undefined)}>
        <DialogContent>
          {dialogType === "keyboard" ? (
            <KeyboardDialogContent />
          ) : dialogType === "account" ? (
            <AccountDialogContent />
          ) : dialogType === "settings" ? (
            <SettingsDialogContent />
          ) : dialogType === "workspaces" ? (
            <WorkspacesDialogContent />
          ) : dialogType === "workflow" ? (
            <WorkflowDialogContent />
          ) : null}
        </DialogContent>
      </DialogWrapper>
    )
  );
};

export { Dialog };
