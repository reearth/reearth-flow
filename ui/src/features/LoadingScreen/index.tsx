import { useNavigate } from "@tanstack/react-router";
import { useEffect } from "react";

import { Loading } from "@flow/components";
import { useTimeoutOnLoad } from "@flow/hooks";

const LoadingScreen: React.FC = () => {
  const { running: isLoading } = useTimeoutOnLoad(1000);
  const navigate = useNavigate({ from: "/" });
  useEffect(() => {
    if (!isLoading) {
      navigate({ to: "/dashboard" });
    }
  }, [isLoading, navigate]);
  return <Loading show={isLoading} />;
};

export { LoadingScreen };
