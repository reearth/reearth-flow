import { PersonIcon } from "@radix-ui/react-icons";
import { PlusIcon } from "lucide-react";

import { Button, FlowLogo } from "@flow/components";
import { config } from "@flow/config";
import { useOpenLink } from "@flow/hooks";
import { useT } from "@flow/providers";

import { ContentSection } from "../../../..";

const WelcomeDialogLeft: React.FC = () => {
  const t = useT();
  const githubRepoUrl = config()?.githubRepoUrl;
  const tosUrl = config()?.tosUrl;
  const documentationUrl = config()?.documentationUrl;

  const handleGithubPageOpen = useOpenLink(githubRepoUrl ?? "");
  const handleTosPageOpen = useOpenLink(tosUrl ?? "");
  const handleDocumentationPageOpen = useOpenLink(documentationUrl ?? "");

  return (
    <div className="border-r border-zinc-700 w-[172px]">
      <FlowLogo className="w-[80px] h-[80px] bg-red-900/80 p-2 ml-6 mb-8 rounded-md" />
      <ContentSection
        className="pr-4"
        title={t("Getting started")}
        content={
          <div className="flex flex-col gap-2">
            <Button className="flex gap-2" variant="outline" size="sm">
              <PlusIcon className="w-3" />
              {t("New Project")}
            </Button>
            <Button className="flex gap-2" variant="outline" size="sm">
              <PlusIcon className="w-3" />
              {t("New Workspace")}
            </Button>
            <Button className="flex gap-2" variant="outline" size="sm">
              <div className="flex">
                <PlusIcon className="w-3" />
                <PersonIcon className="w-3" />
              </div>
              {t("Add member")}
            </Button>
          </div>
        }
      />
      <ContentSection
        title={t("Resources")}
        content={
          <div className="text-zinc-400 flex flex-col">
            {githubRepoUrl && (
              <p
                className="font-extralight px-2 -mx-2 w-[95%] py-1 -my-1 cursor-pointer rounded-md hover:text-zinc-200 hover:bg-zinc-800 truncate"
                onClick={handleGithubPageOpen}>
                {t("Documentation")}
              </p>
            )}
            {tosUrl && (
              <p
                className="font-extralight px-2 -mx-2 w-[95%] py-1 cursor-pointer rounded-md hover:text-zinc-200 hover:bg-zinc-800 truncate"
                onClick={handleTosPageOpen}>
                {t("Github")}
              </p>
            )}
            {documentationUrl && (
              <p
                className="font-extralight px-2 -mx-2 w-[95%] py-1 cursor-pointer rounded-md hover:text-zinc-200 hover:bg-zinc-800 truncate text-wrap"
                onClick={handleDocumentationPageOpen}>
                {t("Terms of Service")}
              </p>
            )}
          </div>
        }
      />
    </div>
  );
};

export { WelcomeDialogLeft };
