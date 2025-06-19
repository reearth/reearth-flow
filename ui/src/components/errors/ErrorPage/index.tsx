import { Button } from "@flow/components/buttons";
import { FlowLogo } from "@flow/components/icons";
import { useT } from "@flow/lib/i18n";

function ErrorPage({ errorMessage }: { errorMessage: string }) {
  const t = useT();
  return (
    <div className="flex h-screen items-center justify-center">
      <div className="flex flex-col items-center gap-10">
        <div className="flex items-center gap-4">
          <div className="rounded bg-logo p-2">
            <FlowLogo className="size-[75px]" />
          </div>
        </div>
        <p className="text-destructive dark:font-extralight">{errorMessage}</p>
        <Button variant="outline" onClick={() => window.location.reload()}>
          <p className="dark:font-extralight">{t("Reload")}</p>
        </Button>
      </div>
    </div>
  );
}

export default ErrorPage;
