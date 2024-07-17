import { useState } from "react";

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
  const { useGetWorkspaceProjects } = useProject();

  const [selectedProject, selectProject] = useState<Project>();
  const [currentWorkspace] = useCurrentWorkspace();
  const { projects } = useGetWorkspaceProjects(currentWorkspace?.id);

  return (
    <div className="flex-1 p-8">
      <div className="flex gap-2 items-center text-lg font-extralight">
        <p>{t("Manual Run")}</p>
      </div>
      <div className="flex flex-col gap-6 mt-4 max-w-[1200px]">
        <div className="flex flex-col gap-4 w-[50%] max-w-[900px]">
          <div className="flex flex-col gap-2">
            <Label htmlFor="manual-run-project">{t("Project")}</Label>
            <Select
              onValueChange={pid =>
                selectProject(currentWorkspace?.projects?.find(p => p.id === pid))
              }>
              <SelectTrigger>
                <SelectValue placeholder={t("Select from published projects")} />
              </SelectTrigger>
              <SelectContent>
                {projects?.map(p => (
                  <SelectItem key={p.id} value={p.id}>
                    {p.name}
                  </SelectItem>
                ))}
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
