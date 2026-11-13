import { useMemo } from "react";

import { useCurrentWorkspace } from "@flow/stores";
import type { User } from "@flow/types";

// The current workspace's members by user ID. User IDs stored on projects and
// jobs are matched against it rather than fetched, so their names and photos
// come from the members query the page already makes.
export default () => {
  const [workspace] = useCurrentWorkspace();
  return useMemo(() => {
    const users = new Map<string, User>();
    workspace?.members?.forEach((m) => {
      if ("userId" in m && m.user) {
        users.set(m.userId, m.user);
      }
    });
    return users;
  }, [workspace?.members]);
};
