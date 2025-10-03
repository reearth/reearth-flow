import { GearFineIcon } from "@phosphor-icons/react";
import { useReactFlow } from "@xyflow/react";
import { memo, useCallback, useEffect, useRef, useState } from "react";

import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import type { Node } from "@flow/types";

import {
  ParamEditor,
  ValueEditorDialog,
  PythonEditorDialog,
} from "./components";
import { FieldContext, setValueAtPath } from "./utils/fieldUtils";

type Props = {
  readonly?: boolean;
  openNode?: Node;
  onOpenNode: (nodeId?: string) => void;
  onDataSubmit?: (
    nodeId: string,
    updatedParams: any,
    updatedCustomizations: any,
  ) => void;
  onWorkflowRename?: (id: string, name: string) => void;
};

const ParamsDialog: React.FC<Props> = ({
  readonly,
  openNode,
  onOpenNode,
  onDataSubmit,
  onWorkflowRename,
}) => {
  const t = useT();

  const [openValueEditor, setOpenValueEditor] = useState(false);
  const [openPythonEditor, setOpenPythonEditor] = useState(false);
  const [currentFieldContext, setCurrentFieldContext] = useState<
    FieldContext | undefined
  >(undefined);

  const handleUpdate = useCallback(
    async (nodeId: string, updatedParams: any, updatedCustomizations: any) => {
      await Promise.resolve(
        onDataSubmit?.(nodeId, updatedParams, updatedCustomizations),
      );
      onOpenNode();
    },
    [onDataSubmit, onOpenNode],
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

  const [updatedParams, setUpdatedParams] = useState(openNode?.data.params);

  useEffect(() => {
    if (openNode && !updatedParams) {
      setUpdatedParams(openNode.data.params);
    }
  }, [openNode, updatedParams]);

  const handleParamChange = (data: any) => {
    setUpdatedParams(data);
  };

  const handleValueChange = (value: any) => {
    if (currentFieldContext && openNode) {
      // Update the node's params with the new value
      const currentParams = openNode.data.params || {};
      const updatedParams = setValueAtPath(
        currentParams,
        currentFieldContext.path,
        value,
      );
      // Update the local state with the new params
      handleParamChange?.(updatedParams);
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
                {t("Node Editor")}
              </div>
            </DialogTitle>
          </DialogHeader>
          {openNode && (
            <ParamEditor
              readonly={readonly}
              nodeId={openNode.id}
              nodeMeta={openNode.data}
              nodeType={openNode.type}
              nodeParams={updatedParams}
              onParamsUpdate={handleParamChange}
              onUpdate={handleUpdate}
              onWorkflowRename={onWorkflowRename}
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
