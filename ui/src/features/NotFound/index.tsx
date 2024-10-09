import { Link } from "@tanstack/react-router";

import { Button, FlowLogo } from "@flow/components";
import { useT } from "@flow/lib/i18n";

type Props = {
  message?: string;
};

const NotFound: React.FC<Props> = ({ message }) => {
  const t = useT();

  return (
    <div className="flex h-screen items-center justify-center">
      <div className="flex flex-col items-center gap-14">
        <div className="flex items-center gap-10">
          <div className="p-2">
            <FlowLogo className="size-[75px]" />
          </div>
          <p className="text-4xl dark:font-extralight">{t("Not Found")}</p>
        </div>
        {message && (
          <p className="text-red-500 dark:font-extralight">{message}</p>
        )}
        <Link to={"/"}>
          <Button variant="outline">
            <p className="italic dark:font-extralight">{t("Go to Home")}</p>
          </Button>
        </Link>
      </div>
    </div>
  );
};

export default NotFound;
