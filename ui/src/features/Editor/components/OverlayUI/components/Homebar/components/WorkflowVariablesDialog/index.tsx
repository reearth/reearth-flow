import {
  ChalkboardTeacherIcon,
  PencilLineIcon,
  PlusIcon,
  TrashIcon,
} from "@phosphor-icons/react";
import { ColumnDef } from "@tanstack/react-table";
import { useMemo } from "react";

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
import { useEditorContext } from "@flow/features/Editor/editorContext";
import { useT } from "@flow/lib/i18n";
import { AnyWorkflowVariable, AwarenessUser, VarType } from "@flow/types";

import { DefaultValueDisplay, NameInput } from "./components/index";
import useWorkflowVariablesDialog from "./hooks";
import VariableEditDialog from "./VariableEditDialog";
import { WorkflowVariablesTable } from "./WorkflowVariablesTable";

type Props = {
  currentWorkflowVariables?: AnyWorkflowVariable[];
  users: Record<string, AwarenessUser>;
  onClose: () => void;
  onAdd: (workflowVariable: AnyWorkflowVariable) => Promise<void>;
  onChange: (workflowVariable: AnyWorkflowVariable) => Promise<void>;
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
      name: string;
      defaultValue: any;
      type: VarType;
      required: boolean;
      publicValue: boolean;
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
  "array",
  "choice",
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

const WorkflowVariablesDialog: React.FC<Props> = ({
  currentWorkflowVariables,
  users,
  projectId,
  onClose,
  onAdd,
  onChange,
  onDelete,
  onDeleteBatch,
  onBatchUpdate,
}) => {
  const t = useT();
  const { isLocked, workflowVarAwareness } = useEditorContext();

  const {
    workflowVariables,
    hasUnsavedChanges,
    isSubmitting,
    editingVariable,
    getUserFacingName,
    handleLocalAdd,
    handleLocalUpdate,
    handleVariableLiveUpdate,
    handleDeleteSingle,
    handleReorder,
    handleSubmit,
    handleCancel,
    handleEditVariable,
    handleCloseEdit,
  } = useWorkflowVariablesDialog({
    currentWorkflowVariables,
    projectId,
    onClose,
    onAdd,
    onChange,
    onDelete,
    onDeleteBatch,
    onBatchUpdate,
  });

  // Users who have the dialog open (shown as avatars in the header)
  const dialogUsers = useMemo(
    () =>
      Object.values(users).filter(
        (u) => u.openWorkflowVariablesDialog === true,
      ),
    [users],
  );

  // Map of variableId → users focused on that row
  const variableFocusMap = useMemo(() => {
    const map: Record<string, AwarenessUser[]> = {};
    Object.values(users).forEach((user) => {
      if (user.openWorkflowVariablesDialog && user.focusedVariableId) {
        const id = user.focusedVariableId;
        if (!map[id]) map[id] = [];
        map[id].push(user);
      }
    });
    return map;
  }, [users]);

  // Map of variableId → users currently in VariableEditDialog for that variable
  const variableEditMap = useMemo(() => {
    const map: Record<string, AwarenessUser[]> = {};
    Object.values(users).forEach((user) => {
      if (user.openWorkflowVariablesDialog && user.editingVariableId) {
        const id = user.editingVariableId;
        if (!map[id]) map[id] = [];
        map[id].push(user);
      }
    });
    return map;
  }, [users]);

  const columns: ColumnDef<AnyWorkflowVariable>[] = useMemo(
    () => [
      {
        accessorKey: "name",
        header: t("Name"),
        cell: ({ row }) => {
          const variable = workflowVariables[row.index];
          return (
            <NameInput
              variable={variable}
              disabled={isLocked}
              onUpdate={handleLocalUpdate}
              onFocus={() =>
                workflowVarAwareness?.onFieldFocus(variable.id, "name")
              }
              onBlur={() => workflowVarAwareness?.onFieldFocus(null, null)}
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
          const variable = workflowVariables[row.index];
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
                const projectVar = { ...workflowVariables[row.index] };
                projectVar.required = !isChecked;
                handleLocalUpdate(projectVar);
              }}
              disabled={isLocked}
            />
          );
        },
      },
      {
        accessorKey: "public",
        header: t("Public"),
        cell: ({ row }) => {
          const variable = workflowVariables[row.index];
          return (
            <Switch
              checked={variable.public}
              onCheckedChange={() => {
                const projectVar = { ...variable };
                projectVar.public = !variable.public;
                handleLocalUpdate(projectVar);
              }}
              disabled={isLocked}
            />
          );
        },
      },
      {
        id: "actions",
        header: t("Actions"),
        cell: ({ row }) => {
          const variable = workflowVariables[row.index];
          return (
            <div className="flex items-center gap-1">
              <IconButton
                icon={<PencilLineIcon size={18} />}
                size="default"
                variant="ghost"
                onClick={(e) => {
                  e.stopPropagation();
                  handleEditVariable(variable);
                }}
                tooltipText={t("Edit default value and advanced options")}
                className="hover:bg-accent"
                disabled={isLocked}
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
                disabled={isLocked}
              />
            </div>
          );
        },
        size: 100,
      },
    ],
    [
      workflowVariables,
      isLocked,
      handleLocalUpdate,
      handleEditVariable,
      handleDeleteSingle,
      workflowVarAwareness,
      t,
    ],
  );

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
                    {t("Workflow Variables")}
                    <div className="flex items-center -space-x-4">
                      {dialogUsers.length > 0 && (
                        <>
                          {dialogUsers.slice(0, 2).map((user) => (
                            <div key={user.clientId}>
                              <div
                                className="flex size-6 items-center justify-center rounded-full ring-2 ring-secondary/20"
                                style={{
                                  backgroundColor: user.color || undefined,
                                }}>
                                <span className="text-xs font-medium text-white select-none">
                                  {user.userName.charAt(0).toUpperCase()}
                                  {user.userName.charAt(1)}
                                </span>
                              </div>
                            </div>
                          ))}
                          {dialogUsers.length > 2 && (
                            <div className="z-10 flex h-6 w-6 items-center justify-center rounded-full bg-secondary/90 ring-2 ring-secondary/20">
                              <span className="text-[10px] font-medium text-white">
                                + {dialogUsers.length - 2}
                              </span>
                            </div>
                          )}
                        </>
                      )}
                    </div>
                  </div>
                  <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                      <Button
                        variant="default"
                        size="sm"
                        className="gap-2"
                        disabled={isLocked}>
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
              <DialogContentSection className="flex min-h-0 flex-3 flex-col">
                <DialogContentSection className="min-h-0 flex-1 overflow-hidden">
                  <WorkflowVariablesTable
                    workflowVariables={workflowVariables}
                    columns={columns}
                    onReorder={handleReorder}
                    variableFocusMap={variableFocusMap}
                    variableEditMap={variableEditMap}
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
                disabled={isSubmitting || !hasUnsavedChanges}>
                {isSubmitting ? t("Saving...") : t("Save Changes")}
              </Button>
            </DialogFooter>
          </div>
        </DialogContent>
      </Dialog>
      <VariableEditDialog
        isOpen={!!editingVariable}
        variable={editingVariable}
        editingUsers={
          editingVariable ? (variableEditMap[editingVariable.id] ?? []) : []
        }
        onClose={handleCloseEdit}
        onUpdate={handleLocalUpdate}
        onLiveUpdate={handleVariableLiveUpdate}
      />
    </>
  );
};

export default WorkflowVariablesDialog;
