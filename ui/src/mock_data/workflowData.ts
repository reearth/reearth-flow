export function generateWorkflows(count: number) {
  const workflows = [];
  for (let i = 0; i < count; i++) {
    workflows.push({
      id: i.toString(),
      name: `My Workflow ${i + 1}`,
      // ...nodes
    });
  }
  return workflows;
}
