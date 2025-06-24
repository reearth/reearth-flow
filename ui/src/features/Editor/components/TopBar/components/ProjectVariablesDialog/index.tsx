import {
  ChalkboardTeacherIcon,
  MinusIcon,
  PlusIcon,
} from "@phosphor-icons/react";
import { ColumnDef } from "@tanstack/react-table";
import { useState, useEffect } from "react";

import {
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuTrigger,
  IconButton,
  Input,
  Switch,
} from "@flow/components";
import { Button } from "@flow/components/buttons/BaseButton";
import { useT } from "@flow/lib/i18n";
import { ProjectVariable, VarType } from "@flow/types";
import { generateUUID, getDefaultValueForProjectVar } from "@flow/utils";

import { ProjectVariablesTable } from "./ProjectVariablesTable";

// Component to handle name input without losing focus
const NameInput: React.FC<{
  variable: ProjectVariable;
  onUpdate: (variable: ProjectVariable) => void;
  placeholder: string;
}> = ({ variable, onUpdate, placeholder }) => {
  const [localValue, setLocalValue] = useState(variable.name);

  // Update local value when variable changes from outside
  useEffect(() => {
    setLocalValue(variable.name);
  }, [variable.name]);

  const handleBlur = () => {
    if (localValue !== variable.name) {
      onUpdate({ ...variable, name: localValue });
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter") {
      e.currentTarget.blur();
    }
  };

  return (
    <Input
      value={localValue}
      onChange={(e) => {
        e.stopPropagation();
        setLocalValue(e.currentTarget.value);
      }}
      onBlur={handleBlur}
      onKeyDown={handleKeyDown}
      onClick={(e) => e.stopPropagation()}
      onFocus={(e) => e.stopPropagation()}
      placeholder={placeholder}
    />
  );
};

type Props = {
  isOpen: boolean;
  currentProjectVariables?: ProjectVariable[];
  onClose: () => void;
  onAdd: (projectVariable: ProjectVariable) => Promise<void>;
  onChange: (projectVariable: ProjectVariable) => Promise<void>;
  onDelete: (id: string) => Promise<void>;
  onDeleteBatch?: (ids: string[]) => Promise<void>;
  onBatchUpdate?: (input: {
    projectId: string;
    creates?: {
      name: string;
      defaultValue: any;
      type: VarType;
      required: boolean;
      publicValue: boolean;
      index?: number;
    }[];
    updates?: {
      paramId: string;
      name?: string;
      defaultValue?: any;
      type?: VarType;
      required?: boolean;
      publicValue?: boolean;
    }[];
    deletes?: string[];
  }) => Promise<void>;
  projectId?: string;
};

type PendingChange =
  | {
      type: "add";
      tempId: string;
      projectVariable: ProjectVariable;
    }
  | {
      type: "update";
      projectVariable: ProjectVariable;
    }
  | {
      type: "delete";
      id: string;
    };

const allVarTypes: VarType[] = [
  "attribute_name",
  "choice",
  "color",
  "coordinate_system",
  "database_connection",
  "datetime",
  "file_folder",
  "geometry",
  "message",
  "number",
  "password",
  "reprojection_file",
  "text",
  "web_connection",
  "yes_no",
  "unsupported",
];

const ProjectVariableDialog: React.FC<Props> = ({
  isOpen,
  currentProjectVariables,
  projectId,
  onClose,
  onAdd,
  onChange,
  onDelete,
  onDeleteBatch,
  onBatchUpdate,
}) => {
  const t = useT();
  const [selectedIndices, setSelectedIndices] = useState<number[]>([]);
  const [localProjectVariables, setLocalProjectVariables] = useState<
    ProjectVariable[]
  >([]);
  const [pendingChanges, setPendingChanges] = useState<PendingChange[]>([]);
  const [isSubmitting, setIsSubmitting] = useState(false);

  // Initialize local state when dialog opens or currentProjectVariables changes
  useEffect(() => {
    if (currentProjectVariables) {
      setLocalProjectVariables([...currentProjectVariables]);
      setPendingChanges([]);
      setSelectedIndices([]);
    }
  }, [currentProjectVariables, isOpen]);

  const handleLocalAdd = (type: VarType) => {
    const tempId = `temp_${generateUUID()}`;
    const newVariable: ProjectVariable = {
      id: tempId,
      name: t("New Project Variable"),
      defaultValue: getDefaultValueForProjectVar(type),
      type,
      required: true,
      public: true,
    };

    setLocalProjectVariables((prev) => [...prev, newVariable]);
    setPendingChanges((prev) => [
      ...prev,
      { type: "add", tempId, projectVariable: newVariable },
    ]);
  };

  const handleLocalUpdate = (updatedVariable: ProjectVariable) => {
    setLocalProjectVariables((prev) =>
      prev.map((variable) =>
        variable.id === updatedVariable.id ? updatedVariable : variable,
      ),
    );

    // Handle pending changes differently for new vs existing variables
    setPendingChanges((prev) => {
      // If this is a new variable (temp ID), update the "add" change
      if (updatedVariable.id.startsWith("temp_")) {
        const existingAddIndex = prev.findIndex(
          (change) =>
            change.type === "add" && change.tempId === updatedVariable.id,
        );

        if (existingAddIndex >= 0) {
          const newChanges = [...prev];
          newChanges[existingAddIndex] = {
            type: "add",
            tempId: updatedVariable.id,
            projectVariable: updatedVariable,
          };
          return newChanges;
        }
      } else {
        // For existing variables, handle as update
        const existingUpdateIndex = prev.findIndex(
          (change) =>
            change.type === "update" &&
            change.projectVariable.id === updatedVariable.id,
        );

        if (existingUpdateIndex >= 0) {
          const newChanges = [...prev];
          newChanges[existingUpdateIndex] = {
            type: "update",
            projectVariable: updatedVariable,
          };
          return newChanges;
        } else {
          return [
            ...prev,
            { type: "update", projectVariable: updatedVariable },
          ];
        }
      }

      return prev;
    });
  };

  const handleLocalDelete = () => {
    if (selectedIndices.length === 0 || !localProjectVariables) return;

    const varsToDelete = selectedIndices.map(
      (index) => localProjectVariables[index],
    );

    setLocalProjectVariables((prev) =>
      prev.filter((_, index) => !selectedIndices.includes(index)),
    );

    varsToDelete.forEach((varToDelete) => {
      if (!varToDelete.id.startsWith("temp_")) {
        setPendingChanges((prev) => [
          ...prev,
          { type: "delete", id: varToDelete.id },
        ]);
      } else {
        setPendingChanges((prev) =>
          prev.filter(
            (change) =>
              !(change.type === "add" && change.tempId === varToDelete.id),
          ),
        );
      }
    });

    setSelectedIndices([]);
  };

  const handleSubmit = async () => {
    setIsSubmitting(true);
    try {
      const addChanges = pendingChanges.filter(
        (change) => change.type === "add",
      );
      const updateChanges = pendingChanges.filter(
        (change) => change.type === "update",
      );
      const deleteChanges = pendingChanges.filter(
        (change) => change.type === "delete",
      );

      if (
        onBatchUpdate &&
        projectId &&
        (addChanges.length > 0 ||
          updateChanges.length > 0 ||
          deleteChanges.length > 0)
      ) {
        const creates = addChanges.map((change) => ({
          name: change.projectVariable.name,
          defaultValue: change.projectVariable.defaultValue,
          type: change.projectVariable.type,
          required: change.projectVariable.required,
          publicValue: change.projectVariable.public,
          index: localProjectVariables.length,
        }));

        const updates = updateChanges.map((change) => ({
          paramId: change.projectVariable.id,
          name: change.projectVariable.name,
          defaultValue: change.projectVariable.defaultValue,
          type: change.projectVariable.type,
          required: change.projectVariable.required,
          publicValue: change.projectVariable.public,
        }));

        const deletes = deleteChanges.map((change) => change.id);

        await onBatchUpdate({
          projectId,
          ...(creates.length > 0 && { creates }),
          ...(updates.length > 0 && { updates }),
          ...(deletes.length > 0 && { deletes }),
        });
      } else {
        for (const change of addChanges) {
          await onAdd(change.projectVariable);
        }

        for (const change of updateChanges) {
          await onChange(change.projectVariable);
        }

        if (deleteChanges.length > 0) {
          const deleteIds = deleteChanges.map((change) => change.id);

          if (onDeleteBatch && deleteChanges.length > 1) {
            await onDeleteBatch(deleteIds);
          } else {
            for (const change of deleteChanges) {
              await onDelete(change.id);
            }
          }
        }
      }

      await new Promise((resolve) => setTimeout(resolve, 100));

      setPendingChanges([]);
      onClose();
    } catch (error) {
      console.error("Failed to submit project variable changes:", error);
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleCancel = () => {
    if (currentProjectVariables) {
      setLocalProjectVariables([...currentProjectVariables]);
    }
    setPendingChanges([]);
    setSelectedIndices([]);
    onClose();
  };

  const getUserFacingName = (type: VarType): string => {
    switch (type) {
      case "attribute_name":
        return t("Attribute Name");
      case "choice":
        return t("Choice");
      case "color":
        return t("Color");
      case "coordinate_system":
        return t("Coordinate System");
      case "database_connection":
        return t("Database Connection");
      case "datetime":
        return t("Date and Time");
      case "file_folder":
        return t("File or Folder");
      case "geometry":
        return t("Geometry");
      case "message":
        return t("Message");
      case "number":
        return t("Number");
      case "password":
        return t("Password");
      case "reprojection_file":
        return t("Reprojection File");
      case "text":
        return t("Text");
      case "web_connection":
        return t("Web Connection");
      case "yes_no":
        return t("Yes/No");
      case "unsupported":
        return t("Unsupported");
      default:
        return t("Unknown");
    }
  };

  const columns: ColumnDef<ProjectVariable>[] = [
    {
      accessorKey: "name",
      header: t("Name"),
      cell: ({ row }) => {
        const variable = localProjectVariables[row.index];
        return (
          <NameInput
            variable={variable}
            onUpdate={handleLocalUpdate}
            placeholder={t("Enter name")}
          />
        );
      },
    },
    {
      accessorKey: "defaultValue",
      header: t("Default Value"),
    },
    {
      accessorKey: "type",
      header: t("Type"),
    },
    {
      accessorKey: "required",
      header: t("Required"),
      cell: ({ row }) => {
        const isChecked = row.getValue("required") as boolean;
        return (
          <Switch
            checked={isChecked}
            onCheckedChange={() => {
              const projectVar = { ...localProjectVariables[row.index] };
              projectVar.required = !isChecked;
              handleLocalUpdate(projectVar);
            }}
          />
        );
      },
    },
    {
      accessorKey: "public",
      header: t("Public"),
      cell: ({ row }) => {
        const variable = localProjectVariables[row.index];
        return (
          <Switch
            checked={variable.public}
            onCheckedChange={() => {
              const projectVar = { ...variable };
              projectVar.public = !variable.public;
              handleLocalUpdate(projectVar);
            }}
          />
        );
      },
    },
  ];

  const handleRowSelect = (selectedIndicesFromTable: number[]) => {
    setSelectedIndices(selectedIndicesFromTable);
  };

  return (
    <Dialog open={isOpen} onOpenChange={handleCancel}>
      <DialogContent className="h-[50vh]" size="2xl" position="off-center">
        <div className="flex h-full flex-col">
          <DialogHeader>
            <DialogTitle>
              <div className="flex items-center gap-2">
                <ChalkboardTeacherIcon />
                {t("Project Variables")}
              </div>
            </DialogTitle>
          </DialogHeader>
          <div className="flex h-full">
            <DialogContentSection className="flex-3 bg-card">
              <DialogContentSection className="flex flex-row items-center gap-2 p-2">
                <DropdownMenu>
                  <DropdownMenuTrigger asChild>
                    <IconButton icon={<PlusIcon />} />
                  </DropdownMenuTrigger>
                  <DropdownMenuContent align="start">
                    <DropdownMenuLabel>
                      {t("Add a new project variable")}
                    </DropdownMenuLabel>
                    <DropdownMenuGroup>
                      {allVarTypes.map((type) => (
                        <DropdownMenuItem
                          key={type}
                          disabled={type === "unsupported"}
                          onClick={() => {
                            handleLocalAdd(type);
                          }}>
                          {getUserFacingName(type)}
                        </DropdownMenuItem>
                      ))}
                    </DropdownMenuGroup>
                  </DropdownMenuContent>
                </DropdownMenu>
                <IconButton
                  icon={<MinusIcon />}
                  onClick={handleLocalDelete}
                  disabled={selectedIndices.length === 0}
                  tooltipText={
                    selectedIndices.length === 0
                      ? t("Select variables to delete")
                      : t("Delete selected variables")
                  }
                />
              </DialogContentSection>
              <DialogContentSection>
                <ProjectVariablesTable
                  projectVariables={localProjectVariables}
                  columns={columns}
                  selectedIndices={selectedIndices}
                  onSelectionChange={handleRowSelect}
                />
              </DialogContentSection>
            </DialogContentSection>
          </div>
          <DialogFooter className="flex justify-end gap-2 p-4">
            <Button
              variant="outline"
              onClick={handleCancel}
              disabled={isSubmitting}>
              {t("Cancel")}
            </Button>
            <Button
              onClick={handleSubmit}
              disabled={isSubmitting || pendingChanges.length === 0}>
              {isSubmitting ? t("Saving...") : t("Save Changes")}
            </Button>
          </DialogFooter>
        </div>
      </DialogContent>
    </Dialog>
  );
};

export default ProjectVariableDialog;
