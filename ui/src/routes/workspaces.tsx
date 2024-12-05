import { createFileRoute, Outlet } from "@tanstack/react-router";
import { useEffect } from "react";

import { WorkspaceIdWrapper } from "@flow/features/PageWrapper";
import LeftPanel from "@flow/features/WorkspaceLeftPanel";
import { TopNavigation } from "@flow/features/WorkspaceTopNavigation";
import { useUser } from "@flow/lib/gql";
import i18n from "@flow/lib/i18n/i18n";

const WorkspacesComponent = () => {
  const { useGetMe } = useUser();
  const { me } = useGetMe();
  const selectedLanguage =
    me?.lang && me.lang !== "und" ? me.lang : i18n.language;

  useEffect(() => {
    if (selectedLanguage) {
      i18n.changeLanguage(selectedLanguage);
    }
  }, [selectedLanguage]);

  return (
    <WorkspaceIdWrapper>
      <div className="flex h-screen flex-col">
        <TopNavigation />
        <div className="flex h-[calc(100vh-57px)] flex-1">
          <LeftPanel />
          <Outlet />
        </div>
      </div>
    </WorkspaceIdWrapper>
  );
};

export const Route = createFileRoute("/workspaces")({
  component: WorkspacesComponent,
});
