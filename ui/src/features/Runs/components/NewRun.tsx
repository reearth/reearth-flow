import { useEffect, useMemo, useState } from "react";

import {
  Button,
  Input,
  Label,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@flow/components";
import DateTimePicker from "@flow/components/DateTimePicker";
import { useProject } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { useCurrentWorkspace } from "@flow/stores";
import type { Project } from "@flow/types";

import "./styles.css";

type RunType = "manual" | "trigger";

const NewRun: React.FC = () => {
  const t = useT();
  const [currentWorkspace] = useCurrentWorkspace();

  const { useGetWorkspaceProjectsInfinite } = useProject();
  const { pages, isFetching, fetchNextPage, hasNextPage } =
    useGetWorkspaceProjectsInfinite(currentWorkspace?.id);

  const [runType, setRunType] = useState<RunType | undefined>(undefined);
  const [trigger, setTrigger] = useState<string | undefined>(undefined);

  const [selectDropDown, setSelectDropDown] = useState<
    HTMLElement | undefined | null
  >();
  const [selectedProject, selectProject] = useState<Project>();

  const runTypes = useMemo(
    () => [
      { label: t("Manual Run"), value: "manual" },
      { label: t("Trigger run"), value: "trigger" },
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

  const projects: Project[] | undefined = useMemo(
    () =>
      pages?.reduce((projects, page) => {
        if (page?.projects) {
          projects.push(...page.projects);
        }
        return projects;
      }, [] as Project[]),
    [pages],
  );

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
        <p className="text-xl dark:font-extralight">{t("New run")}</p>
        <Button className="self-end" variant="outline">
          {t("Run")}
        </Button>
      </div>
      <div className="w-full border-b" />
      <div className="mt-6 flex max-w-[1200px] flex-col gap-6">
        <div className="flex w-1/2 max-w-[900px] flex-col gap-4">
          <div className="flex flex-col gap-2">
            <Label htmlFor="run-type">{t("Run type")}</Label>
            <Select
              onValueChange={(rt) => setRunType(rt as RunType | undefined)}>
              <SelectTrigger>
                <SelectValue placeholder={t("Select run type")} />
              </SelectTrigger>
              <SelectContent>
                <div ref={(el) => setSelectDropDown(el?.parentElement)}>
                  {runTypes?.map((rt) => (
                    <SelectItem key={rt.value} value={rt.value}>
                      {rt.label}
                    </SelectItem>
                  ))}
                </div>
              </SelectContent>
            </Select>
          </div>
          {runType === "trigger" && (
            <>
              <div className="flex flex-col gap-2">
                <Label htmlFor="trigger-type">{t("Trigger type")}</Label>
                <Select onValueChange={(tr) => setTrigger(tr)}>
                  <SelectTrigger>
                    <SelectValue placeholder={t("Select run type")} />
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
            <Label htmlFor="manual-run-project">{t("Project")}</Label>
            <Select
              onValueChange={(pid) =>
                selectProject(
                  currentWorkspace?.projects?.find((p) => p.id === pid),
                )
              }>
              <SelectTrigger>
                <SelectValue
                  placeholder={t("Select from published projects")}
                />
              </SelectTrigger>
              <SelectContent>
                <div ref={(el) => setSelectDropDown(el?.parentElement)}>
                  {projects?.map((p) => (
                    <SelectItem key={p.id} value={p.id}>
                      {p.name}
                    </SelectItem>
                  ))}
                </div>
              </SelectContent>
            </Select>
          </div>
          {selectedProject && (
            <>
              <div className="flex flex-col gap-2">
                <Label htmlFor="manual-run-version">{t("Version")}</Label>
                <Input placeholder="Do we need this?" />
              </div>
              <div className="flex flex-col gap-2">
                <Label htmlFor="manual-run-params">{t("Parameters")}</Label>
                <Input placeholder="What kind of parameters? user params? project params? etc" />
              </div>
            </>
          )}
        </div>
      </div>
    </div>
  );
};

export { NewRun };
