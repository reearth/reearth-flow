import { GearIcon, PencilIcon } from "@phosphor-icons/react";

import {
  ColorDefaultValueInput,
  AssetDefaultSelectionInput,
  DateTimeDefaultValueInput,
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogHeader,
  DialogTitle,
  IconButton,
  Input,
  NumberDefaultValueInput,
  Switch,
  TextArea,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import type { AnyWorkflowVariable, TriggerVariableConfig } from "@flow/types";

import VariableArrayInput from "./VariableArrayInput";
import VariableChoiceInput from "./VariableChoiceInput";

type Props = {
  variable: TriggerVariableConfig | AnyWorkflowVariable;
  index: number;
  showVariableDialog?: boolean;
  onVariableDialogOpen?: (
    variableIndex: number,
    arrayItemIndex?: number,
  ) => void;
  onVariableDialogClose?: () => void;
  onDefaultValueChange: (index: number, newValue: any) => void;
  onAssetDialogOpen: (dialog: "assets" | "cms") => void;
};

const VariableRow: React.FC<Props> = ({
  variable,
  index,
  showVariableDialog,
  onVariableDialogOpen,
  onVariableDialogClose,
  onDefaultValueChange,
  onAssetDialogOpen,
}) => {
  const t = useT();

  switch (variable.type) {
    case "array":
      return (
        <VariableArrayInput
          value={
            Array.isArray(variable.defaultValue) ? variable.defaultValue : []
          }
          onChange={(newValue) => onDefaultValueChange(index, newValue)}
          showVariableDialog={showVariableDialog}
          onVariableDialogOpen={(arrayItemIndex) =>
            onVariableDialogOpen?.(index, arrayItemIndex)
          }
          onVariableDialogClose={onVariableDialogClose}
          onAssetDialogOpen={onAssetDialogOpen}
        />
      );
    case "yes_no":
      return (
        <div className="flex items-center space-x-3">
          <span className="text-sm font-medium">
            {variable.defaultValue ? t("Yes") : t("No")}
          </span>
          <Switch
            checked={Boolean(variable.defaultValue)}
            onCheckedChange={(checked) => onDefaultValueChange(index, checked)}
          />
        </div>
      );
    case "number":
      return (
        <NumberDefaultValueInput
          id={`default-${index}`}
          variable={variable}
          onDefaultValueChange={(newValue) =>
            onDefaultValueChange(index, newValue)
          }
        />
      );
    case "choice":
      if (
        "config" in variable &&
        variable.config &&
        "choices" in variable.config
      ) {
        const rawChoices = variable.config.choices;
        const choices = rawChoices.map((choice: any) => {
          if (typeof choice === "string") {
            return { value: choice, label: choice };
          }
          return choice;
        });
        return (
          <VariableChoiceInput
            index={index}
            variable={variable}
            choices={choices}
            onDefaultValueChange={onDefaultValueChange}
          />
        );
      }
      return (
        <Input
          id={`default-${index}`}
          type="text"
          value={variable.defaultValue}
          onChange={(e) => {
            onDefaultValueChange(index, e.target.value);
          }}
        />
      );
    case "color":
      return (
        <div className="flex items-center gap-2">
          <ColorDefaultValueInput
            id={`default-color-${index}`}
            className="h-6 w-6 rounded border p-0 hover:cursor-pointer"
            variable={variable}
            onDefaultValueChange={(newValue) =>
              onDefaultValueChange(index, newValue)
            }
          />
          <span className="font-mono text-sm">{variable.defaultValue}</span>
        </div>
      );
    case "datetime":
      return (
        <DateTimeDefaultValueInput
          id={`default-${index}`}
          variable={variable}
          onDefaultValueChange={(newValue) =>
            onDefaultValueChange(index, newValue)
          }
        />
      );

    case "text":
      return (
        <div>
          <div className="flex items-center">
            {typeof variable.defaultValue === "string" &&
            variable.defaultValue.length > 50 ? (
              <TextArea
                id={`default-${index}`}
                value={variable.defaultValue}
                onChange={(e) => {
                  onDefaultValueChange(index, e.target.value);
                }}
                className="min-h-[60px]"
              />
            ) : (
              <Input
                id={`default-${index}`}
                type="text"
                value={variable.defaultValue}
                onChange={(e) => {
                  onDefaultValueChange(index, e.target.value);
                }}
              />
            )}
            <div className="flex items-center gap-0">
              <IconButton
                icon={<PencilIcon />}
                onClick={() => onVariableDialogOpen?.(index, undefined)}
                className="ml-2"
              />
            </div>
          </div>
          {showVariableDialog && (
            <Dialog open onOpenChange={onVariableDialogClose}>
              <DialogContent
                size="lg"
                position="center"
                className="p-2"
                onInteractOutside={(e) => e.preventDefault()}>
                <DialogHeader>
                  <DialogTitle>
                    <div className="flex items-center justify-between gap-2">
                      <div className="flex items-center gap-2">
                        <GearIcon />
                        {t("Workflow Variables")}
                      </div>
                    </div>
                  </DialogTitle>
                </DialogHeader>
                <div className="flex h-full min-h-0">
                  <DialogContentSection className="flex-1 overflow-y-auto p-4">
                    <AssetDefaultSelectionInput
                      variable={variable}
                      onDefaultValueChange={(newValue) => {
                        onDefaultValueChange(index, newValue);
                        onVariableDialogClose?.();
                      }}
                      onDialogOpen={onAssetDialogOpen}
                    />
                  </DialogContentSection>
                </div>
              </DialogContent>
            </Dialog>
          )}
        </div>
      );

    default:
      console.error(
        `Unsupported variable type '${variable.type}' in Variable Row (index: ${index}).`,
      );
      return (
        <div className="text-sm font-semibold text-red-600">
          {t("Unsupported variable type")}:{" "}
          <span className="font-mono">{variable.type}</span>
        </div>
      );
  }
};

export { VariableRow };
