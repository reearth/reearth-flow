import { SneakerMoveIcon } from "@phosphor-icons/react";
import { useNavigate } from "@tanstack/react-router";

import { useT } from "@flow/lib/i18n";
import { useCurrentWorkspace } from "@flow/stores";

type Props = {
  selected?: boolean;
};

const JobManager: React.FC<Props> = ({ selected }) => {
  const t = useT();
  const navigate = useNavigate();
  const [currentWorkspace] = useCurrentWorkspace();

  return (
    <div className="flex w-full flex-col gap-1">
      <div
        className={`-mx-2 flex flex-1 cursor-pointer items-center gap-2 rounded px-2 py-1 ${selected && "bg-accent"} hover:bg-accent`}
        onClick={() =>
          navigate({ to: `/workspaces/${currentWorkspace?.id}/jobs` })
        }>
        <SneakerMoveIcon weight="light" />
        <p className="text-sm dark:font-extralight">{t("Jobs")}</p>
      </div>
    </div>
  );
};

export { JobManager };
