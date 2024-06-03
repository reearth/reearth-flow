import { useNavigate } from "@tanstack/react-router";
import { useEffect, useState } from "react";

import { Loading } from "@flow/components";
import { useGetWorkspaceQuery } from "@flow/lib/gql";

const LoadingPage: React.FC = () => {
  const { data, ...rest } = useGetWorkspaceQuery();
  const navigate = useNavigate();
  const [error, setError] = useState<string | undefined>(undefined);

  const isLoading = rest.isLoading;
  const workspaces = data?.me?.workspaces;

  useEffect(() => {
    if (isLoading) return;
    else if (!workspaces) {
      setError("Unable to fetch workspaces");
      return;
    }
    const defaultWorkspace = workspaces.find(w => w.personal);

    if (!defaultWorkspace) {
      setError("No personal id workspace found...");
      return;
    }
    navigate({ to: `/workspace/${defaultWorkspace.id}` });
  }, [workspaces, setError, navigate, isLoading]);

  if (isLoading) {
    return <Loading show={isLoading} />;
  }

  // TODO: Show proper error
  return error && <div>{error}</div>;
};

export { LoadingPage };
