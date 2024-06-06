import { generateWorkflows } from "./workflowData";

export function generateProjects(count: number) {
  const projects = [];
  for (let i = 0; i < count; i++) {
    const date = new Date();
    const createdAt = `${date.getFullYear()}-${date.getMonth() + 1}-${date.getDate()} ${date.getHours()}:${date.getMinutes() < 10 ? "0" + date.getMinutes() : date.getMinutes()}`;
    const updatedAt = `${date.getFullYear()}-${date.getMonth() + 1}-${date.getDate()} ${date.getHours()}:${date.getMinutes() < 10 ? "0" + date.getMinutes() : date.getMinutes()}`;
    projects.push({
      id: i.toString(),
      name: i === 0 ? "New Project (empty)" : `My Project ${i + 1}`,
      // workflow: i === 0 ? undefined : generateWorkflows(5),
      workflow: i === 0 ? undefined : generateWorkflows(1)[0],
      createdAt,
      updatedAt,
      description: `Sample Project Description ${i + 1}`,
    });
  }
  return projects;
}
