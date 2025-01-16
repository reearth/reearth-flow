import { useNavigate } from "@tanstack/react-router";
import { useCallback, useEffect, useMemo, useState } from "react";

import {
  Button,
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogContentWrapper,
  DialogTitle,
  Label,
  DialogFooter,
  Select,
  SelectTrigger,
  SelectValue,
  SelectContent,
  SelectItem,
} from "@flow/components";
import { useDeployment } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { useCurrentWorkspace } from "@flow/stores";
import { Deployment } from "@flow/types";

type Props = {
  setShowDialog: (show: boolean) => void;
};

const JobRunDialog: React.FC<Props> = ({ setShowDialog }) => {
  const t = useT();
  const navigate = useNavigate();

  const [currentWorkspace] = useCurrentWorkspace();

  const { useGetDeploymentsInfinite, executeDeployment } = useDeployment();

  const [selectDropDown, setSelectDropDown] = useState<
    HTMLElement | undefined | null
  >();

  const [selectedDeployment, selectDeployment] = useState<
    Deployment | undefined
  >(undefined);

  const { pages, isFetching, fetchNextPage, hasNextPage } =
    useGetDeploymentsInfinite(currentWorkspace?.id);

  const deployments: Deployment[] | undefined = useMemo(
    () =>
      pages?.reduce((deployments, page) => {
        if (page?.deployments) {
          deployments.push(...page.deployments);
        }
        return deployments;
      }, [] as Deployment[]),
    [pages],
  );

  const handleJob = useCallback(async () => {
    if (!selectedDeployment || !currentWorkspace) return;
    const jobData = await executeDeployment({
      deploymentId: selectedDeployment.id,
    });
    if (jobData) {
      navigate({
        to: `/workspaces/${currentWorkspace.id}/jobs/${jobData.job?.id}`,
      });
    }
  }, [currentWorkspace, selectedDeployment, navigate, executeDeployment]);

  useEffect(() => {
    if (
      !selectDropDown ||
      isFetching ||
      !hasNextPage ||
      selectDropDown.clientHeight === 0
    )
      return;

    const { clientHeight, scrollHeight } = selectDropDown;

    if (clientHeight === scrollHeight) {
      fetchNextPage();
      return;
    }

    const handleScrollEnd = () => !isFetching && hasNextPage && fetchNextPage();
    selectDropDown.addEventListener("scrollend", handleScrollEnd);

    return () =>
      selectDropDown.removeEventListener("scrollend", handleScrollEnd);
  }, [selectDropDown, isFetching, hasNextPage, fetchNextPage]);

  return (
    <Dialog open={true} onOpenChange={() => setShowDialog(false)}>
      <DialogContent size="sm">
        <DialogTitle>{t("Run a deployment")}</DialogTitle>
        <DialogContentWrapper>
          <DialogContentSection className="flex flex-col">
            <Label htmlFor="manual-job-deployment">{t("Deployment")}</Label>
            <Select
              onValueChange={(pid) =>
                selectDeployment(deployments?.find((p) => p.id === pid))
              }>
              <SelectTrigger>
                <SelectValue
                  placeholder={t(
                    "Select from the selected workspace's deployments",
                  )}
                />
              </SelectTrigger>
              <SelectContent>
                <div ref={(el) => setSelectDropDown(el?.parentElement)}>
                  {deployments?.map((d) => (
                    <SelectItem key={d.id} value={d.id}>
                      {deploymentDisplay(
                        d.projectName ?? t("Unknown project"),
                        d.version,
                        d.description,
                      )}
                    </SelectItem>
                  ))}
                </div>
              </SelectContent>
            </Select>
          </DialogContentSection>
        </DialogContentWrapper>
        <DialogFooter>
          <Button
            className="self-end"
            variant="outline"
            disabled={!selectedDeployment}
            onClick={handleJob}>
            {t("Run Deployment")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export { JobRunDialog };

const deploymentDisplay = (
  projectName: string,
  version: string,
  description?: string,
) => {
  return description
    ? `${projectName} [${description}] @${version}`
    : `${projectName}@${version}`;
};
