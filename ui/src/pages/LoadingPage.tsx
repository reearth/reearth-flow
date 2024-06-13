import { useNavigate } from "@tanstack/react-router";

import { Loading } from "@flow/components";
import { ErrorPage } from "@flow/features/ErrorPage";
import { useUser } from "@flow/lib/gql";

const LoadingPage: React.FC = () => {
  const navigate = useNavigate();
  const { getMe } = useUser();
  const { isLoading, me } = getMe();

  if (isLoading) return <Loading />;

  if (!me || !me?.myWorkspaceId) return <ErrorPage errorMessage={"Could not fetch user"} />;

  // TODO: This gives error in the console
  navigate({ to: `/workspace/${me?.myWorkspaceId}`, replace: true });
};

export { LoadingPage };
