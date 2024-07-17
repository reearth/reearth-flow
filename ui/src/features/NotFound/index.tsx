import { Link } from "@tanstack/react-router";

import { Button, FlowLogo } from "@flow/components";
import { useT } from "@flow/lib/i18n";

type Props = {
  message?: string;
};

const NotFound: React.FC<Props> = ({ message }) => {
  const t = useT();

  return (
    <div className="flex h-screen items-center justify-center bg-zinc-800">
      <div className="flex flex-col items-center gap-10">
        <div className="flex items-center gap-4">
          <div className="rounded bg-red-900 p-2">
            <FlowLogo className="size-[75px]" />
          </div>
          <p className="text-4xl font-extralight text-zinc-300">{t("Not Found")}</p>
        </div>
        {message && <p className="font-extralight text-red-500">{message}</p>}
        <Link to={"/"}>
          <Button variant="outline">
            <p className="font-extralight italic text-zinc-300">{t("Go to Home")}</p>
          </Button>
        </Link>
      </div>
    </div>
  );
};

export default NotFound;
