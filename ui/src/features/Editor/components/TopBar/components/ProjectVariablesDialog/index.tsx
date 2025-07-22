import {
  ChalkboardTeacherIcon,
  PencilSimpleIcon,
  PlusIcon,
  TrashIcon,
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
  DropdownMenuTrigger,
  IconButton,
  Switch,
} from "@flow/components";
import { Button } from "@flow/components/buttons/BaseButton";
import { useT } from "@flow/lib/i18n";
import { AnyProjectVariable, VarType } from "@flow/types";

import { DefaultValueDisplay, NameInput } from "./components/index";
import useProjectVariablesDialog from "./hooks";
import { ProjectVariablesTable } from "./ProjectVariablesTable";
import VariableEditDialog from "./VariableEditDialog";

type Props = {
  currentProjectVariables?: AnyProjectVariable[];
  onClose: () => void;
  onAdd: (projectVariable: AnyProjectVariable) => Promise<void>;
  onChange: (projectVariable: AnyProjectVariable) => Promise<void>;
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
  // "attribute_name",
  "text",
  "number",
  "choice",
  "file_folder",
  "yes_no",
  "datetime",
  "color",
  // "coordinate_system",
  // "database_connection",
  // "geometry",
  // "message",
  // "password",
  // "reprojection_file",
  // "web_connection",
];

const ProjectVariableDialog: React.FC<Props> = ({
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
    localProjectVariables,
    pendingChanges,
    isSubmitting,
    editingVariable,
    getUserFacingName,
    handleLocalAdd,
    handleLocalUpdate,
    handleDeleteSingle,
    handleReorder,
    handleSubmit,
    handleCancel,
    handleEditVariable,
    handleCloseEdit,
  } = useProjectVariablesDialog({
    currentProjectVariables,
    projectId,
    onClose,
    onAdd,
    onChange,
    onDelete,
    onDeleteBatch,
    onBatchUpdate,
  });

  const columns: ColumnDef<AnyProjectVariable>[] = [
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
            <IconButton
              icon={<TrashIcon size={18} />}
              size="default"
              variant="ghost"
              onClick={(e) => {
                e.stopPropagation();
                handleDeleteSingle(variable.id);
              }}
              tooltipText={t("Delete variable")}
              className="hover:bg-accent"
            />
          </div>
        );
      },
      size: 100,
    },
  ];

  return (
    <>
      <Dialog open onOpenChange={handleCancel}>
        <DialogContent
          className="h-[50vh]"
          size="2xl"
          position="off-center"
          hideCloseButton
          onInteractOutside={(e) => e.preventDefault()}>
          <div className="flex h-full flex-col">
            <DialogHeader>
              <DialogTitle>
                <div className="flex items-center justify-between gap-2">
                  <div className="flex items-center gap-2">
                    <ChalkboardTeacherIcon />
                    {t("Project Variables")}
                  </div>
                  <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                      <Button variant="default" size="sm" className="gap-2">
                        <PlusIcon size={16} />
                        {t("Add Variable")}
                      </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="end">
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
                </div>
              </DialogTitle>
            </DialogHeader>
            <div className="flex h-full min-h-0">
              <DialogContentSection className="flex min-h-0 flex-3 flex-col bg-card">
                <DialogContentSection className="min-h-0 flex-1 overflow-hidden">
                  <ProjectVariablesTable
                    projectVariables={localProjectVariables}
                    columns={columns}
                    onReorder={handleReorder}
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
