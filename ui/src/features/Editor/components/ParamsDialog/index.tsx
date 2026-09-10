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
import { applySchemaDefaults } from "@flow/components/SchemaForm/patchSchemaTypes";
import { useIsReadOnly } from "@flow/features/Editor/editorContext";
import { useT } from "@flow/lib/i18n";
import type { AwarenessSubEditor, AwarenessUser, Node } from "@flow/types";
import { normalizeParams } from "@flow/utils";

import {
  ParamEditor,
  ValueEditorDialog,
  PythonEditorDialog,
  FlowExprEditorDialog,
  type CodeValue,
} from "./components";
import { AutocompleteSuggestion } from "./components/ValueEditorDialog/components/flowExprConstants";
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
  onNodeParamsSaved?: (node: Node) => void;
  attributeSuggestions?: AutocompleteSuggestion[];
  onWorkflowRename?: (id: string, name: string) => void;
  onParamFieldFocus?: (fieldId: string | null) => void;
  onSubEditorAwareness?: (subEditor: AwarenessSubEditor | null) => void;
  isFollowing?: boolean;
  followSubEditor?: AwarenessSubEditor | null;
  /** Dropped when the follower interacts with a dialog spotlight opened. */
  onSpotlightUserDeselect?: () => void;
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
  onSubEditorAwareness,
  isFollowing = false,
  followSubEditor,
  onSpotlightUserDeselect,
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

  // Field contexts published by each rendered field. Awareness only carries a
  // field id, so this is how a spotlight follow recovers the schema and value
  // needed to reopen the same sub-editor.
  //
  // The registry is stamped with the node it belongs to and reset on write
  // rather than in an effect: child effects run before the parent's, so an
  // effect-based reset would wipe the fields that just registered.
  const fieldContextsRef = useRef<{
    nodeId: string | undefined;
    contexts: Record<string, FieldContext>;
  }>({ nodeId: undefined, contexts: {} });
  const [registeredFieldKey, setRegisteredFieldKey] = useState("");

  const openNodeIdRef = useRef(openNode?.id);
  openNodeIdRef.current = openNode?.id;

  const handleFieldContextRegister = useCallback(
    (fieldContext: FieldContext) => {
      const nodeId = openNodeIdRef.current;
      if (fieldContextsRef.current.nodeId !== nodeId) {
        fieldContextsRef.current = { nodeId, contexts: {} };
      }
      const { contexts } = fieldContextsRef.current;
      const isNew = !contexts[fieldContext.id];
      contexts[fieldContext.id] = fieldContext;
      // Re-render only when a field id shows up for the first time — every
      // keystroke re-registers, and that must not churn the tree.
      if (isNew) {
        setRegisteredFieldKey(
          `${nodeId ?? ""}:${Object.keys(contexts).length}`,
        );
      }
    },
    [],
  );

  const handleSubEditorOpen = useCallback(
    (kind: AwarenessSubEditor["kind"], fieldContext: FieldContext) => {
      if (kind === "value") {
        setValueEditorContext(fieldContext);
        setOpenValueEditor(true);
      } else if (kind === "python") {
        setPythonEditorContext(fieldContext);
        setOpenPythonEditor(true);
      } else {
        setFlowExprEditorContext(fieldContext);
        setOpenFlowExprEditor(true);
      }
      onSubEditorAwareness?.({
        kind,
        fieldId: fieldContext.id,
        fieldName: fieldContext.fieldName,
      });
    },
    [onSubEditorAwareness],
  );

  const handleSubEditorClose = useCallback(() => {
    setOpenValueEditor(false);
    setValueEditorContext(undefined);
    setOpenPythonEditor(false);
    setPythonEditorContext(undefined);
    setOpenFlowExprEditor(false);
    setFlowExprEditorContext(undefined);
    onSubEditorAwareness?.(null);
  }, [onSubEditorAwareness]);

  const localSubEditor: AwarenessSubEditor | null = useMemo(() => {
    if (openValueEditor && valueEditorContext)
      return {
        kind: "value",
        fieldId: valueEditorContext.id,
        fieldName: valueEditorContext.fieldName,
      };
    if (openPythonEditor && pythonEditorContext)
      return {
        kind: "python",
        fieldId: pythonEditorContext.id,
        fieldName: pythonEditorContext.fieldName,
      };
    if (openFlowExprEditor && flowExprEditorContext)
      return {
        kind: "flowExpr",
        fieldId: flowExprEditorContext.id,
        fieldName: flowExprEditorContext.fieldName,
      };
    return null;
  }, [
    openValueEditor,
    valueEditorContext,
    openPythonEditor,
    pythonEditorContext,
    openFlowExprEditor,
    flowExprEditorContext,
  ]);

  // Sub-editor follow is hand-rolled rather than using `useFollowSync` because
  // the field may not have rendered yet when the follow arrives; it also has to
  // rerun as the field registry fills in.
  const localSubEditorRef = useRef(localSubEditor);
  localSubEditorRef.current = localSubEditor;
  const openedByFollowRef = useRef(false);
  const followKey = followSubEditor
    ? `${followSubEditor.kind}:${followSubEditor.fieldId}`
    : null;
  const followSubEditorRef = useRef(followSubEditor);
  followSubEditorRef.current = followSubEditor;
  const subEditorOpenRef = useRef(handleSubEditorOpen);
  subEditorOpenRef.current = handleSubEditorOpen;
  const subEditorCloseRef = useRef(handleSubEditorClose);
  subEditorCloseRef.current = handleSubEditorClose;

  useEffect(() => {
    if (!isFollowing) {
      openedByFollowRef.current = false;
      return;
    }

    const follow = followSubEditorRef.current;
    const local = localSubEditorRef.current;

    if (follow) {
      if (
        local &&
        local.kind === follow.kind &&
        local.fieldId === follow.fieldId
      )
        return;
      const registry = fieldContextsRef.current;
      const fieldContext =
        registry.nodeId === openNodeIdRef.current
          ? registry.contexts[follow.fieldId]
          : undefined;
      // Not rendered yet — this effect reruns when the registry grows.
      if (!fieldContext) return;
      openedByFollowRef.current = true;
      subEditorOpenRef.current(follow.kind, fieldContext);
      return;
    }

    if (openedByFollowRef.current) {
      openedByFollowRef.current = false;
      if (local) subEditorCloseRef.current();
    }
  }, [isFollowing, followKey, registeredFieldKey]);

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

      const mergedParams = applyMergedPatch(
        openNode.data.params,
        latestNodeDrafts,
        "paramsPatch",
      );

      // Normalize after defaults are applied to prevent issues with whitespace-only values in flowExpr fields, which are considered empty.
      const updatedParams = normalizeParams(
        paramsSchema
          ? applySchemaDefaults(paramsSchema, mergedParams)
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
      rawDrafts,
      onDataSubmit,
      onNodeParamsSaved,
      yDoc,
      removeMyDraft,
      onOpenNode,
    ],
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
            updatedParams: normalizeParams(newParams),
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
        {/* Interacting with a dialog spotlight opened hands control back to
            the follower, matching the canvas' click-to-unfollow behaviour. */}
        <DialogContent
          size="2xl"
          onPointerDownCapture={() => onSpotlightUserDeselect?.()}>
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
              onFieldContextRegister={handleFieldContextRegister}
              onValueEditorOpen={(fieldContext) =>
                handleSubEditorOpen("value", fieldContext)
              }
              onPythonEditorOpen={(fieldContext) =>
                handleSubEditorOpen("python", fieldContext)
              }
              onFlowExprEditorOpen={(fieldContext) =>
                handleSubEditorOpen("flowExpr", fieldContext)
              }
            />
          )}
        </DialogContent>
      </Dialog>
      {valueEditorContext && (
        <ValueEditorDialog
          open={openValueEditor}
          fieldContext={valueEditorContext}
          onClose={handleSubEditorClose}
          onValueSubmit={handleValueChange}
        />
      )}
      {pythonEditorContext && (
        <PythonEditorDialog
          open={openPythonEditor}
          fieldContext={pythonEditorContext}
          onClose={handleSubEditorClose}
          onValueSubmit={(value) => applyFieldPatch(pythonEditorContext, value)}
        />
      )}
      {flowExprEditorContext && (
        <FlowExprEditorDialog
          open={openFlowExprEditor}
          fieldContext={flowExprEditorContext}
          attributeSuggestions={attributeSuggestions}
          onClose={() => {
            handleSubEditorClose();
            onParamFieldFocus?.(null);
          }}
          onValueSubmit={handleFlowExprValueSubmit}
        />
      )}
    </>
  );
};

export default memo(ParamsDialog);
