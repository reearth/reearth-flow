import { ProjectVar } from "@flow/types";

export const projectVariables: ProjectVar[] = [
  {
    id: "asdfasfsdasdffsdf",
    name: "project_name",
    definition: "My Projectjlasjdflakjsdflkjsadflkjasdflkjasdlkjafsdlk",
    type: "string",
    required: true,
  },
  {
    id: "asdfasf3333sdfsdf",
    name: "project_id",
    definition: 5558687687888887,
    type: "number",
    required: false,
  },
  {
    id: "asdfasf234234sdfsdf",
    name: "is_active",
    definition: true,
    type: "boolean",
    required: false,
  },
  {
    id: "asdfas132131fsdfsdf",
    name: "tags",
    definition: ["tag1", "tag2"],
    type: "array",
    required: true,
  },
  // {
  //   key: "config",
  //   value: '{"key": "value"}',
  //   type: "object",
  // },
];
