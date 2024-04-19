import { generateWorkflows } from "./workflowData";

export function generateProjects(count: number) {
  const projects = [];
  for (let i = 0; i < count; i++) {
    projects.push({
      id: i.toString(),
      name: `My Project ${i + 1}`,
      workflows: generateWorkflows(5),
    });
  }
  return projects;
}
