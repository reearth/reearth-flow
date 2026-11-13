import { useMemo } from "react";

import { useWorkspaceUsers } from "@flow/hooks";
import { useT } from "@flow/lib/i18n";
import type { Project, User } from "@flow/types";

export type ProjectAuthor = Pick<User, "id" | "name" | "photoURL">;

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
