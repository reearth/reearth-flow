import { GearIcon } from "@phosphor-icons/react";

import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@flow/components";
import { Button } from "@flow/components/buttons/BaseButton";
import AssetsDialog from "@flow/features/AssetsDialog";
import CmsIntegrationDialog from "@flow/features/CmsIntegrationDialog";
import { useT } from "@flow/lib/i18n";
import { AwarenessUser, WorkflowVariable, VarType } from "@flow/types";

import { ArrayEditor } from "./components/ArrayEditor";
// import { AttributeNameEditor } from "./components/AttributeNameEditor";
import { ChoiceEditor } from "./components/ChoiceEditor";
import { ColorEditor } from "./components/ColorEditor";
import { DateTimeEditor } from "./components/DateTimeEditor";
import { DefaultEditor } from "./components/DefaultEditor";
import { NumberEditor } from "./components/NumberEditor";
import { YesNoEditor } from "./components/YesNoEditor";
import useVariableEditDialog from "./hooks";

type Props = {
  isOpen: boolean;
  variable: WorkflowVariable | null;
  editingUsers?: AwarenessUser[];
  onClose: () => void;
  onUpdate: (variable: WorkflowVariable) => void;
  onLiveUpdate?: (variable: WorkflowVariable) => void;
};

const VariableEditDialog: React.FC<Props> = ({
  isOpen,
  variable,
  editingUsers = [],
  onClose,
  onUpdate,
  onLiveUpdate,
}) => {
  const t = useT();

  const {
    localVariable,
    hasChanges,
    showDialog,
    assetUrl,
    handleAssetDoubleClick,
    handleCmsItemValue,
    handleDialogOpen,
    handleDialogClose,
    handleFieldUpdate,
    handleSave,
    handleCancel,
    clearUrl,
  } = useVariableEditDialog({
    variable,
    onClose,
    onUpdate,
    onLiveUpdate,
  });

  if (!localVariable) return null;

  const getOriginalType = (type: VarType): VarType => {
    const typeMapping: Record<string, VarType> = {
      [t("Array")]: "array",
      [t("Attribute Name")]: "attribute_name",
      [t("Choice")]: "choice",
      [t("Color")]: "color",
      [t("Coordinate System")]: "coordinate_system",
      [t("Database Connection")]: "database_connection",
      [t("Date and Time")]: "datetime",
      [t("Geometry")]: "geometry",
      [t("Message")]: "message",
      [t("Number")]: "number",
      [t("Password")]: "password",
      [t("Reprojection File")]: "reprojection_file",
      [t("Text")]: "text",
      [t("Web Connection")]: "web_connection",
      [t("Yes/No")]: "yes_no",
      [t("Unsupported")]: "unsupported",
    };

    return typeMapping[type] || type;
  };

  const originalType = getOriginalType(localVariable.type);

  const renderEditor = () => {
    switch (originalType) {
      case "array":
        return (
          <ArrayEditor
            variable={localVariable}
            assetUrl={assetUrl}
            onUpdate={handleFieldUpdate}
            onDialogOpen={handleDialogOpen}
            clearUrl={clearUrl}
          />
        );
      case "text":
        return (
          <DefaultEditor
            variable={localVariable}
            onUpdate={handleFieldUpdate}
            onDialogOpen={handleDialogOpen}
          />
        );
      case "number":
        return (
          <NumberEditor variable={localVariable} onUpdate={handleFieldUpdate} />
        );
      case "yes_no":
        return (
          <YesNoEditor variable={localVariable} onUpdate={handleFieldUpdate} />
        );
      case "datetime":
        return (
          <DateTimeEditor
            variable={localVariable}
            onUpdate={handleFieldUpdate}
          />
        );
      case "choice":
        return (
          <ChoiceEditor
            variable={localVariable}
            onUpdate={handleFieldUpdate}
            onDialogOpen={handleDialogOpen}
            clearUrl={clearUrl}
            assetUrl={assetUrl}
          />
        );
      case "color":
        return (
          <ColorEditor variable={localVariable} onUpdate={handleFieldUpdate} />
        );
      default:
        return (
          <DefaultEditor
            variable={localVariable}
            onUpdate={handleFieldUpdate}
            onDialogOpen={handleDialogOpen}
          />
        );
    }
  };

  return (
    <Dialog open={isOpen} onOpenChange={handleCancel}>
      <DialogContent size="lg" position="center">
        <DialogHeader>
          <DialogTitle>
            <div className="flex items-center gap-2">
              <GearIcon />
              {t("Edit Variable")} - {localVariable.name}
              {editingUsers.length > 0 && (
                <div className="flex items-center -space-x-2">
                  {editingUsers.slice(0, 3).map((user) => (
                    <div
                      key={user.clientId}
                      className="flex size-6 items-center justify-center rounded-full ring-2 ring-secondary/20"
                      style={{ backgroundColor: user.color || undefined }}
                      title={user.userName}>
                      <span className="text-xs font-medium text-white select-none">
                        {user.userName.charAt(0).toUpperCase()}
                        {user.userName.charAt(1)}
                      </span>
                    </div>
                  ))}
                  {editingUsers.length > 3 && (
                    <div className="z-10 flex size-6 items-center justify-center rounded-full bg-secondary/90 ring-2 ring-secondary/20">
                      <span className="text-[10px] font-medium text-white">
                        +{editingUsers.length - 3}
                      </span>
                    </div>
                  )}
                </div>
              )}
            </div>
          </DialogTitle>
        </DialogHeader>
        <div className="flex-1 overflow-y-auto p-4">{renderEditor()}</div>
        <DialogFooter className="flex justify-end gap-2">
          <Button variant="outline" onClick={handleCancel}>
            {t("Cancel")}
          </Button>
          <Button onClick={handleSave} disabled={!hasChanges}>
            {t("Save Changes")}
          </Button>
        </DialogFooter>
      </DialogContent>
      {showDialog === "assets" && (
        <AssetsDialog
          onDialogClose={handleDialogClose}
          onAssetSelect={handleAssetDoubleClick}
        />
      )}
      {showDialog === "cms" && (
        <CmsIntegrationDialog
          onDialogClose={handleDialogClose}
          onCmsItemValue={handleCmsItemValue}
        />
      )}
    </Dialog>
  );
};

export default VariableEditDialog;
