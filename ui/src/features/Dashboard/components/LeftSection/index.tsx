import { PlusIcon } from "lucide-react";

import { Button } from "@flow/components";
import { config } from "@flow/config";
import { useOpenLink } from "@flow/hooks";
import { useT } from "@flow/providers";

import { WorkspaceMembers } from "./components";

const LeftSection: React.FC = () => {
  const t = useT();
  const githubRepoUrl = config()?.githubRepoUrl;
  const tosUrl = config()?.tosUrl;
  const documentationUrl = config()?.documentationUrl;

  const handleGithubPageOpen = useOpenLink(githubRepoUrl ?? "");
  const handleTosPageOpen = useOpenLink(tosUrl ?? "");
  const handleDocumentationPageOpen = useOpenLink(documentationUrl ?? "");

  return (
    <div className="flex flex-col justify-between gap-6 border border-zinc-700 m-2 p-4 rounded-lg w-[280px]">
      <div className="flex flex-col gap-2">
        {/* <WorkspaceNavigation className="border-b border-zinc-700 pb-2 mb-2" /> */}
        <div className="flex gap-2 py-2">
          <Button className="flex flex-1 gap-1 font-extralight" variant="outline" size="sm">
            <PlusIcon className="w-3" />
            {t("Project")}
          </Button>
          <Button className="flex flex-1 gap-1 font-extralight" variant="outline" size="sm">
            <PlusIcon className="w-3" />
            {t("Workspace")}
          </Button>
        </div>
      </div>
      <WorkspaceMembers />
      <div>
        {githubRepoUrl && (
          <p
            className="font-extralight px-2 -mx-2 w-[95%] py-1 -my-1 cursor-pointer rounded-md hover:text-zinc-100 hover:bg-zinc-800 truncate"
            onClick={handleGithubPageOpen}>
            {t("Documentation")}
          </p>
        )}
        {tosUrl && (
          <p
            className="font-extralight px-2 -mx-2 w-[95%] py-1 cursor-pointer rounded-md hover:text-zinc-100 hover:bg-zinc-800 truncate"
            onClick={handleTosPageOpen}>
            {t("Github")}
          </p>
        )}
        {documentationUrl && (
          <p
            className="font-extralight px-2 -mx-2 w-[95%] py-1 cursor-pointer rounded-md hover:text-zinc-100 hover:bg-zinc-800 truncate text-wrap"
            onClick={handleDocumentationPageOpen}>
            {t("Terms of Service")}
          </p>
        )}
      </div>
    </div>
  );
};

export { LeftSection };
