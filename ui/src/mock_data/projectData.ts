import { generateWorkflows } from "./workflowData";

export function generateProjects(count: number) {
  const projects = [];
  for (let i = 0; i < count; i++) {
    projects.push({
      id: i.toString(),
      name: i === 0 ? "New Project (empty)" : `My Project ${i + 1}`,
      // workflow: i === 0 ? undefined : generateWorkflows(5),
      workflow: i === 0 ? undefined : generateWorkflows(1)[0],
    });
  }
  return projects;
}
