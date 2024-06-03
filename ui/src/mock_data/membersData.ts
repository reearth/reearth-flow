import { random } from "lodash-es";

import { Member, Role } from "@flow/types";

export function generateTeam(numberOfMembers: number): Member[] {
  const team: Member[] = [];

  for (let i = 0; i < numberOfMembers; i++) {
    // User members
    team.push({
      userId: i.toString(),
      user: {
        id: i + "-id",
        name: `Member ${i}`,
        email: `${i}@reearth.io`,
      },
      // status: i % 2 === 0 ? "online" : "offline",
      role: (i % 2 === 0 ? "writer" : "admin") as Role,
    });

    // Integration members
    team.push({
      id: i.toString(),
      integrationRole: (i % 3 === 0 ? "writer" : "admin") as Role,
      active: i % 2 === 0 ? true : false,
      invitedById: i.toString(),
      integration: {
        id: i + "-id",
        name: `Integration ${i}`,
        logoUrl: "",
        developerId: i + "-developer",
        developer: {
          id: i + "-developer",
          name: `Developer ${i}`,
          email: `${i}@reearth.io`,
        },
        iType: "Public",
        config: {
          token: random(10000).toString(),
        },
      },
    });
  }
  return team;
}
