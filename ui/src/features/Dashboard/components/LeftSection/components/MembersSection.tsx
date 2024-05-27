import { GitHubLogoIcon, PersonIcon } from "@radix-ui/react-icons";
import { PlusIcon } from "lucide-react";

import { ButtonWithTooltip } from "@flow/components";
import { config } from "@flow/config";
import { useOpenLink } from "@flow/hooks";
import { useT } from "@flow/providers";

import { MembersList } from "./MembersList";

const MembersSection: React.FC = () => {
  const t = useT();
  const { githubRepoUrl, tosUrl, documentationUrl } = config();

  const handleGithubPageOpen = useOpenLink(githubRepoUrl ?? "");
  const handleTosPageOpen = useOpenLink(tosUrl ?? "");
  const handleDocumentationPageOpen = useOpenLink(documentationUrl ?? "");

  return (
    <div className="flex flex-col justify-between gap-6 flex-1 border border-zinc-700 rounded-lg w-[280px] bg-zinc-900/50">
      <div className="flex gap-1 justify-between items-center py-2 px-4 border-b border-zinc-700">
        <p className="text-lg font-extralight">{t("Members")}</p>
        <ButtonWithTooltip
          className="flex gap-2 self-start font-extralight bg-zinc-800 hover:bg-zinc-700"
          variant="outline"
          tooltipPosition="top"
          tooltipText={t("Add a team member")}>
          <div className="flex items-center">
            <PlusIcon className="w-3" />
            <PersonIcon className="w-3" />
          </div>
        </ButtonWithTooltip>
      </div>
      <MembersList />
      <div className="px-4 pb-4">
        {githubRepoUrl && (
          <div className="flex gap-2 items-center">
            <GitHubLogoIcon />
            <p
              className="font-extralight px-2 -mx-2 w-[95%] py-1 -my-1 cursor-pointer rounded-md hover:text-zinc-100 hover:bg-zinc-800 truncate"
              onClick={handleGithubPageOpen}>
              {t("Github")}
            </p>
          </div>
        )}
        {tosUrl && (
          <p
            className="font-extralight px-2 -mx-2 w-[95%] py-1 cursor-pointer rounded-md hover:text-zinc-100 hover:bg-zinc-800 truncate"
            onClick={handleDocumentationPageOpen}>
            {t("Documentation")}
          </p>
        )}
        {documentationUrl && (
          <p
            className="font-extralight px-2 -mx-2 w-[95%] py-1 cursor-pointer rounded-md hover:text-zinc-100 hover:bg-zinc-800 truncate text-wrap"
            onClick={handleTosPageOpen}>
            {t("Terms of Service")}
          </p>
        )}
      </div>
    </div>
  );
};

export { MembersSection };
