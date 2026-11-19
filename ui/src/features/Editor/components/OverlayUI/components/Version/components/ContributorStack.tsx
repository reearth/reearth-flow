import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
  UserAvatar,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import type { SnapshotContributor } from "@flow/types";

const MAX_AVATARS = 3;
const MAX_TOOLTIP_NAMES = 10;

type Props = {
  contributors: SnapshotContributor[];
  contributorCount: number;
  lastEditorId?: string;
};

// Who a version is attributed to: the last editor first, then the others. "+N"
// counts every contributor without an avatar: those that did not fit, those cut
// from the stored list, and those that are no longer users.
const ContributorStack: React.FC<Props> = ({
  contributors,
  contributorCount,
  lastEditorId,
}) => {
  const t = useT();
  const count = Math.max(contributorCount, contributors.length);
  if (count === 0) return null;

  // The list arrives sorted by id, an order that means nothing to a reader.
  const ordered = [...contributors].sort(
    (a, b) => Number(b.id === lastEditorId) - Number(a.id === lastEditorId),
  );
  const shown = ordered.slice(0, MAX_AVATARS);
  const named = ordered.slice(0, MAX_TOOLTIP_NAMES);
  const hidden = count - shown.length;
  const unnamed = count - named.length;

  return (
    <Tooltip>
      <TooltipTrigger
        delay={200}
        render={
          <div
            className="flex items-center gap-1"
            aria-label={t("Contributors")}
          />
        }>
        {shown.length > 0 && (
          <div className="flex -space-x-1">
            {shown.map((c) => (
              <UserAvatar
                key={c.id}
                user={c}
                className="size-4 ring-1 ring-card"
              />
            ))}
          </div>
        )}
        {hidden > 0 && <span className="text-xs font-thin">+{hidden}</span>}
      </TooltipTrigger>
      <TooltipContent side="bottom" align="start">
        <ul className="flex flex-col gap-0.5">
          {named.map((c) => (
            <li key={c.id}>
              {c.id === lastEditorId
                ? t("{{name}} (last edit)", { name: c.name })
                : c.name}
            </li>
          ))}
          {unnamed > 0 && (
            <li className="opacity-70">
              {t("{{remaining}} more", { remaining: unnamed })}
            </li>
          )}
        </ul>
      </TooltipContent>
    </Tooltip>
  );
};

export default ContributorStack;
