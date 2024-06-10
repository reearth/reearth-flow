import { useNavigate, useParams } from "@tanstack/react-router";
import { useEffect } from "react";

import { Loading } from "@flow/components";
import { useWorkspace } from "@flow/lib/gql";
import { useCurrentWorkspace } from "@flow/stores";

import { TopNavigation } from "../TopNavigation";

import { LeftSection, MainSection } from "./components";

const Dashboard: React.FC = () => {
  const [_, setCurrentWorkspace] = useCurrentWorkspace();
  const { workspaceId } = useParams({ strict: false });
  const navigate = useNavigate();

  const { getWorkspaces } = useWorkspace();
  const { workspaces } = getWorkspaces();

  useEffect(() => {
    if (!workspaces) return;
    const selectedWorkspace = workspaces?.find(w => w.id === workspaceId);

    if (!selectedWorkspace) {
      // TODO: This returns a promise but it can't be awaited
      navigate({ to: `/workspace/${workspaces[0].id}`, replace: true });
    }

    setCurrentWorkspace(selectedWorkspace);
  }, [workspaces, navigate, setCurrentWorkspace, workspaceId]);

  if (!workspaces) {
    return <Loading />;
  }

  return (
    <div className="[&>*]:dark flex flex-col bg-zinc-800 text-zinc-300 h-[100vh]">
      <TopNavigation />
      <div className="flex-1 flex">
        <LeftSection />
        <MainSection />
      </div>
    </div>
  );
};

export { Dashboard };
