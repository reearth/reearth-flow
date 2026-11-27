import {
  CircleNotchIcon,
  GraphIcon,
  WarningCircleIcon,
} from "@phosphor-icons/react";

import type { EmbedErrorReason } from "@flow/lib/embedBridge";
import { useT } from "@flow/lib/i18n";

type Props =
  | { status: "loading" }
  | { status: "empty" }
  | { status: "error"; reason: EmbedErrorReason };

const EmbedStatus: React.FC<Props> = (props) => {
  const t = useT();

  if (props.status === "loading") {
    return (
      <div className="flex flex-col items-center gap-3 text-muted-foreground">
        <CircleNotchIcon size={24} className="animate-spin" />
        <p className="text-sm">{t("Loading workflow")}</p>
      </div>
    );
  }

  if (props.status === "empty") {
    return (
      <div className="flex flex-col items-center gap-3 text-center">
        <div className="flex size-10 items-center justify-center rounded-md bg-secondary">
          <GraphIcon size={20} weight="light" />
        </div>
        <p className="text-sm font-medium">{t("No actions yet")}</p>
        <p className="text-xs text-muted-foreground">
          {t("Actions added in the Flow editor show up here.")}
        </p>
      </div>
    );
  }

  return (
    <div className="flex flex-col items-center gap-3 text-center">
      <WarningCircleIcon size={24} weight="light" />
      <p className="text-sm text-muted-foreground">
        {props.reason === "corrupted"
          ? t("Project or version is corrupted.")
          : t("This workflow can't be shown.")}
      </p>
    </div>
  );
};

export default EmbedStatus;
