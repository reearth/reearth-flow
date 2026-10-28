import { useMemo } from "react";

import { useT } from "@flow/lib/i18n";
import { useCurrentWorkspace } from "@flow/stores";
import type { Project, User, UserMember } from "@flow/types";

export type ProjectAuthor = Pick<User, "id" | "name" | "photoURL">;

// The workspace's members by user ID. Project authors are matched against it
// rather than fetched, so their names and photos come from the members query
// the page already makes.
export const useWorkspaceUsers = () => {
  const [workspace] = useCurrentWorkspace();
  return useMemo(() => {
    const users = new Map<string, User>();
    workspace?.members?.forEach((m) => {
      if ("userId" in m && (m as UserMember).user) {
        users.set(m.userId, (m as UserMember).user as User);
      }
    });
    return users;
  }, [workspace?.members]);
};

export default (project: Project) => {
  const t = useT();
  const users = useWorkspaceUsers();

  return useMemo(() => {
    // An author who has left the workspace is still shown, by a placeholder.
    const resolve = (id?: string): ProjectAuthor | undefined =>
      id ? (users.get(id) ?? { id, name: t("Unknown user") }) : undefined;

    const createdBy = resolve(project.createdById);
    // A project nobody has edited yet was last touched by its creator.
    const updatedBy = resolve(project.updatedById) ?? createdBy;
    return { createdBy, updatedBy };
  }, [users, project.createdById, project.updatedById, t]);
};
