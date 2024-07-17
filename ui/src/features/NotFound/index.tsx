import { Link } from "@tanstack/react-router";

import { Button, FlowLogo } from "@flow/components";
import { useT } from "@flow/lib/i18n";

type Props = {
  message?: string;
};

const NotFound: React.FC<Props> = ({ message }) => {
  const t = useT();

  return (
    <div className="bg-zinc-800 h-[100vh] flex justify-center items-center">
      <div className="flex flex-col gap-10 items-center">
        <div className="flex gap-4 items-center">
          <div className="bg-red-900 p-2 rounded">
            <FlowLogo className="h-[75px] w-[75px]" />
          </div>
          <p className="text-zinc-300 text-4xl font-extralight">{t("Not Found")}</p>
        </div>
        {message && <p className="text-red-500 font-extralight">{message}</p>}
        <Link to={"/"}>
          <Button variant="outline">
            <p className="text-zinc-300 font-extralight italic">{t("Go to Home")}</p>
          </Button>
        </Link>
      </div>
    </div>
  );
};

export default NotFound;
