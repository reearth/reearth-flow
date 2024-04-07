import { DialogContent, Dialog as DialogWrapper } from "@flow/components";
import { DialogType, useDialogAtom } from "@flow/stores";

import { SettingsDialogContent } from "./components/Settings";

const Dialog: React.FC = () => {
  const [dialogType, setDialogType] = useDialogAtom();

  const handleDialogTypeChange = (type?: DialogType) => {
    setDialogType(type);
  };

  return (
    dialogType && (
      <DialogWrapper open={!!dialogType} onOpenChange={o => !o && setDialogType(undefined)}>
        <DialogContent>
          {dialogType.includes("settings") ? (
            <SettingsDialogContent tab={dialogType} onTabChange={handleDialogTypeChange} />
          ) : null}
        </DialogContent>
      </DialogWrapper>
    )
  );
};

export { Dialog };
