import { PencilLineIcon } from "@phosphor-icons/react";

import {
  Button,
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  TextArea,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";

type Props = {
  open: boolean;
  currentValue?: string;
  onClose: () => void;
};

const ValueEditorDialog: React.FC<Props> = ({
  open,
  currentValue,
  onClose,
}) => {
  const t = useT();
  return (
    <Dialog open={open} onOpenChange={onClose}>
      <DialogContent size="xl">
        <DialogHeader>
          <DialogTitle>
            <div className="flex items-center gap-2">
              <PencilLineIcon weight="thin" />
              {t("Value Editor")}
            </div>
          </DialogTitle>
        </DialogHeader>
        <div className="flex h-[400px]">
          <div className="w-[200px] border-r bg-secondary">asdf</div>
          <div className="flex flex-1 flex-col">
            <TextArea
              className="max-h-full flex-1 resize-none rounded-none bg-card focus-visible:ring-0"
              autoFocus
              placeholder={t("Enter value...")}
              defaultValue={currentValue}
              onChange={(e) => {
                // Handle value change if needed
                console.log("Value changed:", e.target.value);
              }}
              spellCheck={false}
              data-testid="value-editor-textarea"
              aria-label={t("Value Editor Text Area")}
              data-placeholder={t("Enter value...")}
            />
            <div className="flex justify-end p-2">
              <Button>{t("Submit")}</Button>
            </div>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
};

export default ValueEditorDialog;
