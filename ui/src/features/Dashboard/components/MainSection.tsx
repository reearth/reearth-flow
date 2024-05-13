import { useNavigate } from "@tanstack/react-router";

import projectImage from "@flow/assets/project-screenshot.png"; // TODO: replace with actual project image
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@flow/components";
import { useCurrentProject, useCurrentWorkspace } from "@flow/stores";
import type { Project } from "@flow/types";

const MainSection: React.FC = () => {
  const [currentWorkspace] = useCurrentWorkspace();
  const [currentProject, setCurrentProject] = useCurrentProject();
  const navigate = useNavigate({ from: "/workspace/$workspaceId" });

  const handleProjectSelect = (p: Project) => {
    if (currentWorkspace) {
      setCurrentProject(p);
      navigate({ to: `/workspace/${currentWorkspace.id}/project/${p.id}` });
    }
  };

  const projects = currentWorkspace?.projects;

  return (
    <div className="flex flex-col flex-1 justify-between border border-zinc-700 rounded-lg bg-zinc-900/50">
      <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4 overflow-auto py-4 px-4">
        {projects?.map(p => (
          <Card
            className={`cursor-pointer bg-zinc-700/30 border border-transparent ${currentProject && currentProject.id === p.id ? "border-zinc-600" : "hover:border-zinc-600"}`}
            key={p.id}
            onClick={() => handleProjectSelect(p)}>
            <CardHeader className="p-3">
              <CardTitle className="truncate font-extralight">{p.name}</CardTitle>
              {p.description && <CardDescription>{p.description}</CardDescription>}
            </CardHeader>
            <CardContent className="p-0">
              <img src={projectImage} />
            </CardContent>
            <CardFooter className="p-2 flex justify-center">
              <p className="font-thin text-xs">Modified on: 2024/04/26</p>
            </CardFooter>
          </Card>
        ))}
      </div>
      <div className="border-t border-zinc-700 bg-zinc-900/50 rounded-b-lg">
        <p className="font-extralight text-center py-1 border-t">
          Total Projects: {projects?.length ?? 0}
        </p>
      </div>
    </div>
  );
};

export { MainSection };
