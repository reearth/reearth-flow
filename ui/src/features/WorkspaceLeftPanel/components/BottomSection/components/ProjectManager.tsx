import { SquaresFour } from "@phosphor-icons/react";
import { useNavigate } from "@tanstack/react-router";

import { useT } from "@flow/lib/i18n";
import { useCurrentWorkspace } from "@flow/stores";

type Props = {
  selected?: boolean;
};

const ProjectManager: React.FC<Props> = ({ selected }) => {
  const t = useT();
  const navigate = useNavigate();
  const [currentWorkspace] = useCurrentWorkspace();

  const handleNavigation = () => {
    if (selected) return;
    navigate({ to: `/workspaces/${currentWorkspace?.id}` });
  };

  return (
    <div className="flex w-full flex-col gap-1">
      <p className="text-sm dark:font-thin">{t("Projects")}</p>
      <div
        className={`-mx-2 flex flex-1 cursor-pointer items-center gap-2 rounded px-2 py-1 ${selected && "bg-accent"} hover:bg-accent`}
        onClick={handleNavigation}>
        <SquaresFour weight="light" />
        <p className="dark:font-extralight">{t("Manage Projects")}</p>
      </div>
    </div>
  );
};

export { ProjectManager };
