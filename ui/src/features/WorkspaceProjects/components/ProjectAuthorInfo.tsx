import { InfoIcon } from "@phosphor-icons/react";

import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
  UserAvatar,
} from "@flow/components";
import { useLang, useT } from "@flow/lib/i18n";
import type { Project } from "@flow/types";
import { formatRelativeTime } from "@flow/utils";

import useProjectAuthors from "../useProjectAuthors";
import type { ProjectAuthor } from "../useProjectAuthors";

type Props = {
  project: Project;
};

const Separator = () => (
  <span className="shrink-0 text-muted-foreground/70" aria-hidden>
    •
  </span>
);

const AuthorName: React.FC<{ author: ProjectAuthor }> = ({ author }) => (
  <span className="flex min-w-0 items-center gap-1.5">
    <UserAvatar user={author} />
    <span className="truncate">{author.name}</span>
  </span>
);

const DetailRow: React.FC<{ label: string; children: React.ReactNode }> = ({
  label,
  children,
}) => (
  <div className="flex items-center justify-between gap-4 py-1">
    <span className="text-muted-foreground opacity-80">{label}</span>
    <span className="flex min-w-0 justify-end text-foreground">{children}</span>
  </div>
);

// The popups render in a portal, but React still bubbles their clicks up to
// the card, which would open the project.
const stopPropagation = (e: React.MouseEvent) => e.stopPropagation();

// The card footer: who last edited the project and when, with the full
// record of both edits in a tooltip.
const ProjectAuthorInfo: React.FC<Props> = ({ project }) => {
  const t = useT();
  const lang = useLang();
  const { createdBy, updatedBy } = useProjectAuthors(project);

  const dateTime = (iso: string) =>
    new Date(iso).toLocaleString(lang, {
      year: "numeric",
      month: "numeric",
      day: "numeric",
      hour: "numeric",
      minute: "2-digit",
    });

  return (
    <div className="flex w-full min-w-0 items-center gap-1.5 text-xs text-muted-foreground">
      {updatedBy && (
        <>
          <Tooltip>
            <TooltipTrigger
              render={<span />}
              className="flex min-w-0 items-center gap-1">
              <UserAvatar user={updatedBy} />
              <span className="truncate">{updatedBy.name}</span>
            </TooltipTrigger>
            <TooltipContent onClick={stopPropagation}>
              {updatedBy.name}
            </TooltipContent>
          </Tooltip>
          <Separator />
        </>
      )}
      <span className="shrink-0 whitespace-nowrap">
        {t("Updated {{time}}", {
          time: formatRelativeTime(project.updatedAt, lang),
        })}
      </span>
      <Separator />
      <Tooltip>
        <TooltipTrigger
          className="shrink-0 rounded p-0.5 hover:bg-primary data-popup-open:bg-primary"
          aria-label={t("Project details")}
          onClick={(e) => e.stopPropagation()}>
          <InfoIcon className="size-[18px]" />
        </TooltipTrigger>
        <TooltipContent
          side="bottom"
          onClick={stopPropagation}
          className="w-[260px] bg-secondary/95 px-2 py-1.5 shadow-md">
          <DetailRow label={t("Last updated by")}>
            {updatedBy ? <AuthorName author={updatedBy} /> : "-"}
          </DetailRow>
          <DetailRow label={t("Last updated")}>
            {dateTime(project.updatedAt)}
          </DetailRow>
          <div className="my-1 border-t border-border" />
          <DetailRow label={t("Created by")}>
            {createdBy ? <AuthorName author={createdBy} /> : "-"}
          </DetailRow>
          <DetailRow label={t("Created")}>
            {new Date(project.createdAt).toLocaleDateString(lang)}
          </DetailRow>
        </TooltipContent>
      </Tooltip>
    </div>
  );
};

export { ProjectAuthorInfo };
