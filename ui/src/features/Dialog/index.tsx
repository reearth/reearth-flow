import { Dialog as DialogWrapper } from "@flow/components";
import { DialogType, useDialogAtom } from "@flow/stores";

import { DialogContent } from "./components/Content";

const Dialog: React.FC = () => {
  const [dialogType, setDialogType] = useDialogAtom();

  const handleDialogTypeChange = (type?: DialogType) => {
    setDialogType(type);
  };

  return (
    dialogType && (
      <DialogWrapper open={!!dialogType} onOpenChange={o => !o && setDialogType(undefined)}>
        <DialogContent tab={dialogType} onTabChange={handleDialogTypeChange} />
      </DialogWrapper>
    )
  );
};

export { Dialog };
