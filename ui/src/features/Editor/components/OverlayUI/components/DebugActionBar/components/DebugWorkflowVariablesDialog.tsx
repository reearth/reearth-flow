import {
  ArrowUDownLeftIcon,
  ChalkboardTeacherIcon,
  PencilLineIcon,
} from "@phosphor-icons/react";
import { ColumnDef } from "@tanstack/react-table";
import { useCallback, useMemo, useRef, useState } from "react";

import {
  DataTable as Table,
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogHeader,
  DialogTitle,
  Button,
  DialogFooter,
  IconButton,
  VariableRow,
} from "@flow/components";
import AssetsDialog from "@flow/features/AssetsDialog";
import CmsIntegrationDialog from "@flow/features/CmsIntegrationDialog";
import {
  useAwarenessEcho,
  useEchoFocus,
  useEditorContext,
} from "@flow/features/Editor/editorContext";
import { useT } from "@flow/lib/i18n";
import {
  debugVariableEchoKey,
  ECHO_KEYS,
  useDebugVarSession,
} from "@flow/lib/yjs";
import { AnyWorkflowVariable, Asset } from "@flow/types";
import { awarenessEchoStyles } from "@flow/utils";

/**
 * Wraps a variable's editor so the cell rings while another user has it
 * focused. Own component because each row needs its own subscription.
 */
const EchoVariableCell: React.FC<{
  variable: AnyWorkflowVariable;
  index: number;
  showVariableDialog: boolean;
  onVariableDialogClose: () => void;
  onAssetDialogOpen: (dialog: DialogOptions) => void;
  onDefaultValueChange: (index: number, value: any) => void;
}> = ({
  variable,
  index,
  showVariableDialog,
  onVariableDialogClose,
  onAssetDialogOpen,
  onDefaultValueChange,
}) => {
  const { users, focusProps } = useEchoFocus(debugVariableEchoKey(variable.id));

  return (
    <div style={awarenessEchoStyles(users)} {...focusProps}>
      <VariableRow
        variable={variable}
        index={index}
        key={variable.id}
        showVariableDialog={showVariableDialog}
        onVariableDialogClose={onVariableDialogClose}
        onAssetDialogOpen={onAssetDialogOpen}
        onDefaultValueChange={onDefaultValueChange}
      />
    </div>
  );
};

type Props = {
  debugRunWorkflowVariables?: AnyWorkflowVariable[];
  workflowVariableDefaults?: AnyWorkflowVariable[];
  onDebugRunStart: (variables?: AnyWorkflowVariable[]) => Promise<void>;
  onDialogClose: () => void;
};
type DialogOptions = "assets" | "cms" | undefined;

const DebugWorkflowVariablesDialog: React.FC<Props> = ({
  debugRunWorkflowVariables,
  workflowVariableDefaults,
  onDebugRunStart,
  onDialogClose,
}) => {
  const t = useT();
  const { yDoc } = useEditorContext();

  const baseVariables = useMemo(
    () => debugRunWorkflowVariables ?? [],
    [debugRunWorkflowVariables],
  );

  // Staged values live in the shared doc, not local state: everyone with this
  // dialog open is configuring the same run and should see each other type.
  const { variables, setVariableValue, clearSession } = useDebugVarSession({
    yDoc,
    baseVariables,
  });

  // Other users with this dialog open, from the same echo key the DebugActionBar
  // already broadcasts.
  const dialogUsers = useAwarenessEcho(ECHO_KEYS.debugVariablesDialog);

  const [startingDebugRun, setStartingDebugRun] = useState(false);
  const [showDialog, setShowDialog] = useState<DialogOptions>(undefined);
  const [activeVariableIndex, setActiveVariableIndex] = useState<number>(0);
  const [activeArrayItemIndex, setActiveArrayItemIndex] = useState<number>(0);
  const [showVariableDialog, setShowVariableDialog] = useState(false);

  const variablesRef = useRef(variables);
  variablesRef.current = variables;

  const handleDefaultValueChange = useCallback(
    (index: number, newValue: any) => {
      const variable = variablesRef.current[index];
      if (!variable) return;
      setVariableValue(variable.id, newValue);
    },
    [setVariableValue],
  );

  const handleResetToDefault = useCallback(
    (variableId: string) => {
      const original = workflowVariableDefaults?.find(
        (defaultVariable) => defaultVariable.id === variableId,
      );
      if (!original) return;
      setVariableValue(variableId, original.defaultValue);
    },
    [workflowVariableDefaults, setVariableValue],
  );

  const isAtDefault = useCallback(
    (variable: AnyWorkflowVariable): boolean => {
      const original = workflowVariableDefaults?.find(
        (defaultVariable) => defaultVariable.id === variable.id,
      );
      if (!original) return true;
      return (
        JSON.stringify(variable.defaultValue) ===
        JSON.stringify(original.defaultValue)
      );
    },
    [workflowVariableDefaults],
  );

  const handleAssetDialogOpen = (dialog: DialogOptions) => {
    setShowDialog(dialog);
  };
  const handleDialogClose = () => setShowDialog(undefined);
  const handleVariableDialogOpen = useCallback(
    (variableIndex: number, arrayItemIndex = 0) => {
      setActiveVariableIndex(variableIndex);
      setActiveArrayItemIndex(arrayItemIndex);
      setShowVariableDialog(true);
    },
    [],
  );
  const handleVariableDialogClose = useCallback(
    () => setShowVariableDialog(false),
    [],
  );
  const handleAssetDoubleClick = (asset: Asset) => {
    const variable = variables[activeVariableIndex];
    if (Array.isArray(variable?.defaultValue)) {
      const newArray = [...variable.defaultValue];
      newArray[activeArrayItemIndex] = asset.url;
      handleDefaultValueChange(activeVariableIndex, newArray);
    } else {
      handleDefaultValueChange(activeVariableIndex, asset.url);
    }
    handleVariableDialogClose();
  };

  const handleCmsItemValue = (cmsItemAssetUrl: string) => {
    const variable = variables[activeVariableIndex];
    if (Array.isArray(variable?.defaultValue)) {
      const newArray = [...variable.defaultValue];
      newArray[activeArrayItemIndex] = cmsItemAssetUrl;
      handleDefaultValueChange(activeVariableIndex, newArray);
    } else {
      handleDefaultValueChange(activeVariableIndex, cmsItemAssetUrl);
    }
    handleDialogClose();
    handleVariableDialogClose();
  };

  const handleDebugRunStart = async () => {
    setStartingDebugRun(true);
    await onDebugRunStart(variables);
    setStartingDebugRun(false);
    clearSession();
    onDialogClose();
  };

  const handleCancel = () => {
    // Only the last participant clears — bailing out while someone else is
    // still editing must not wipe their staged values.
    if (dialogUsers.length === 0) clearSession();
    onDialogClose();
  };

  const columns: ColumnDef<AnyWorkflowVariable>[] = useMemo(
    () => [
      {
        accessorKey: "name",
        header: t("Name"),
      },
      {
        accessorKey: "type",
        header: t("Type"),
      },
      {
        accessorKey: "defaultValue",
        header: t("Default Value"),
        cell: ({ row }) => {
          return (
            <EchoVariableCell
              variable={row.original}
              index={row.index}
              showVariableDialog={
                showVariableDialog && activeVariableIndex === row.index
              }
              onVariableDialogClose={handleVariableDialogClose}
              onAssetDialogOpen={handleAssetDialogOpen}
              onDefaultValueChange={handleDefaultValueChange}
            />
          );
        },
      },
      {
        accessorKey: "required",
        header: t("Required"),
        cell: ({ getValue }) => (getValue() ? t("Yes") : t("No")),
      },
      {
        accessorKey: "public",
        header: t("Public"),
        cell: ({ getValue }) => (getValue() ? t("Yes") : t("No")),
      },
      {
        id: "actions",
        header: t("Actions"),
        cell: ({ row }) => (
          <div>
            <IconButton
              size="sm"
              variant="ghost"
              icon={<PencilLineIcon />}
              tooltipText={t("Edit default value")}
              onClick={() => handleVariableDialogOpen(row.index)}
            />
            <IconButton
              size="sm"
              variant="ghost"
              icon={<ArrowUDownLeftIcon />}
              tooltipText={t("Reset to default")}
              onClick={() => handleResetToDefault(row.original.id)}
              disabled={isAtDefault(row.original)}
            />
          </div>
        ),
        size: 100,
      },
    ],
    [
      activeVariableIndex,
      handleDefaultValueChange,
      handleResetToDefault,
      handleVariableDialogClose,
      handleVariableDialogOpen,
      isAtDefault,
      showVariableDialog,
      t,
    ],
  );

  return (
    <Dialog open disablePointerDismissal onOpenChange={onDialogClose}>
      <DialogContent className="h-[50vh]" size="2xl" position="off-center">
        <div className="flex h-full flex-col">
          <DialogHeader>
            <DialogTitle>
              <div className="flex items-center justify-between gap-2">
                <div className="flex items-center gap-2">
                  <ChalkboardTeacherIcon />
                  {t("Workflow Variables")}
                  {dialogUsers.length > 0 && (
                    <div className="flex items-center -space-x-2">
                      {dialogUsers.slice(0, 3).map((user) => (
                        <div
                          key={user.userName}
                          title={user.userName}
                          className="flex size-6 items-center justify-center rounded-full ring-2 ring-secondary/20"
                          style={{ backgroundColor: user.color }}>
                          <span className="text-xs font-medium text-white select-none">
                            {user.userName.charAt(0).toUpperCase()}
                          </span>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              </div>
            </DialogTitle>
          </DialogHeader>
          <div className="flex h-full min-h-0">
            <DialogContentSection className="flex min-h-0 flex-3 flex-col">
              <DialogContentSection className="min-h-0 flex-1 overflow-hidden">
                <Table
                  columns={columns}
                  data={variables}
                  showOrdering={false}
                />
              </DialogContentSection>
            </DialogContentSection>
          </div>
          <DialogFooter className="flex justify-end gap-2 p-4">
            <Button
              variant="outline"
              disabled={startingDebugRun}
              onClick={handleCancel}>
              {t("Cancel")}
            </Button>
            <Button onClick={handleDebugRunStart} disabled={startingDebugRun}>
              {startingDebugRun ? t("Starting...") : t("Start")}
            </Button>
          </DialogFooter>
        </div>
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

export default DebugWorkflowVariablesDialog;
