import { XIcon } from "@phosphor-icons/react";
import { useCallback, useState } from "react";

import { FlowLogo, IconButton } from "@flow/components";
import { config } from "@flow/config";
import { useUser } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";

const storageKey = (userId: string) =>
  `reearth-flow:early-access-banner-dismissed:${userId}`;

const readDismissed = (userId: string) => {
  try {
    return window.localStorage.getItem(storageKey(userId)) === "true";
  } catch {
    return false;
  }
};

const EarlyAccessBanner: React.FC = () => {
  const { useGetMe } = useUser();
  const { me } = useGetMe();

  if (!me?.id) return null;
  return <EarlyAccessBannerContent key={me.id} userId={me.id} />;
};

const EarlyAccessBannerContent: React.FC<{ userId: string }> = ({ userId }) => {
  const t = useT();
  const { feedbackUrl } = config();
  const [dismissed, setDismissed] = useState(() => readDismissed(userId));

  const handleClose = useCallback(() => {
    setDismissed(true);
    try {
      window.localStorage.setItem(storageKey(userId), "true");
    } catch {
      return;
    }
  }, [userId]);

  if (dismissed) return null;

  return (
    <div className="flex items-center gap-3 overflow-hidden rounded-md border border-green-300 bg-green-50 py-1.5 pr-1.5 pl-3 dark:border-[#1e5a35] dark:bg-[#15291d]">
      <FlowLogo className="size-4.5 shrink-0" />
      <div className="flex min-w-0 flex-1 items-center gap-2 text-sm">
        <p className="truncate font-light text-foreground dark:text-zinc-200">
          {t("Flow is in early access. Your feedback helps us improve it.")}
        </p>
        {feedbackUrl && (
          <a
            href={feedbackUrl}
            target="_blank"
            rel="noopener noreferrer"
            className="shrink-0 text-green-700 hover:underline dark:text-green-400">
            {t("Share feedback")}
          </a>
        )}
      </div>
      <IconButton
        className="size-7 shrink-0 rounded"
        icon={<XIcon size={16} weight="light" />}
        aria-label={t("Close")}
        onClick={handleClose}
      />
    </div>
  );
};

export { EarlyAccessBanner };
