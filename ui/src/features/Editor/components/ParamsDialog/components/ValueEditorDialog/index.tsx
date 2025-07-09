import { PencilLineIcon } from "@phosphor-icons/react";
import { useCallback, useState } from "react";

import {
  Button,
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  TextArea,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";

import { FieldContext } from "../../utils/fieldUtils";

type Props = {
  open: boolean;
  fieldContext: FieldContext;
  onClose: () => void;
  onValueSubmit?: (value: any) => void;
};

const ValueEditorDialog: React.FC<Props> = ({
  open,
  fieldContext,
  onClose,
  onValueSubmit,
}) => {
  const t = useT();
  const [value, setValue] = useState(fieldContext.value);

  const handleSubmit = useCallback(() => {
    if (!onValueSubmit) return;

    try {
      // Try to parse as JSON first for complex values
      let parsedValue: any = value;
      if (
        fieldContext?.schema?.type === "object" ||
        fieldContext?.schema?.type === "array"
      ) {
        parsedValue = JSON.parse(value);
      } else if (fieldContext?.schema?.type === "number") {
        parsedValue = Number(value);
      } else if (fieldContext?.schema?.type === "integer") {
        parsedValue = parseInt(value, 10);
      } else if (fieldContext?.schema?.type === "boolean") {
        parsedValue = value === "true";
      }

      onValueSubmit(parsedValue);
      onClose();
    } catch (_error) {
      // If JSON parsing fails, use the raw string value
      onValueSubmit(value);
      onClose();
    }
  }, [value, onValueSubmit, onClose, fieldContext?.schema?.type]);

  const getFieldTypeDisplay = (schema: any) => {
    if (schema?.type) {
      return schema.type;
    }
    if (schema?.format) {
      return schema.format;
    }
    return "text";
  };

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
          <div className="w-[200px] border-r bg-secondary p-4">
            {fieldContext && (
              <div className="space-y-3">
                <div>
                  <h4 className="text-sm font-medium text-foreground/90">
                    {t("Field")}
                  </h4>
                  <p className="text-sm text-foreground/70">
                    {fieldContext.fieldName}
                  </p>
                </div>
                <div>
                  <h4 className="text-sm font-medium text-foreground/90">
                    {t("Type")}
                  </h4>
                  <p className="text-sm text-foreground/70">
                    {getFieldTypeDisplay(fieldContext.schema)}
                  </p>
                </div>
                {fieldContext.schema?.description && (
                  <div>
                    <h4 className="text-sm font-medium text-foreground/90">
                      {t("Description")}
                    </h4>
                    <p className="text-xs text-foreground/60">
                      {fieldContext.schema.description}
                    </p>
                  </div>
                )}
              </div>
            )}
          </div>
          <div className="flex flex-1 flex-col">
            <TextArea
              className="max-h-full flex-1 resize-none rounded-none bg-card focus-visible:ring-0"
              autoFocus
              placeholder={t("Enter value...")}
              value={value}
              onChange={(e) => setValue(e.target.value)}
              spellCheck={false}
              data-testid="value-editor-textarea"
              aria-label={t("Value Editor Text Area")}
              data-placeholder={t("Enter value...")}
            />
            <div className="flex justify-end gap-2 p-2">
              <Button onClick={handleSubmit}>{t("Submit")}</Button>
            </div>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
};

export default ValueEditorDialog;
