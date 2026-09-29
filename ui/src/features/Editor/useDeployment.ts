import { useCallback, useMemo } from "react";
import type { Map as YMap } from "yjs";

import { useDeployment, useWorkflowVariables } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { rebuildWorkflow } from "@flow/lib/yjs/conversions";
import type { YWorkflow } from "@flow/lib/yjs/types";
import { useCurrentProject } from "@flow/stores";
import type { Deployment, Node } from "@flow/types";
import { isDefined } from "@flow/utils";
import { jsonToFormData } from "@flow/utils/jsonToFormData";
import { createEngineReadyWorkflow } from "@flow/utils/toEngineWorkflow/engineReadyWorkflow";

import { useToast } from "../NotificationSystem/useToast";

export default ({
  currentNodes,
  yWorkflows,
  captureDeploymentFingerprint,
  recordDeployment,
}: {
  currentNodes: Node[];
  yWorkflows: YMap<YWorkflow>;
  captureDeploymentFingerprint: () => string;
  recordDeployment: (version: string, fingerprint: string) => void;
}) => {
  const { toast } = useToast();
  const t = useT();

  const [currentProject] = useCurrentProject();
  const { createDeployment, useUpdateDeployment } = useDeployment();
  const { useGetWorkflowVariables } = useWorkflowVariables();

  const { workflowVariables } = useGetWorkflowVariables(
    currentProject?.id ?? "",
  );

  const allowedToDeploy = useMemo(
    () => currentNodes.length > 0,
    [currentNodes],
  );

  const handleWorkflowDeployment = useCallback(
    async (
      description: string,
      deploymentId?: string,
    ): Promise<Deployment | undefined> => {
      const {
        name: projectName,
        workspaceId,
        id: projectId,
      } = currentProject ?? {};

      if (!workspaceId || !projectId) return;

      const fingerprint = captureDeploymentFingerprint();
      const engineReadyWorkflow = createEngineReadyWorkflow(
        projectName,
        workflowVariables,
        Array.from(yWorkflows.entries())
          .map(([, w]) => rebuildWorkflow(w))
          .filter(isDefined),
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

      const { deployment } = deploymentId
        ? await useUpdateDeployment(
            deploymentId,
            formData.get("file") ?? undefined,
            description,
          )
        : await createDeployment(
            workspaceId,
            projectId,
            engineReadyWorkflow,
            description,
          );
      if (deployment) recordDeployment(deployment.version, fingerprint);
      return deployment;
    },
    [
      yWorkflows,
      currentProject,
      workflowVariables,
      t,
      createDeployment,
      useUpdateDeployment,
      toast,
      captureDeploymentFingerprint,
      recordDeployment,
    ],
  );

  return { allowedToDeploy, handleWorkflowDeployment };
};
