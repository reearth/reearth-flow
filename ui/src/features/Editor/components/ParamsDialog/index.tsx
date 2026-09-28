import { GearFineIcon } from "@phosphor-icons/react";
import { useReactFlow } from "@xyflow/react";
import { memo, useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useY } from "react-yjs";
import type { Doc } from "yjs";
import { Map as YMap } from "yjs";

import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@flow/components";
import type { EditorContext as FieldContext } from "@flow/components/SchemaForm";
import { useIsReadOnly } from "@flow/features/Editor/editorContext";
import { useT } from "@flow/lib/i18n";
import type { FlowSchema } from "@flow/lib/schemaForm";
import {
  applyDefaults,
  compile,
  getAtPath,
  parsePathKey,
} from "@flow/lib/schemaForm";
import type { AwarenessUser, Node } from "@flow/types";
import { normalizeParams } from "@flow/utils";

import {
  ParamEditor,
  ValueEditorDialog,
  PythonEditorDialog,
  FlowExprEditorDialog,
} from "./components";
import type { CodeValue } from "./components";
import type { AutocompleteSuggestion } from "./components/ValueEditorDialog/components/flowExprConstants";
import {
  applyMergedPatch,
  changedFieldPath,
  nextSeq,
} from "./utils/paramsAwareness";
import type {
  DraftPatch,
  DraftStore,
  NodeDrafts,
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
      paramsSchema?: FlowSchema;
    }[],
  ) => void;
  onNodeParamsSaved?: (node: Node) => void;
  attributeSuggestions?: AutocompleteSuggestion[];
  onWorkflowRename?: (id: string, name: string) => void;
  onParamFieldFocus?: (fieldId: string | null) => void;
};

const ParamsDialog: React.FC<Props> = ({
  yDoc,
  users = {},
  openNode,
  onOpenNode,
  onDataSubmit,
  onNodeParamsSaved,
  attributeSuggestions,
  onWorkflowRename,
  onParamFieldFocus,
}) => {
  const t = useT();
  const readonly = useIsReadOnly();
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

  /**
   * This node's drafts as the document holds them *now*.
   *
   * `rawDrafts` is a render snapshot, so it is a frame behind: two edits
   * dispatched in one React tick both read the state the render began with,
   * and the second overwrites the first. Keystrokes arrive in exactly that
   * pattern. Yjs is the authority, so every read that precedes a write goes to
   * the map rather than to the snapshot — which also makes the callbacks below
   * stable, since they no longer close over a value that changes every render.
   */
  const liveNodeDrafts = useCallback(
    (nodeId: string): NodeDrafts => (yDrafts?.get(nodeId) ?? {}) as NodeDrafts,
    [yDrafts],
  );

  const setMyDraft = useCallback(
    (
      nodeId: string,
      updater: (existing: DraftPatch, nodeDrafts: NodeDrafts) => DraftPatch,
    ) => {
      const existingNodeDrafts = liveNodeDrafts(nodeId);
      const existingMyDraft = existingNodeDrafts[clientId] ?? {};
      const nextMyDraft = updater(existingMyDraft, existingNodeDrafts);

      yDrafts?.set(nodeId, {
        ...existingNodeDrafts,
        [clientId]: nextMyDraft,
      });
    },
    [liveNodeDrafts, yDrafts, clientId],
  );

  const removeMyDraft = useCallback(
    (nodeId: string) => {
      const existingNodeDrafts = yDrafts?.get(nodeId) as NodeDrafts | undefined;
      if (!existingNodeDrafts) return;

      const { [clientId]: _removed, ...remainingDrafts } = existingNodeDrafts;

      if (Object.keys(remainingDrafts).length === 0) {
        yDrafts?.delete(nodeId);
      } else {
        yDrafts?.set(nodeId, remainingDrafts);
      }
    },
    [yDrafts, clientId],
  );

  const updateMyFieldPatch = useCallback(
    (
      nodeId: string,
      patchKey: "paramsPatch" | "customizationsPatch",
      path: string,
      value: any,
    ) => {
      setMyDraft(nodeId, (existing, nodeDrafts) => ({
        ...existing,
        [patchKey]: {
          ...(existing[patchKey] ?? {}),
          [path]: {
            value,
            updatedAt: Date.now(),
            // Stamped above every counter already in this node's drafts, so
            // edits order by what each client had seen rather than by its
            // clock. Counted from the drafts `setMyDraft` is about to write
            // over, not from the render snapshot: two edits in one tick would
            // otherwise claim the same counter and fall back to `updatedAt`,
            // which cannot separate them inside a millisecond.
            seq: nextSeq(nodeDrafts),
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
      paramsSchema?: FlowSchema,
    ) => {
      if (!openNode || openNode.id !== id) return;

      const latestNodeDrafts = liveNodeDrafts(id);

      const mergedParams = applyMergedPatch(
        openNode.data.params,
        latestNodeDrafts,
        "paramsPatch",
      );

      // Normalize after defaults are applied to prevent issues with whitespace-only values in flowExpr fields, which are considered empty.
      const updatedParams = normalizeParams(
        paramsSchema
          ? (applyDefaults(compile(paramsSchema), mergedParams) ?? mergedParams)
          : mergedParams,
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

      onNodeParamsSaved?.({
        ...openNode,
        data: { ...openNode.data, params: updatedParams },
      });

      removeMyDraft(id);
      onOpenNode();
    },
    [
      openNode,
      liveNodeDrafts,
      onDataSubmit,
      onNodeParamsSaved,
      yDoc,
      removeMyDraft,
      onOpenNode,
    ],
  );

  const handleMigrate = useCallback(
    (id: string, newParams: any, paramsSchema?: FlowSchema) => {
      if (!openNode || openNode.id !== id) return;

      const latestNodeDrafts = liveNodeDrafts(id);
      const updatedCustomizations = applyMergedPatch(
        openNode.data.customizations,
        latestNodeDrafts,
        "customizationsPatch",
      );

      yDoc?.transact(() => {
        onDataSubmit?.([
          {
            nodeId: id,
            updatedParams: normalizeParams(newParams),
            updatedCustomizations,
            paramsSchema,
          },
        ]);
      }, "params");

      removeMyDraft(id);
      // Dialog stays open — user reviews migrated values in normal editor and saves explicitly
    },
    [openNode, liveNodeDrafts, onDataSubmit, yDoc, removeMyDraft],
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

      const path = changedFieldPath(changedFieldId);
      // `""` is the root, which a union-rooted schema writes to; only an
      // absent path means there is nothing to record.
      if (path === undefined) return;

      const value = getAtPath(data, parsePathKey(path));

      updateMyFieldPatch(openNode.id, "paramsPatch", path, value);
    },
    [openNode, updateMyFieldPatch],
  );

  const handleCustomizationChange = useCallback(
    (data: any, changedFieldId?: string) => {
      if (!openNode) return;

      const path = changedFieldPath(changedFieldId);
      // `""` is the root, which a union-rooted schema writes to; only an
      // absent path means there is nothing to record.
      if (path === undefined) return;

      const value = getAtPath(data, parsePathKey(path));

      updateMyFieldPatch(openNode.id, "customizationsPatch", path, value);
    },
    [openNode, updateMyFieldPatch],
  );

  const applyFieldPatch = (fieldContext: FieldContext, value: any) => {
    if (!fieldContext || !openNode) return;
    // `key` is the same dot path a draft patch is filed under.
    updateMyFieldPatch(openNode.id, "paramsPatch", fieldContext.key, value);
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
          attributeSuggestions={attributeSuggestions}
          onClose={() => {
            setOpenFlowExprEditor(false);
            setFlowExprEditorContext(undefined);
            onParamFieldFocus?.(null);
          }}
          onValueSubmit={handleFlowExprValueSubmit}
        />
      )}
    </>
  );
};

export default memo(ParamsDialog);
