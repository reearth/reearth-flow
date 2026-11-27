import {
  CircleNotchIcon,
  GraphIcon,
  WarningCircleIcon,
  WifiSlashIcon,
} from "@phosphor-icons/react";
import { useSyncExternalStore } from "react";

import { Button } from "@flow/components";
import type { EmbedErrorReason } from "@flow/lib/embedBridge";
import { useT } from "@flow/lib/i18n";

type Props =
  | { status: "loading" }
  | { status: "empty" }
  | {
      status: "error";
      reason: EmbedErrorReason;
      isRetrying?: boolean;
      onRetry?: () => void;
    };

const subscribeOnline = (onChange: () => void) => {
  window.addEventListener("online", onChange);
  window.addEventListener("offline", onChange);
  return () => {
    window.removeEventListener("online", onChange);
    window.removeEventListener("offline", onChange);
  };
};

const useIsOnline = () =>
  useSyncExternalStore(
    subscribeOnline,
    () => navigator.onLine,
    () => true,
  );

const EmbedStatus: React.FC<Props> = (props) => {
  const t = useT();
  const isOnline = useIsOnline();

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

  if (props.reason === "corrupted") {
    return (
      <div className="flex flex-col items-center gap-3 text-center">
        <WarningCircleIcon size={24} weight="light" />
        <p className="text-sm text-muted-foreground">
          {t("Project or version is corrupted.")}
        </p>
      </div>
    );
  }

  const isNetwork = props.reason === "network";

  return (
    <div className="flex flex-col items-center gap-3 text-center">
      {isNetwork ? (
        <WifiSlashIcon size={24} weight="light" />
      ) : (
        <WarningCircleIcon size={24} weight="light" />
      )}
      <p className="text-sm text-muted-foreground">
        {!isNetwork
          ? t("This workflow can't be shown.")
          : isOnline
            ? t("Couldn't reach Flow. Check your connection and try again.")
            : t("You're offline. Reconnect and try again.")}
      </p>
      {props.onRetry && (
        <Button
          variant="outline"
          size="sm"
          disabled={props.isRetrying}
          onClick={props.onRetry}>
          {props.isRetrying ? t("Retrying...") : t("Retry")}
        </Button>
      )}
    </div>
  );
};

export default EmbedStatus;
