import { useNavigate } from "@tanstack/react-router";
import { useEffect } from "react";

import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@flow/components";
import { workspaces } from "@flow/mock_data/workspaceData";
import { useCurrentProject, useCurrentWorkspace } from "@flow/stores";
import { Project } from "@flow/types";

const MainSection: React.FC = () => {
  const [currentWorkspace, setCurrentWorkspace] = useCurrentWorkspace();
  const [currentProject, setCurrentProject] = useCurrentProject();
  const navigate = useNavigate({ from: "/dashboard" });

  const handleProjectSelect = (p: Project) => {
    setCurrentProject(p);
    navigate({ to: `/project/${p.id}` });
  };

  const projects = currentWorkspace?.projects;

  useEffect(() => {
    if (!currentWorkspace) {
      setCurrentWorkspace(workspaces[0]);
    }
  }, [currentWorkspace, setCurrentWorkspace]);

  return (
    <div className="flex flex-col justify-between gap-2 flex-1 border border-zinc-700 m-2 rounded-lg">
      <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4 overflow-auto py-4 px-4">
        {projects?.map(p => (
          <Card
            className={`cursor-pointer border border-transparent ${currentProject && currentProject.id === p.id ? "border-zinc-600" : "hover:border-zinc-700"}`}
            key={p.id}
            onClick={() => handleProjectSelect(p)}>
            <CardHeader className="p-4">
              <CardTitle>{p.name}</CardTitle>
              {p.description && <CardDescription>{p.description}</CardDescription>}
            </CardHeader>
            <CardContent className="p-0">
              <div className="w-full h-[180px] bg-[url('@flow/assets/project-screenshot.png')] bg-cover bg-center" />
            </CardContent>
            <CardFooter className="p-2 flex justify-center">
              <p className="font-thin text-xs">Modified on: 2024/04/26</p>
            </CardFooter>
          </Card>
        ))}
      </div>
      <p className="font-extralight self-center pb-2">Total Projects: {projects?.length ?? 0}</p>
    </div>
  );
};

export { MainSection };
