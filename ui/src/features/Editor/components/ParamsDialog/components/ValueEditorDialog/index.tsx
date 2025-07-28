import { PencilLineIcon } from "@phosphor-icons/react";
import { QuestionIcon } from "@phosphor-icons/react/dist/ssr";
import { useCallback, useState } from "react";

import {
  Button,
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  TextArea,
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@flow/components";
import AssetsDialog from "@flow/features/AssetsDialog";
import { useProjectVariables } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { useCurrentProject } from "@flow/stores";
import { Asset } from "@flow/types";

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
  const [showAssets, setShowAssets] = useState(false);

  const [value, setValue] = useState(fieldContext.value);

  const [currentProject] = useCurrentProject();

  const { useGetProjectVariables } = useProjectVariables();

  const { projectVariables } = useGetProjectVariables(currentProject?.id);

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

  const fieldType = getFieldTypeDisplay(fieldContext.schema);

  const handleProjectVariableSet = useCallback((variable: any) => {
    const v = `env.get("${variable.name}")`;
    setValue(v);
  }, []);

  const handleAssetDoubleClick = (asset: Asset) => {
    const v = asset.url;
    setValue?.(v);
  };

  return (
    <>
      <Dialog open={open} onOpenChange={onClose}>
        <DialogContent size="xl">
          <DialogHeader>
            <DialogTitle>
              <div className="flex items-center gap-2">
                <PencilLineIcon weight="thin" />
                {t("Value Editor")} -{" "}
                {fieldContext?.fieldName || t("Unknown Field")}{" "}
                {fieldType ? `(${fieldType})` : ""}
              </div>
            </DialogTitle>
          </DialogHeader>
          <div className="flex h-[400px]">
            <div className="flex w-[200px] flex-col justify-between gap-2 border-r bg-secondary p-4">
              <div className="flex w-full flex-col gap-1">
                <p className="mb-2 text-sm text-muted-foreground">
                  {t("Assets")}
                </p>
                <div className="flex gap-1">
                  <Button variant="outline" onClick={() => setShowAssets(true)}>
                    {t("Asset")}
                  </Button>
                  <Button
                    variant="outline"
                    onClick={() => alert(t("Not implemented yet"))}>
                    {t("CMS")}
                  </Button>
                </div>
              </div>
              <div className="w-full">
                {/* Rhai script stuff here */}
                <p className="mb-2 text-sm text-muted-foreground">
                  {t("Project Variables")}
                </p>
                {projectVariables?.map((variable) => (
                  <Button
                    key={variable.id}
                    variant="ghost"
                    className="w-full justify-start text-left"
                    // disabled={variable.type !== fieldType}
                    onClick={() => handleProjectVariableSet(variable)}>
                    {variable.name} ({variable.type})
                  </Button>
                ))}
              </div>
              <div>
                <Tooltip>
                  <TooltipTrigger>
                    <QuestionIcon />
                  </TooltipTrigger>
                  <TooltipContent
                    className="flex flex-col gap-2"
                    side="top"
                    align="start">
                    <p>{t("For Advanced Users")}</p>
                    <p className="max-w-[200px] text-xs text-muted-foreground">
                      {t(
                        "For people familiar with Rhai, you can write Rhai directly here.",
                      )}
                    </p>
                    <p className="max-w-[200px] text-xs text-muted-foreground">
                      {t(
                        "Furthermore, you can use custom functions to access project variables, such as ",
                      )}{" "}
                      <code>env.get('variable_name')</code>.
                    </p>
                    <p className="max-w-[200px] text-xs text-muted-foreground">
                      {t(
                        "For more information, please refer to the documentation.",
                      )}
                    </p>
                  </TooltipContent>
                </Tooltip>
              </div>
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
      {showAssets && fieldContext && (
        <AssetsDialog
          onDialogClose={() => {
            setShowAssets(false);
          }}
          onAssetDoubleClick={handleAssetDoubleClick}
        />
      )}
    </>
  );
};

export default ValueEditorDialog;
