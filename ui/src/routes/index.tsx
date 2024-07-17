import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { useEffect } from "react";

import { Loading, Button, FlowLogo } from "@flow/components";
import { useUser } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";

export const Route = createFileRoute("/")({
  component: Index,
});

function Index() {
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
}

function ErrorPage({ errorMessage }: { errorMessage: string }) {
  const t = useT();
  return (
    <div className="bg-zinc-800 h-[100vh] flex justify-center items-center">
      <div className="flex flex-col gap-10 items-center">
        <div className="flex gap-4 items-center">
          <div className="bg-red-900 p-2 rounded">
            <FlowLogo className="h-[75px] w-[75px]" />
          </div>
        </div>
        <p className=" text-red-500 font-extralight">{errorMessage}</p>
        <Button variant="outline" onClick={() => window.location.reload()}>
          <p className="text-zinc-300 font-extralight">{t("Reload")}</p>
        </Button>
      </div>
    </div>
  );
}
