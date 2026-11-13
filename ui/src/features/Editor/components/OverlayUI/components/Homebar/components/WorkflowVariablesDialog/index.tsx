import {
  SlidersHorizontalIcon,
  PencilLineIcon,
  PlusIcon,
  TrashIcon,
} from "@phosphor-icons/react";
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
  PresenceAvatar,
} from "@flow/components";
import { Button } from "@flow/components/buttons/BaseButton";
import {
  useEditorContext,
  useIsReadOnly,
} from "@flow/features/Editor/editorContext";
import { useT } from "@flow/lib/i18n";
import type { AppColumnDef } from "@flow/lib/table/features";
import type { AnyWorkflowVariable, AwarenessUser, VarType } from "@flow/types";

import { DefaultValueDisplay, NameInput } from "./components/index";
import useWorkflowVariablesDialog from "./hooks";
import { NameProblemsContext } from "./nameProblemsContext";
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
  const { workflowVarAwareness } = useEditorContext();
  const readonly = useIsReadOnly();

  const {
    workflowVariables,
    hasUnsavedChanges,
    nameProblems,
    canSave,
    isSubmitting,
    editingVariable,
    getUserFacingName,
    handleLocalAdd,
    handleUpdate,
    handleDeleteSingle,
    handleReorder,
    handleSubmit,
    handleCancel,
    handleEditVariable,
    handleCloseEdit,
  } = useWorkflowVariablesDialog({
    currentWorkflowVariables,
    users,
    projectId,
    onClose,
    onAdd,
    onChange,
    onDelete,
    onDeleteBatch,
    onBatchUpdate,
  });

  // Why Save is disabled, if it is because of a name.
  const nameMessages = [
    nameProblems.hasEmpty ? t("Every variable needs a name") : undefined,
    nameProblems.duplicates.length > 0
      ? t("Variable names must be unique: {{names}}", {
          names: nameProblems.duplicates.join(", "),
        })
      : undefined,
    nameProblems.withWhitespace.length > 0
      ? t("Variable names cannot contain spaces: {{names}}", {
          names: nameProblems.withWhitespace.join(", "),
        })
      : undefined,
  ].filter((message): message is string => !!message);

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

  const columns: AppColumnDef<AnyWorkflowVariable>[] = useMemo(
    () => [
      {
        accessorKey: "name",
        header: t("Name"),
        cell: ({ row }) => {
          const variable = row.original;
          return (
            <NameInput
              variable={variable}
              disabled={readonly}
              onUpdate={handleUpdate}
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
          return <DefaultValueDisplay variable={row.original} />;
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
                handleUpdate({ ...row.original, required: !isChecked });
              }}
              disabled={readonly}
            />
          );
        },
      },
      {
        accessorKey: "public",
        header: t("Public"),
        cell: ({ row }) => {
          const variable = row.original;
          return (
            <Switch
              checked={variable.public}
              onCheckedChange={() => {
                handleUpdate({ ...variable, public: !variable.public });
              }}
              disabled={readonly}
            />
          );
        },
      },
      {
        id: "actions",
        header: t("Actions"),
        cell: ({ row }) => {
          const variable = row.original;
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
                disabled={readonly}
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
                disabled={readonly}
              />
            </div>
          );
        },
        size: 100,
      },
    ],
    [
      readonly,
      handleUpdate,
      handleEditVariable,
      handleDeleteSingle,
      workflowVarAwareness,
      t,
    ],
  );

  return (
    <>
      <Dialog open disablePointerDismissal onOpenChange={handleCancel}>
        <DialogContent
          className="h-[50vh]"
          size="2xl"
          position="off-center"
          hideCloseButton>
          <div className="flex h-full flex-col">
            <DialogHeader>
              <DialogTitle>
                <div className="flex items-center justify-between gap-2">
                  <div className="flex items-center gap-2">
                    <SlidersHorizontalIcon />
                    {t("Workflow Variables")}
                    <div className="flex items-center -space-x-4">
                      {dialogUsers.length > 0 && (
                        <>
                          {dialogUsers.slice(0, 2).map((user) => (
                            <div key={user.clientId}>
                              <PresenceAvatar user={user} />
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
                    <DropdownMenuTrigger
                      render={
                        <Button
                          variant="default"
                          size="sm"
                          className="gap-2"
                          disabled={readonly}>
                          <PlusIcon size={16} />
                          {t("Add Variable")}
                        </Button>
                      }
                    />
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
                  <NameProblemsContext.Provider value={nameProblems}>
                    <WorkflowVariablesTable
                      workflowVariables={workflowVariables}
                      columns={columns}
                      onReorder={handleReorder}
                      readonly={readonly}
                      variableFocusMap={variableFocusMap}
                      variableEditMap={variableEditMap}
                    />
                  </NameProblemsContext.Provider>
                </DialogContentSection>
              </DialogContentSection>
            </div>
            <DialogFooter className="flex items-center justify-end gap-2 p-4">
              {nameMessages.length > 0 && (
                <div className="mr-auto flex flex-col gap-0.5 text-xs text-destructive">
                  {nameMessages.map((message) => (
                    <p key={message}>{message}</p>
                  ))}
                </div>
              )}
              <Button
                variant="outline"
                onClick={handleCancel}
                disabled={isSubmitting}>
                {t("Cancel")}
              </Button>
              <Button
                onClick={handleSubmit}
                disabled={isSubmitting || !hasUnsavedChanges || !canSave}>
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
        onUpdate={handleUpdate}
        onLiveUpdate={handleUpdate}
        onFieldFocus={(field) =>
          workflowVarAwareness?.onFieldFocus(
            field ? (editingVariable?.id ?? null) : null,
            field,
          )
        }
      />
    </>
  );
};

export default WorkflowVariablesDialog;
