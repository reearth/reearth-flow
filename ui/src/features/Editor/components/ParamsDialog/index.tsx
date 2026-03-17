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

  const [openValueEditor, setOpenValueEditor] = useState(false);
  const [openPythonEditor, setOpenPythonEditor] = useState(false);
  const [currentFieldContext, setCurrentFieldContext] = useState<
    FieldContext | undefined
  >(undefined);

  // Shared Y.Map for draft state — keyed by nodeId, value is { params, customizations }
  const yDrafts = useMemo(() => yDoc?.getMap<any>("paramDrafts"), [yDoc]);
  const rawDrafts = useY(yDrafts ?? new YMap()) as Record<
    string,
    { params?: any; customizations?: any } | undefined
  >;

  const draft = openNode?.id ? rawDrafts[openNode?.id] : undefined;
  const currentParams =
    draft?.params !== undefined ? draft.params : openNode?.data.params;
  const currentCustomizations =
    draft?.customizations !== undefined
      ? draft.customizations
      : openNode?.data.customizations;

  const handleUpdate = useCallback(
    async (id: string, updatedParams: any, updatedCustomizations: any) => {
      await Promise.resolve(
        onDataSubmit?.([{ nodeId: id, updatedParams, updatedCustomizations }]),
      );
      yDrafts?.delete(id);
      onOpenNode();
    },
    [onDataSubmit, onOpenNode, yDrafts],
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

  // Keep refs always up-to-date so the unmount cleanup can use the latest values
  const nodeIdRef = useRef<string | undefined>(openNode?.id);
  nodeIdRef.current = openNode?.id;
  const usersRef = useRef(users);
  usersRef.current = users;
  const yDraftsRef = useRef(yDrafts);
  yDraftsRef.current = yDrafts;

  useEffect(() => {
    return () => {
      const nodeId = nodeIdRef.current;
      if (!nodeId) return;
      const otherUserHasNodeOpen = Object.values(usersRef.current).some(
        (u) => u.openNodeId === nodeId,
      );
      if (!otherUserHasNodeOpen) {
        yDraftsRef.current?.delete(nodeId);
      }
    };
  }, []);

  const fieldFocusMap = useMemo(() => {
    const map: Record<string, { color: string; userName: string }[]> = {};
    if (!openNode) return map;
    Object.values(users).forEach((user) => {
      if (user.openNodeId === openNode.id && user.focusedParamField) {
        const fieldId = user.focusedParamField;
        if (!map[fieldId]) map[fieldId] = [];

        map[fieldId].push({
          color: user.color,
          userName: user.userName,
        });
      }
    });
    return map;
  }, [users, openNode]);

  const handleParamChange = useCallback(
    (data: any) => {
      if (openNode) {
        const existing = rawDrafts[openNode.id] ?? {};
        yDrafts?.set(openNode.id, { ...existing, params: data });
      }
    },
    [openNode, yDrafts, rawDrafts],
  );

  const handleCustomizationChange = useCallback(
    (data: any) => {
      if (openNode) {
        const existing = rawDrafts[openNode.id] ?? {};
        yDrafts?.set(openNode.id, { ...existing, customizations: data });
      }
    },
    [openNode, yDrafts, rawDrafts],
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
                {Object.entries(users).map(([key, user]) => {
                  return (
                    <div key={key}>
                      {user.openNodeId === openNode?.id && (
                        <div
                          className="flex h-5 w-5 shrink-0 items-center justify-center rounded-full ring-2 ring-secondary/20"
                          style={{ backgroundColor: user.color }}>
                          <span className="text-xs font-medium text-white select-none">
                            {user.userName.charAt(0).toUpperCase()}
                            {user.userName.charAt(1)}
                          </span>
                        </div>
                      )}
                    </div>
                  );
                })}
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
