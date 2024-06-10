import { useNavigate } from "@tanstack/react-router";

import { Loading } from "@flow/components";
import { useUser } from "@flow/lib/gql";

const LoadingPage: React.FC = () => {
  const navigate = useNavigate();
  const { getMe } = useUser();
  const { isLoading, me } = getMe();

  if (isLoading) return <Loading />;

  // TODO: Show proper error
  if (!me || !me?.myWorkspaceId) return <div>Could not fetch user</div>;

  // TODO: This gives error in the console
  navigate({ to: `/workspace/${me?.myWorkspaceId}`, replace: true });
};

export { LoadingPage };
