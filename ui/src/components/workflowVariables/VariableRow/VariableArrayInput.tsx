import { GearIcon, PencilIcon } from "@phosphor-icons/react";
import { useState } from "react";

import {
  ArrayDefaultItemInput,
  AssetDefaultSelectionInput,
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogHeader,
  DialogTitle,
  IconButton,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";

type VariableArrayInputProps = {
  value: any[];
  onChange: (newValue: any[]) => void;
  showVariableDialog?: boolean;
  onVariableDialogOpen?: (arrayItemIndex: number) => void;
  onVariableDialogClose?: () => void;
  onAssetDialogOpen?: (dialog: "assets" | "cms") => void;
};

export default function VariableArrayInput({
  value,
  onChange,
  showVariableDialog,
  onVariableDialogOpen,
  onVariableDialogClose,
  onAssetDialogOpen,
}: VariableArrayInputProps) {
  const t = useT();
  const [activeItemIndex, setActiveItemIndex] = useState(0);

  const handleUpdateItem = (index: number, newValue: any) => {
    const updatedArray = [...value];
    updatedArray[index] = newValue;
    onChange(updatedArray);
  };

  return (
    <div className="space-y-2">
      {value.map((item, index) => (
        <div key={`${item}-${index}`} className="flex items-center">
          <span className="w-6 text-sm text-muted-foreground">
            {index + 1}.
          </span>
          <div className="flex flex-1 items-center">
            <ArrayDefaultItemInput
              value={item}
              onChange={(v) => handleUpdateItem(index, v)}
            />
            {typeof item === "string" && onVariableDialogOpen && (
              <IconButton
                icon={<PencilIcon />}
                onClick={() => {
                  setActiveItemIndex(index);
                  onVariableDialogOpen(index);
                }}
                className="ml-2"
              />
            )}
          </div>
          {typeof item === "string" &&
            showVariableDialog &&
            activeItemIndex === index && (
              <Dialog open onOpenChange={onVariableDialogClose}>
                <DialogContent
                  size="lg"
                  position="center"
                  className="p-2"
                  onInteractOutside={(e) => e.preventDefault()}>
                  <DialogHeader>
                    <DialogTitle>
                      <div className="flex items-center gap-2">
                        <GearIcon />
                        {t("Workflow Variables")}
                      </div>
                    </DialogTitle>
                  </DialogHeader>
                  <div className="flex h-full min-h-0">
                    <DialogContentSection className="flex-1 overflow-y-auto p-4">
                      <AssetDefaultSelectionInput
                        variable={{ defaultValue: item }}
                        onDefaultValueChange={(newValue) => {
                          handleUpdateItem(index, newValue);
                          onVariableDialogClose?.();
                        }}
                        onDialogOpen={onAssetDialogOpen ?? (() => {})}
                      />
                    </DialogContentSection>
                  </div>
                </DialogContent>
              </Dialog>
            )}
        </div>
      ))}
    </div>
  );
}
