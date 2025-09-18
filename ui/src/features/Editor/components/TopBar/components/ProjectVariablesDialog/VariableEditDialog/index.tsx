import { GearIcon } from "@phosphor-icons/react";

import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@flow/components";
import { Button } from "@flow/components/buttons/BaseButton";
import { useT } from "@flow/lib/i18n";
import { ProjectVariable, VarType } from "@flow/types";

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
  variable: ProjectVariable | null;
  onClose: () => void;
  onUpdate: (variable: ProjectVariable) => void;
};

const VariableEditDialog: React.FC<Props> = ({
  isOpen,
  variable,
  onClose,
  onUpdate,
}) => {
  const t = useT();

  const {
    localVariable,
    hasChanges,
    handleFieldUpdate,
    handleSave,
    handleCancel,
  } = useVariableEditDialog({
    variable,
    onClose,
    onUpdate,
  });

  if (!localVariable) return null;

  // Determine the original type from the user-facing name
  const getOriginalType = (type: VarType): VarType => {
    const typeMapping: Record<string, VarType> = {
      [t("Array")]: "array",
      [t("Attribute Name")]: "attribute_name",
      [t("Choice")]: "choice",
      [t("Color")]: "color",
      [t("Coordinate System")]: "coordinate_system",
      [t("Database Connection")]: "database_connection",
      [t("Date and Time")]: "datetime",
      [t("File or Folder")]: "file_folder",
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
          <ArrayEditor variable={localVariable} onUpdate={handleFieldUpdate} />
        );
      // case "attribute_name":
      //   return (
      //     <AttributeNameEditor
      //       variable={localVariable}
      //       onUpdate={handleFieldUpdate}
      //     />
      //   );
      case "text":
        return (
          <DefaultEditor
            variable={localVariable}
            onUpdate={handleFieldUpdate}
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
          <ChoiceEditor variable={localVariable} onUpdate={handleFieldUpdate} />
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
    </Dialog>
  );
};

export default VariableEditDialog;
