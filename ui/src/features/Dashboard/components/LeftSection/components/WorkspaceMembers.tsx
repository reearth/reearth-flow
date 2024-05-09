import { PersonIcon } from "@radix-ui/react-icons";
import { PlusIcon } from "lucide-react";

import { Button } from "@flow/components";
import { useT } from "@flow/providers";
import { useCurrentWorkspace } from "@flow/stores";

type Props = {};

const WorkspaceMembers: React.FC<Props> = () => {
  const t = useT();
  const [currentWorkspace] = useCurrentWorkspace();

  return (
    currentWorkspace?.members && (
      <div className="flex flex-col flex-1 gap-4 overflow-auto">
        <div className="flex gap-1 justify-between">
          <p className="text-lg font-extralight">Team</p>
          <Button className="flex gap-2 self-start font-extralight" variant="outline" size="sm">
            <div className="flex items-center">
              <PlusIcon className="w-3" />
              <PersonIcon className="w-3" />
            </div>
            {t("Member")}
          </Button>
        </div>
        <div className="flex flex-col gap-2 overflow-auto bg-zinc-800/50 p-2 rounded">
          {currentWorkspace.members.map(member => (
            <div
              className="flex justify-between bg-zinc-700/30 border border-zinc-600/75 text-zinc-300 rounded py-1 px-2"
              key={member.id}>
              <div className="flex gap-2 items-center truncate">
                <div>
                  <PersonIcon />
                </div>
                <p key={member.id} className="font-thin truncate">
                  {member.name}
                </p>
              </div>
            </div>
          ))}
        </div>
      </div>
    )
  );
};

export { WorkspaceMembers };
