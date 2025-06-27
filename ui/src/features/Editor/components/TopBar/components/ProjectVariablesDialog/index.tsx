import {
  ChalkboardTeacherIcon,
  MinusIcon,
  PencilSimpleIcon,
  PlusIcon,
} from "@phosphor-icons/react";
import { ColumnDef } from "@tanstack/react-table";

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
  Switch,
} from "@flow/components";
import { Button } from "@flow/components/buttons/BaseButton";
import { useT } from "@flow/lib/i18n";
import { ProjectVariable, VarType } from "@flow/types";

import { DefaultValueDisplay, NameInput } from "./components/index";
import useProjectVariablesDialog from "./hooks";
import { ProjectVariablesTable } from "./ProjectVariablesTable";
import VariableEditDialog from "./VariableEditDialog";

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
    reorders?: {
      paramId: string;
      newIndex: number;
    }[];
  }) => Promise<void>;
  projectId?: string;
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

  const {
    selectedIndices,
    localProjectVariables,
    pendingChanges,
    isSubmitting,
    editingVariable,
    getUserFacingName,
    handleLocalAdd,
    handleLocalUpdate,
    handleLocalDelete,
    handleMoveUp,
    handleMoveDown,
    handleSubmit,
    handleCancel,
    handleRowSelect,
    handleEditVariable,
    handleCloseEdit,
  } = useProjectVariablesDialog({
    isOpen,
    currentProjectVariables,
    projectId,
    onClose,
    onAdd,
    onChange,
    onDelete,
    onDeleteBatch,
    onBatchUpdate,
  });

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
      accessorKey: "type",
      header: t("Type"),
    },
    {
      accessorKey: "defaultValue",
      header: t("Default Value"),
      cell: ({ row }) => {
        const variable = localProjectVariables[row.index];
        return <DefaultValueDisplay variable={variable} />;
      },
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
    {
      id: "actions",
      header: t("Actions"),
      cell: ({ row }) => {
        const variable = localProjectVariables[row.index];
        return (
          <div className="flex items-center gap-1">
            <IconButton
              icon={<PencilSimpleIcon size={18} />}
              size="default"
              variant="ghost"
              onClick={(e) => {
                e.stopPropagation();
                handleEditVariable(variable);
              }}
              tooltipText={t("Edit default value and advanced options")}
              className="hover:bg-accent"
            />
          </div>
        );
      },
      size: 80,
    },
  ];

  return (
    <>
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
                    onMoveUp={handleMoveUp}
                    onMoveDown={handleMoveDown}
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

      <VariableEditDialog
        isOpen={!!editingVariable}
        variable={editingVariable}
        onClose={handleCloseEdit}
        onUpdate={handleLocalUpdate}
      />
    </>
  );
};

export default ProjectVariableDialog;
