import { useNavigate } from "@tanstack/react-router";
import { useEffect } from "react";

import { Loading } from "@flow/components";
import { ErrorPage } from "@flow/features/ErrorPage";
import { useUser } from "@flow/lib/gql";

const IndexLoadingPage: React.FC = () => {
  const navigate = useNavigate();
  const { useGetMe } = useUser();
  const { me, isLoading } = useGetMe();

  useEffect(() => {
    if (!me || !me?.myWorkspaceId) return;
    navigate({ to: `/workspace/${me?.myWorkspaceId}`, replace: true });
  }, [me, navigate]);

  return isLoading ? (
    <Loading />
  ) : !me || !me?.myWorkspaceId ? (
    <ErrorPage errorMessage={"Could not fetch user"} />
  ) : null;
};

export { IndexLoadingPage };
