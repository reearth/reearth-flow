import { Workspace } from "@flow/types";

import { generateTeam } from "./membersData";
import { generateProjects } from "./projectData";

export const workspaces: Workspace[] = [
  {
    id: "1234",
    name: "Coolest workspace",
    members: generateTeam(10),
    projects: generateProjects(10),
  },
  {
    id: "9120389211231231321233d63234",
    name: "No projects workspace",
    members: generateTeam(30),
    projects: [],
  },
  {
    id: "9120389211231231321233d6",
    name: "Many members and projects",
    members: generateTeam(330),
    projects: generateProjects(50),
  },
  {
    id: "9120389213",
    name: "Long name workspace 2asdfasdfasfdasdasdfasdfsdf",
    members: generateTeam(3),
    projects: generateProjects(3),
  },
  {
    id: "5678",
    name: "another workspace",
    members: generateTeam(5),
    projects: generateProjects(5),
  },
  {
    id: "91011",
    name: "Test workspace",
    members: generateTeam(7),
    projects: generateProjects(7),
  },
  {
    id: "9120389213d3",
    name: "Test workspace 3",
    members: generateTeam(5),
    projects: generateProjects(13),
  },
  {
    id: "9120389213d4",
    name: "Test workspace 4",
    members: generateTeam(5),
    projects: generateProjects(3),
  },
  {
    id: "9120389213d5",
    name: "Test workspace 5",
    members: generateTeam(15),
    projects: generateProjects(3),
  },
  {
    id: "9120389213d6",
    name: "Test workspace 6",
    members: generateTeam(1),
    projects: generateProjects(3),
  },
  {
    id: "9120389213d6123123",
    name: "Test workspace 7",
    members: generateTeam(2),
    projects: generateProjects(13),
  },
];
