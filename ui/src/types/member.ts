export type Role = "reader" | "writer" | "admin";

export type Member = {
  id: string;
  name: string;
  // status?: "online" | "offline"; // "away" | "idle" ??
  role: Role;
};
