import { useCallback, useMemo } from "react";
import { Array as YArray } from "yjs";

import { useDeployment } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { rebuildWorkflow } from "@flow/lib/yjs/conversions";
import { YWorkflow } from "@flow/lib/yjs/types";
import { useCurrentProject } from "@flow/stores";
import { Node } from "@flow/types";
import { isDefined } from "@flow/utils";
import { jsonToFormData } from "@flow/utils/jsonToFormData";
import { createEngineReadyWorkflow } from "@flow/utils/toEngineWorkflow/engineReadyWorkflow";

import { useToast } from "../NotificationSystem/useToast";

export default ({
  currentNodes,
  yWorkflows,
}: {
  currentNodes: Node[];
  yWorkflows: YArray<YWorkflow>;
}) => {
  const { toast } = useToast();
  const t = useT();

  const [currentProject] = useCurrentProject();
  const { createDeployment, useUpdateDeployment } = useDeployment();

  const allowedToDeploy = useMemo(
    () => currentNodes.length > 0,
    [currentNodes],
  );

  const handleWorkflowDeployment = useCallback(
    async (description: string, deploymentId?: string) => {
      const {
        name: projectName,
        workspaceId,
        id: projectId,
      } = currentProject ?? {};

      if (!workspaceId || !projectId) return;

      const engineReadyWorkflow = createEngineReadyWorkflow(
        projectName,
        yWorkflows.map((w) => rebuildWorkflow(w)).filter(isDefined),
      );

      if (!engineReadyWorkflow) {
        toast({
          title: t("Empty workflow detected"),
          description: t("You cannot create a deployment without a workflow."),
        });
        return;
      }

      const formData = jsonToFormData(
        engineReadyWorkflow,
        engineReadyWorkflow.id,
      );

      if (deploymentId) {
        await useUpdateDeployment(
          deploymentId,
          formData.get("file") ?? undefined,
          description,
        );
      } else {
        await createDeployment(
          workspaceId,
          projectId,
          engineReadyWorkflow,
          description,
        );
      }
    },
    [
      yWorkflows,
      currentProject,
      t,
      createDeployment,
      useUpdateDeployment,
      toast,
    ],
  );

  return { allowedToDeploy, handleWorkflowDeployment };
};
