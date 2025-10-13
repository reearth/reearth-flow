import { ReactFlowProvider } from "@xyflow/react";
import * as Y from "yjs";

import { RenderFallback } from "@flow/components";
import VersionCanvas from "@flow/features/VersionCanvas";
import { useT } from "@flow/lib/i18n";
import { YWorkflow } from "@flow/lib/yjs/types";

const VersionEditorComponent: React.FC<{
  yDoc: Y.Doc | null;
  previewDocYWorkflows: Y.Map<YWorkflow> | null;
  onWorkflowCorruption?: () => void;
}> = ({ yDoc, previewDocYWorkflows, onWorkflowCorruption }) => {
  const t = useT();
  const yWorkflows = previewDocYWorkflows
    ? previewDocYWorkflows
    : yDoc
      ? yDoc.getMap<YWorkflow>("workflows")
      : null;

  return (
    <div className="h-full w-full">
      {yWorkflows && (
        <RenderFallback
          onError={onWorkflowCorruption}
          message={t("Selected version is corrupted or not available.")}
          textSize="md">
          <ReactFlowProvider>
            <VersionCanvas yWorkflows={yWorkflows} />
          </ReactFlowProvider>
        </RenderFallback>
      )}
    </div>
  );
};

export default VersionEditorComponent;
