import { Role } from "@flow/types";

export function generateTeam(numberOfMembers: number) {
  const team = [];
  for (let i = 0; i < numberOfMembers; i++) {
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
  }
  return team;
}
