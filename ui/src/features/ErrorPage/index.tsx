import { Button, FlowLogo } from "@flow/components";
import { useT } from "@flow/lib/i18n";

type Props = {
  errorMessage: string;
};
const ErrorPage: React.FC<Props> = ({ errorMessage }) => {
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
};

export { ErrorPage };
