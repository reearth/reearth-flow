import { generateWorkflows } from "./workflowData";

export function generateProjects(count: number) {
  const projects = [];
  for (let i = 0; i < count; i++) {
    projects.push({
      id: i.toString(),
      name: i === 0 ? "New Project (empty)" : `My Project ${i + 1}`,
      workflows: i === 0 ? undefined : generateWorkflows(5),
    });
  }
  return projects;
}
