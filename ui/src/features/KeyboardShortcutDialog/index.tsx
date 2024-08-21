import {
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogContentWrapper,
  DialogHeader,
  DialogTitle,
} from "@flow/components";

import { Shortcuts } from "./components";
import useShortcuts from "./useShortcuts";

type Props = {
  isOpen: boolean;
  onOpenChange: (open: boolean) => void;
};

const KeyboardShortcutDialog: React.FC<Props> = ({ isOpen, onOpenChange }) => {
  const { title, editorShortcuts, canvasShortcuts } = useShortcuts();

  return (
    <Dialog open={isOpen} onOpenChange={(o) => onOpenChange(o)}>
      <DialogContent size="lg">
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
        </DialogHeader>
        <DialogContentWrapper>
          <DialogContentSection>
            <p className="text-lg">{editorShortcuts.title}</p>
            <Shortcuts shortcuts={editorShortcuts.shortcuts} />
          </DialogContentSection>
          <DialogContentSection>
            <p className="text-lg">{canvasShortcuts.title}</p>
            <Shortcuts shortcuts={canvasShortcuts.shortcuts} />
          </DialogContentSection>
        </DialogContentWrapper>
      </DialogContent>
    </Dialog>
  );
};

export default KeyboardShortcutDialog;
