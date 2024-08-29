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
import { useProject } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { useCurrentWorkspace } from "@flow/stores";
import { Project } from "@flow/types";

const ManualRun: React.FC = () => {
  const t = useT();
  const { useGetWorkspaceProjectsInfinite } = useProject();
  const [selectDropDown, setSelectDropDown] = useState<
    HTMLElement | undefined | null
  >();

  const [selectedProject, selectProject] = useState<Project>();
  const [currentWorkspace] = useCurrentWorkspace();
  const { pages, isFetching, fetchNextPage, hasNextPage } =
    useGetWorkspaceProjectsInfinite(currentWorkspace?.id);

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
    <div className="flex-1 p-8">
      <div className="flex items-center gap-2 text-lg font-extralight">
        <p>{t("Manual Run")}</p>
      </div>
      <div className="mt-4 flex max-w-[1200px] flex-col gap-6">
        <div className="flex w-1/2 max-w-[900px] flex-col gap-4">
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
          <Button className="self-end" size="lg">
            {t("Run")}
          </Button>
        </div>
      </div>
    </div>
  );
};

export { ManualRun };
