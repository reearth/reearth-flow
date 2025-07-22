import { GearFineIcon } from "@phosphor-icons/react";
import { useReactFlow } from "@xyflow/react";
import { memo, useCallback, useEffect, useRef, useState } from "react";

import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@flow/components";
import AssetsDialog from "@flow/features/AssetsDialog";
import { useT } from "@flow/lib/i18n";
import { Asset, Node } from "@flow/types";

import { ParamEditor, ValueEditorDialog } from "./components";
import { FieldContext, setValueAtPath } from "./utils/fieldUtils";

type Props = {
  readonly?: boolean;
  openNode?: Node;
  onOpenNode: (nodeId?: string) => void;
  onDataSubmit?: (
    nodeId: string,
    dataField: "params" | "customizations",
    updatedValue: any,
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
  const [showAssets, setShowAssets] = useState(false);
  const [currentFieldContext, setCurrentFieldContext] = useState<
    FieldContext | undefined
  >(undefined);

  const handleUpdate = useCallback(
    async (nodeId: string, data: any, type: "params" | "customizations") => {
      if (type === "params") {
        await Promise.resolve(onDataSubmit?.(nodeId, "params", data));
      } else if (type === "customizations") {
        await Promise.resolve(onDataSubmit?.(nodeId, "customizations", data));
      }
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

  const handleAssetDoubleClick = (asset: Asset) => {
    const v = asset.url;
    handleValueChange(v);
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
              onAssetsOpen={(fieldContext) => {
                setCurrentFieldContext(fieldContext);
                setShowAssets(true);
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
      {showAssets && currentFieldContext && (
        <AssetsDialog
          onDialogClose={() => {
            setShowAssets(false);
            setCurrentFieldContext(undefined);
          }}
          onAssetDoubleClick={handleAssetDoubleClick}
        />
      )}
    </>
  );
};

export default memo(ParamsDialog);
