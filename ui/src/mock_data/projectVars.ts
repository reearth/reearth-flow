import { ProjectVar } from "@flow/features/Editor/components/LeftPanel/components/ProjectVariables/ProjectVariable";

export const projectVariables: ProjectVar[] = [
  {
    key: "project_name",
    value: "My Projectjlasjdflakjsdflkjsadflkjasdflkjasdlkjafsdlk",
    type: "string",
  },
  {
    key: "project_id",
    value: 5558687687888887,
    type: "number",
  },
  {
    key: "is_active",
    value: "true",
    type: "boolean",
  },
  {
    key: "tags",
    value: "['tag1', 'tag2']",
    type: "array",
  },
  {
    key: "config",
    value: '{"key": "value"}',
    type: "object",
  },
];
