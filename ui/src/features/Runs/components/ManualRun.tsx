import { useState } from "react";

import {
  Button,
  Input,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@flow/components";
import { useT } from "@flow/providers";
import { useCurrentWorkspace } from "@flow/stores";
import { Project } from "@flow/types";

const ManualRun: React.FC = () => {
  const t = useT();
  const [currentWorkspace] = useCurrentWorkspace();
  const [selectedProject, selectProject] = useState<Project>();
  return (
    <>
      <div className="flex gap-2 items-center py-2 px-4 border-b border-zinc-700">
        <p className="text-xl font-thin">{t("Manual Run")}</p>
      </div>
      <div className="flex justify-center py-[50px]">
        <div className="flex flex-col items-center gap-4 w-[50%] max-w-[900px]">
          <div className="flex justify-between items-center gap-4 w-full">
            <p className="shrink-0 font-extralight">{t("Project")}</p>
            <Select
              onValueChange={pid =>
                selectProject(currentWorkspace?.projects?.find(p => p.id === pid))
              }>
              <SelectTrigger>
                <SelectValue placeholder={t("Select from published projects")} />
              </SelectTrigger>
              <SelectContent>
                {currentWorkspace?.projects?.map(p => (
                  <SelectItem key={p.id} value={p.id}>
                    {p.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          {selectedProject && (
            <>
              <div className="flex justify-between items-center gap-4 w-full">
                <p className="shrink-0 font-extralight">Version</p>
                <Input placeholder="Do we need this?" />
              </div>
              <div className="flex justify-between items-center gap-4 w-full">
                <p className="shrink-0 font-extralight">Parameters</p>
                <Input placeholder="What kind of parameters? user params? project params? etc" />
              </div>
            </>
          )}
          <Button className="self-end" size="lg">
            {t("Run")}
          </Button>
        </div>
      </div>
    </>
  );
};

export { ManualRun };
