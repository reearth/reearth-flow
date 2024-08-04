import { createContext } from "react";

import { Dialog as DialogWrapper } from "@flow/components";
import { useWorkspace } from "@flow/lib/gql";
import { DialogType, useDialogType } from "@flow/stores";
import { Workspace } from "@flow/types";

import { DialogContent } from "./components";

export const DialogContext = createContext<
  { workspaces: Workspace[] | undefined } | undefined
>(undefined);

const Dialog: React.FC = () => {
  const [dialogType, setDialogType] = useDialogType();
  const { useGetWorkspaces } = useWorkspace();

  const handleDialogTypeChange = (type?: DialogType) => {
    setDialogType(type);
  };

  const { workspaces } = useGetWorkspaces();

  return (
    dialogType && (
      <DialogContext.Provider value={{ workspaces }}>
        <DialogWrapper
          open={!!dialogType}
          onOpenChange={(o) => !o && setDialogType(undefined)}
        >
          <DialogContent
            tab={dialogType}
            position={dialogType === "canvas-search" ? "top" : undefined}
            onTabChange={handleDialogTypeChange}
          />
        </DialogWrapper>
      </DialogContext.Provider>
    )
  );
};

export default Dialog;
