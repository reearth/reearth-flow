import { GearFineIcon } from "@phosphor-icons/react";
import { RJSFSchema } from "@rjsf/utils";
import { useReactFlow } from "@xyflow/react";
import { memo, useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useY } from "react-yjs";
import { Doc, Map as YMap } from "yjs";

import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@flow/components";
import { useEditorContext } from "@flow/features/Editor/editorContext";
import { useT } from "@flow/lib/i18n";
import type { AwarenessUser, Node } from "@flow/types";

import {
  ParamEditor,
  ValueEditorDialog,
  PythonEditorDialog,
  FlowExprEditorDialog,
  type CodeValue,
} from "./components";
import { FieldContext, getValueAtPath } from "./utils/fieldUtils";
import {
  applyMergedPatch,
  DraftPatch,
  DraftStore,
  rjsfIdToPath,
} from "./utils/paramsAwareness";

type Props = {
  yDoc?: Doc | null;
  users?: Record<string, AwarenessUser>;
  openNode?: Node;
  onOpenNode: (nodeId?: string) => void;
  onDataSubmit?: (
    nodesToChange: {
      nodeId: string;
      updatedParams: any;
      updatedCustomizations: any;
      paramsSchema?: RJSFSchema;
    }[],
  ) => void;
  onWorkflowRename?: (id: string, name: string) => void;
  onParamFieldFocus?: (fieldId: string | null) => void;
};

const ParamsDialog: React.FC<Props> = ({
  yDoc,
  users = {},
  openNode,
  onOpenNode,
  onDataSubmit,
  onWorkflowRename,
  onParamFieldFocus,
}) => {
  const t = useT();
  const { isLocked } = useEditorContext();
  const clientId = String(yDoc?.clientID ?? "local");

  const [openValueEditor, setOpenValueEditor] = useState(false);
  const [openPythonEditor, setOpenPythonEditor] = useState(false);
  const [openFlowExprEditor, setOpenFlowExprEditor] = useState(false);
  const [valueEditorContext, setValueEditorContext] = useState<
    FieldContext | undefined
  >(undefined);
  const [pythonEditorContext, setPythonEditorContext] = useState<
    FieldContext | undefined
  >(undefined);
  const [flowExprEditorContext, setFlowExprEditorContext] = useState<
    FieldContext | undefined
  >(undefined);

  const yDrafts = useMemo(() => yDoc?.getMap<any>("paramDrafts"), [yDoc]);
  const rawDrafts = useY(yDrafts ?? new YMap()) as DraftStore;

  const nodeDrafts = openNode?.id ? rawDrafts[openNode.id] : undefined;

  const currentParams = useMemo(() => {
    if (!openNode) return undefined;
    return applyMergedPatch(openNode.data.params, nodeDrafts, "paramsPatch");
  }, [openNode, nodeDrafts]);

  const currentCustomizations = useMemo(() => {
    if (!openNode) return undefined;
    return applyMergedPatch(
      openNode.data.customizations,
      nodeDrafts,
      "customizationsPatch",
    );
  }, [openNode, nodeDrafts]);

  const setMyDraft = useCallback(
    (nodeId: string, updater: (existing: DraftPatch) => DraftPatch) => {
      const existingNodeDrafts = rawDrafts[nodeId] ?? {};
      const existingMyDraft = existingNodeDrafts[clientId] ?? {};
      const nextMyDraft = updater(existingMyDraft);

      yDrafts?.set(nodeId, {
        ...existingNodeDrafts,
        [clientId]: nextMyDraft,
      });
    },
    [rawDrafts, yDrafts, clientId],
  );

  const removeMyDraft = useCallback(
    (nodeId: string) => {
      const existingNodeDrafts = rawDrafts[nodeId];
      if (!existingNodeDrafts) return;

      const { [clientId]: _removed, ...remainingDrafts } = existingNodeDrafts;

      if (Object.keys(remainingDrafts).length === 0) {
        yDrafts?.delete(nodeId);
      } else {
        yDrafts?.set(nodeId, remainingDrafts);
      }
    },
    [rawDrafts, yDrafts, clientId],
  );

  const updateMyFieldPatch = useCallback(
    (
      nodeId: string,
      patchKey: "paramsPatch" | "customizationsPatch",
      path: string,
      value: any,
    ) => {
      setMyDraft(nodeId, (existing) => ({
        ...existing,
        [patchKey]: {
          ...(existing[patchKey] ?? {}),
          [path]: {
            value,
            updatedAt: Date.now(),
          },
        },
      }));
    },
    [setMyDraft],
  );

  const handleUpdate = useCallback(
    async (
      id: string,
      _updatedParams: any,
      _updatedCustomizations: any,
      paramsSchema?: RJSFSchema,
    ) => {
      if (!openNode || openNode.id !== id) return;

      const latestNodeDrafts = rawDrafts[id] ?? {};

      const updatedParams = applyMergedPatch(
        openNode.data.params,
        latestNodeDrafts,
        "paramsPatch",
      );

      const updatedCustomizations = applyMergedPatch(
        openNode.data.customizations,
        latestNodeDrafts,
        "customizationsPatch",
      );

      yDoc?.transact(() => {
        onDataSubmit?.([
          {
            nodeId: id,
            updatedParams,
            updatedCustomizations,
            paramsSchema,
          },
        ]);
      }, "params");

      removeMyDraft(id);
      onOpenNode();
    },
    [openNode, rawDrafts, onDataSubmit, yDoc, removeMyDraft, onOpenNode],
  );

  const handleMigrate = useCallback(
    (id: string, newParams: any, paramsSchema?: RJSFSchema) => {
      if (!openNode || openNode.id !== id) return;

      const latestNodeDrafts = rawDrafts[id] ?? {};
      const updatedCustomizations = applyMergedPatch(
        openNode.data.customizations,
        latestNodeDrafts,
        "customizationsPatch",
      );

      yDoc?.transact(() => {
        onDataSubmit?.([
          {
            nodeId: id,
            updatedParams: newParams,
            updatedCustomizations,
            paramsSchema,
          },
        ]);
      }, "params");

      removeMyDraft(id);
      // Dialog stays open — user reviews migrated values in normal editor and saves explicitly
    },
    [openNode, rawDrafts, onDataSubmit, yDoc, removeMyDraft],
  );

  const { getViewport, setViewport } = useReactFlow();

  const previousViewportRef = useRef<{
    x: number;
    y: number;
    zoom: number;
  } | null>(null);

  useEffect(() => {
    if (openNode && !previousViewportRef.current) {
      const { x, y, zoom } = getViewport();
      previousViewportRef.current = { x, y, zoom };
    } else if (!openNode && previousViewportRef.current) {
      setViewport(previousViewportRef.current, { duration: 400 });
      previousViewportRef.current = null;
    }
  }, [setViewport, getViewport, openNode]);

  const previousNodeIdRef = useRef<string | undefined>(undefined);

  useEffect(() => {
    const previousNodeId = previousNodeIdRef.current;
    const currentNodeId = openNode?.id;

    if (previousNodeId && !currentNodeId) {
      removeMyDraft(previousNodeId);
    }

    previousNodeIdRef.current = currentNodeId;
  }, [openNode?.id, removeMyDraft]);

  const nodeIdRef = useRef<string | undefined>(openNode?.id);
  nodeIdRef.current = openNode?.id;
  const removeMyDraftRef = useRef(removeMyDraft);
  removeMyDraftRef.current = removeMyDraft;

  useEffect(() => {
    return () => {
      const nodeId = nodeIdRef.current;
      if (!nodeId) return;
      removeMyDraftRef.current(nodeId);
    };
  }, []);

  const fieldFocusMap = useMemo(() => {
    const map: Record<string, AwarenessUser[]> = {};
    if (!openNode) return map;
    Object.values(users).forEach((user) => {
      if (user.openNodeId === openNode.id && user.focusedParamField) {
        const fieldId = user.focusedParamField;
        if (!map[fieldId]) map[fieldId] = [];

        map[fieldId].push({
          clientId: user.clientId,
          color: user.color,
          userName: user.userName,
        });
      }
    });
    return map;
  }, [users, openNode]);

  const handleParamChange = useCallback(
    (data: any, changedFieldId?: string) => {
      if (!openNode) return;

      const path = rjsfIdToPath(changedFieldId);
      if (!path) return;

      const value = getValueAtPath(data, path.split("."));

      updateMyFieldPatch(openNode.id, "paramsPatch", path, value);
    },
    [openNode, updateMyFieldPatch],
  );

  const handleCustomizationChange = useCallback(
    (data: any, changedFieldId?: string) => {
      if (!openNode) return;

      const path = rjsfIdToPath(changedFieldId);
      if (!path) return;

      const value = getValueAtPath(data, path.split("."));

      updateMyFieldPatch(openNode.id, "customizationsPatch", path, value);
    },
    [openNode, updateMyFieldPatch],
  );

  const applyFieldPatch = (fieldContext: FieldContext, value: any) => {
    if (!fieldContext || !openNode) return;
    const path = Array.isArray(fieldContext.path)
      ? fieldContext.path.join(".")
      : fieldContext.path;
    updateMyFieldPatch(openNode.id, "paramsPatch", path, value);
  };

  const handleValueChange = (value: any) => {
    if (valueEditorContext) applyFieldPatch(valueEditorContext, value);
  };

  const handleFlowExprValueSubmit = (codeValue: CodeValue) => {
    if (flowExprEditorContext)
      applyFieldPatch(flowExprEditorContext, codeValue);
  };

  const handleOpenNode = useCallback(() => {
    onOpenNode();
  }, [onOpenNode]);

  return (
    <>
      <Dialog open={!!openNode} onOpenChange={handleOpenNode}>
        <DialogContent size="2xl">
          <DialogHeader>
            <DialogTitle>
              <div className="flex items-center gap-2">
                <GearFineIcon weight="thin" />
                {t("Action Editor")}
                <div className="flex items-center -space-x-4">
                  {(() => {
                    const nodeUsers = Object.values(users).filter(
                      (user) => user.openNodeId === openNode?.id,
                    );
                    return (
                      <>
                        {nodeUsers.slice(0, 2).map((user) => (
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
                        {nodeUsers.length > 2 && (
                          <div className="z-10 flex h-6 w-6 items-center justify-center rounded-full bg-secondary/90 ring-2 ring-secondary/20">
                            <span className="text-[10px] font-medium text-white">
                              + {nodeUsers.length - 2}
                            </span>
                          </div>
                        )}
                      </>
                    );
                  })()}
                </div>
              </div>
            </DialogTitle>
          </DialogHeader>
          {openNode && (
            <ParamEditor
              readonly={isLocked}
              nodeId={openNode.id}
              nodeMeta={openNode.data}
              nodeType={openNode.type}
              nodeParams={currentParams}
              nodeCustomizations={currentCustomizations}
              fieldFocusMap={fieldFocusMap}
              onParamsUpdate={handleParamChange}
              onCustomizationsUpdate={handleCustomizationChange}
              onUpdate={handleUpdate}
              onMigrate={handleMigrate}
              onWorkflowRename={onWorkflowRename}
              onParamFieldFocus={onParamFieldFocus}
              onValueEditorOpen={(fieldContext) => {
                setValueEditorContext(fieldContext);
                setOpenValueEditor(true);
              }}
              onPythonEditorOpen={(fieldContext) => {
                setPythonEditorContext(fieldContext);
                setOpenPythonEditor(true);
              }}
              onFlowExprEditorOpen={(fieldContext) => {
                setFlowExprEditorContext(fieldContext);
                setOpenFlowExprEditor(true);
              }}
            />
          )}
        </DialogContent>
      </Dialog>
      {valueEditorContext && (
        <ValueEditorDialog
          open={openValueEditor}
          fieldContext={valueEditorContext}
          onClose={() => {
            setOpenValueEditor(false);
            setValueEditorContext(undefined);
          }}
          onValueSubmit={handleValueChange}
        />
      )}
      {pythonEditorContext && (
        <PythonEditorDialog
          open={openPythonEditor}
          fieldContext={pythonEditorContext}
          onClose={() => {
            setOpenPythonEditor(false);
            setPythonEditorContext(undefined);
          }}
          onValueSubmit={(value) => applyFieldPatch(pythonEditorContext, value)}
        />
      )}
      {flowExprEditorContext && (
        <FlowExprEditorDialog
          open={openFlowExprEditor}
          fieldContext={flowExprEditorContext}
          onClose={() => {
            setOpenFlowExprEditor(false);
            setFlowExprEditorContext(undefined);
          }}
          onValueSubmit={handleFlowExprValueSubmit}
        />
      )}
    </>
  );
};

export default memo(ParamsDialog);
