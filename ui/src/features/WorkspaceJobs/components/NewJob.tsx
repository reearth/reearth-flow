import { useNavigate } from "@tanstack/react-router";
import { useCallback, useEffect, useMemo, useState } from "react";

import {
  Button,
  Label,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@flow/components";
import DateTimePicker from "@flow/components/DateTimePicker";
import { useDeployment } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { useCurrentWorkspace } from "@flow/stores";
import type { Deployment } from "@flow/types";

import "./styles.css";

type JobType = "manual" | "trigger";

const NewJob: React.FC = () => {
  const t = useT();
  const navigate = useNavigate();

  const [currentWorkspace] = useCurrentWorkspace();

  const { useGetDeploymentsInfinite, executeDeployment } = useDeployment();

  const { pages, isFetching, fetchNextPage, hasNextPage } =
    useGetDeploymentsInfinite(currentWorkspace?.id);

  const [jobType, setJobType] = useState<JobType | undefined>(undefined);
  const [trigger, setTrigger] = useState<string | undefined>(undefined);

  const [selectDropDown, setSelectDropDown] = useState<
    HTMLElement | undefined | null
  >();
  const [selectedDeployment, selectDeployment] = useState<
    Deployment | undefined
  >(undefined);

  const jobTypes = useMemo(
    () => [
      { label: t("Manual Job"), value: "manual" },
      // { label: t("Trigger Job"), value: "trigger" },
    ],
    [t],
  );

  const triggers = useMemo(
    () => [
      { label: t("Scheduled"), value: "scheduled" },
      { label: t("API"), value: "api" },
      { label: t("Trigger1"), value: "trigger1" },
    ],
    [t],
  );

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
    <div className="flex flex-1 flex-col gap-4 px-6 pb-2 pt-6">
      <div className="flex items-center justify-between gap-4">
        <p className="text-xl dark:font-extralight">{t("New Job")}</p>
        <Button
          className="self-end"
          variant="outline"
          disabled={!jobType || !selectedDeployment}
          onClick={handleJob}>
          {t("Run Job")}
        </Button>
      </div>
      <div className="w-full border-b" />
      <div className="mt-6 flex max-w-[1200px] flex-col gap-6">
        <div className="flex w-1/2 max-w-[900px] flex-col gap-4">
          <div className="flex flex-col gap-2">
            <Label htmlFor="job-type">{t("Job type")}</Label>
            <Select
              onValueChange={(rt) => setJobType(rt as JobType | undefined)}>
              <SelectTrigger>
                <SelectValue placeholder={t("Select desired job type")} />
              </SelectTrigger>
              <SelectContent>
                <div ref={(el) => setSelectDropDown(el?.parentElement)}>
                  {jobTypes?.map((rt) => (
                    <SelectItem key={rt.value} value={rt.value}>
                      {rt.label}
                    </SelectItem>
                  ))}
                </div>
              </SelectContent>
            </Select>
          </div>
          {jobType === "trigger" && (
            <>
              <div className="flex flex-col gap-2">
                <Label htmlFor="trigger-type">{t("Trigger type")}</Label>
                <Select onValueChange={(tr) => setTrigger(tr)}>
                  <SelectTrigger>
                    <SelectValue placeholder={t("Select job type")} />
                  </SelectTrigger>
                  <SelectContent>
                    <div ref={(el) => setSelectDropDown(el?.parentElement)}>
                      {triggers?.map((tr) => (
                        <SelectItem key={tr.value} value={tr.value}>
                          {tr.label}
                        </SelectItem>
                      ))}
                    </div>
                  </SelectContent>
                </Select>
              </div>
              {trigger === "scheduled" && (
                <div className="flex flex-col gap-2">
                  <Label htmlFor="schedule">{t("Schedule")}</Label>
                  <div className="flex items-center justify-around gap-4">
                    <div className="flex items-center gap-2">
                      <Label htmlFor="schedule-start">{t("Start")}</Label>
                      <DateTimePicker />
                    </div>
                    <div className="flex items-center gap-2">
                      <Label htmlFor="schedule-finish">{t("Finish")}</Label>
                      <DateTimePicker />
                    </div>
                  </div>
                </div>
              )}
            </>
          )}
          <div className="flex flex-col gap-2">
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
          </div>
          {/* {selectedDeployment && (
            <>
              <div className="flex flex-col gap-2">
                <Label htmlFor="manual-job-version">{t("Version")}</Label>
                <Input placeholder="Do we need this?" />
              </div>
              <div className="flex flex-col gap-2">
                <Label htmlFor="manual-job-params">{t("Parameters")}</Label>
                <Input placeholder="What kind of parameters? user params? project params? etc" />
              </div>
            </>
          )} */}
        </div>
      </div>
    </div>
  );
};

export { NewJob };

const deploymentDisplay = (
  projectName: string,
  version: string,
  description?: string,
) => {
  return description
    ? `${projectName} [${description}] @${version}`
    : `${projectName}@${version}`;
};
