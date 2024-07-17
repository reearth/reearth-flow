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
    <div className="flex h-screen items-center justify-center bg-zinc-800">
      <div className="flex flex-col items-center gap-10">
        <div className="flex items-center gap-4">
          <div className="rounded bg-red-900 p-2">
            <FlowLogo className="size-[75px]" />
          </div>
        </div>
        <p className=" font-extralight text-red-500">{errorMessage}</p>
        <Button variant="outline" onClick={() => window.location.reload()}>
          <p className="font-extralight text-zinc-300">{t("Reload")}</p>
        </Button>
      </div>
    </div>
  );
}
