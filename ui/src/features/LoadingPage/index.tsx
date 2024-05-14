import { useNavigate } from "@tanstack/react-router";
import { useEffect } from "react";

import { Loading } from "@flow/components";
import { useTimeoutOnLoad } from "@flow/hooks";
import { workspaces } from "@flow/mock_data/workspaceData";

const LoadingPage: React.FC = () => {
  const { running: isLoading } = useTimeoutOnLoad(1000);
  const navigate = useNavigate({ from: "/" });
  useEffect(() => {
    if (!isLoading) {
      navigate({ to: `/workspace/${workspaces[0].id}` });
    }
  }, [isLoading, navigate]);
  return <Loading show={isLoading} />;
};

export { LoadingPage };
