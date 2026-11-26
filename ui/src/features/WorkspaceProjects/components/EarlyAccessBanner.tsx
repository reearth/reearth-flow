import { ArrowUpRightIcon, XIcon } from "@phosphor-icons/react";
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
  const { feedbackUrl, earlyAccessUrl } = config();
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
    <div className="flex items-start gap-3 overflow-hidden rounded-md border border-green-300 bg-green-50 pt-1.5 pr-1.5 pb-2.5 pl-3 dark:border-[#1e5a35] dark:bg-[#15291d]">
      <FlowLogo className="mt-1.25 size-4.5 shrink-0" />
      <div className="flex min-w-0 flex-1 flex-col items-start gap-2 pt-1.25">
        <p className="text-sm font-light text-foreground dark:text-zinc-200">
          {t("Flow is in early access. Your feedback helps us improve it.")}
        </p>
        {(feedbackUrl || earlyAccessUrl) && (
          <div className="flex flex-wrap items-center gap-4 text-xs">
            {feedbackUrl && (
              <a
                href={feedbackUrl}
                target="_blank"
                rel="noopener noreferrer"
                className="rounded-md border border-black/10 bg-black/5 px-2.5 py-1 text-foreground transition-colors hover:bg-black/10 dark:border-white/16 dark:bg-white/6 dark:text-zinc-200 dark:hover:bg-white/12">
                {t("Share feedback")}
              </a>
            )}
            {earlyAccessUrl && (
              <a
                href={earlyAccessUrl}
                target="_blank"
                rel="noopener noreferrer"
                className="flex items-center gap-1 text-green-700 hover:underline dark:text-green-400">
                {t("Learn more about early access")}
                <ArrowUpRightIcon size={12} />
              </a>
            )}
          </div>
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
