import { useNavigate } from "@tanstack/react-router";

import { Loading } from "@flow/components";

const LoadingPage: React.FC = () => {
  const navigate = useNavigate();
  navigate({ to: `/workspace/` });

  return <Loading />;
};

export { LoadingPage };
