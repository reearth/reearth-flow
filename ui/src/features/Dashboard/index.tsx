import { useNavigate, useParams } from "@tanstack/react-router";
import { useEffect } from "react";

import { Loading } from "@flow/components";
import { useGetWorkspaceQuery } from "@flow/lib/gql";
import { useCurrentWorkspace } from "@flow/stores";

import { LeftSection, MainSection, Nav } from "./components";

const Dashboard: React.FC = () => {
  const [_, setCurrentWorkspace] = useCurrentWorkspace();
  const { workspaceId } = useParams({ strict: false });
  const navigate = useNavigate();

  const { data } = useGetWorkspaceQuery();

  const workspaces = data?.me?.workspaces;

  useEffect(() => {
    if (!workspaces) return;
    const selectedWorkspace = workspaces?.find(w => w.id === workspaceId);
    setCurrentWorkspace(selectedWorkspace);

    if (!selectedWorkspace) {
      setCurrentWorkspace(workspaces[0]);
      navigate({ to: `/workspace/${workspaces[0].id}`, replace: true });
    }
  }, [workspaces, navigate, setCurrentWorkspace, workspaceId]);

  // TODO: this needs a common component
  if (!workspaces) {
    return <Loading />;
  }

  return (
    <div className="[&>*]:dark flex flex-col bg-zinc-800 text-zinc-300 h-[100vh]">
      <Nav />
      <div className="flex-1 m-[8px] flex gap-[8px]">
        <LeftSection />
        <MainSection />
      </div>
    </div>
  );
};

export { Dashboard };
