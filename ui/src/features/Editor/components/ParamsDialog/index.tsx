import { GearFineIcon } from "@phosphor-icons/react";
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
import { useT } from "@flow/lib/i18n";
import type { AwarenessUser, Node } from "@flow/types";

import {
  ParamEditor,
  ValueEditorDialog,
  PythonEditorDialog,
} from "./components";
import { FieldContext, setValueAtPath } from "./utils/fieldUtils";
import {
  applyMergedPatch,
  diffToPatch,
  DraftPatch,
  DraftStore,
} from "./utils/paramsAwareness";

type Props = {
  readonly?: boolean;
  yDoc?: Doc | null;
  users?: Record<string, AwarenessUser>;
  openNode?: Node;
  onOpenNode: (nodeId?: string) => void;
  onDataSubmit?: (
    nodesToChange: {
      nodeId: string;
      updatedParams: any;
      updatedCustomizations: any;
    }[],
  ) => void;
  onWorkflowRename?: (id: string, name: string) => void;
  onParamFieldFocus?: (fieldId: string | null) => void;
};

const ParamsDialog: React.FC<Props> = ({
  readonly,
  yDoc,
  users = {},
  openNode,
  onOpenNode,
  onDataSubmit,
  onWorkflowRename,
  onParamFieldFocus,
}) => {
  const t = useT();
  const clientId = String(yDoc?.clientID ?? "local");

  const [openValueEditor, setOpenValueEditor] = useState(false);
  const [openPythonEditor, setOpenPythonEditor] = useState(false);
  const [currentFieldContext, setCurrentFieldContext] = useState<
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

  const handleUpdate = useCallback(
    async (id: string, _updatedParams: any, _updatedCustomizations: any) => {
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

      await Promise.resolve(
        onDataSubmit?.([{ nodeId: id, updatedParams, updatedCustomizations }]),
      );

      removeMyDraft(id);
      onOpenNode();
    },
    [openNode, rawDrafts, onDataSubmit, removeMyDraft, onOpenNode],
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

  // Keep refs always up-to-date so the unmount cleanup can use the latest values
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
    (data: any) => {
      if (!openNode) return;

      const base = openNode.data.params ?? {};
      const patch = diffToPatch(base, data);

      setMyDraft(openNode.id, (existing) => ({
        ...existing,
        paramsPatch: patch,
      }));
    },
    [openNode, setMyDraft],
  );

  const handleCustomizationChange = useCallback(
    (data: any) => {
      if (!openNode) return;

      const base = openNode.data.customizations ?? {};
      const patch = diffToPatch(base, data);

      setMyDraft(openNode.id, (existing) => ({
        ...existing,
        customizationsPatch: patch,
      }));
    },
    [openNode, setMyDraft],
  );

  const handleValueChange = (value: any) => {
    if (currentFieldContext && openNode) {
      const current = currentParams || {};
      const newParams = setValueAtPath(
        current,
        currentFieldContext.path,
        value,
      );
      handleParamChange(newParams);
    }
  };

  return (
    <>
      <Dialog open={!!openNode} onOpenChange={() => onOpenNode()}>
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
              readonly={readonly}
              nodeId={openNode.id}
              nodeMeta={openNode.data}
              nodeType={openNode.type}
              nodeParams={currentParams}
              nodeCustomizations={currentCustomizations}
              fieldFocusMap={fieldFocusMap}
              onParamsUpdate={handleParamChange}
              onCustomizationsUpdate={handleCustomizationChange}
              onUpdate={handleUpdate}
              onWorkflowRename={onWorkflowRename}
              onParamFieldFocus={onParamFieldFocus}
              onValueEditorOpen={(fieldContext) => {
                setCurrentFieldContext(fieldContext);
                setOpenValueEditor(true);
              }}
              onPythonEditorOpen={(fieldContext) => {
                setCurrentFieldContext(fieldContext);
                setOpenPythonEditor(true);
              }}
            />
          )}
        </DialogContent>
      </Dialog>
      {currentFieldContext && (
        <ValueEditorDialog
          open={openValueEditor}
          fieldContext={currentFieldContext}
          onClose={() => {
            setOpenValueEditor(false);
            setCurrentFieldContext(undefined);
          }}
          onValueSubmit={handleValueChange}
        />
      )}
      {currentFieldContext && (
        <PythonEditorDialog
          open={openPythonEditor}
          fieldContext={currentFieldContext}
          onClose={() => {
            setOpenPythonEditor(false);
            setCurrentFieldContext(undefined);
          }}
          onValueSubmit={handleValueChange}
        />
      )}
    </>
  );
};

export default memo(ParamsDialog);
