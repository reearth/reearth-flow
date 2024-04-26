import { PersonIcon } from "@radix-ui/react-icons";

// import { useT } from "@flow/providers";
import { useCurrentWorkspace } from "@flow/stores";

type Props = {};

const WorkspaceMembers: React.FC<Props> = () => {
  //   const t = useT();
  const [currentWorkspace] = useCurrentWorkspace();
  console.log("currentWorkspace", currentWorkspace);
  return (
    currentWorkspace?.members && (
      <div className="flex flex-col gap-2 overflow-auto">
        <p className="font-thin">Team</p>
        {currentWorkspace.members.map(member => (
          <div
            className="flex justify-between border border-zinc-600 text-zinc-400 rounded py-1 px-2"
            key={member.id}>
            <div className="flex gap-2 items-center">
              <PersonIcon />
              <p key={member.id}>{member.name}</p>
            </div>
            {/* <div className="flex gap-2 items-center">
              <div
                className={`h-2 w-2 rounded-full ${member.status === "online" ? "bg-green-300" : "bg-zinc-700"}`}
              />
              <p className={`${member.status === "online" ? "text-zinc-300" : undefined}`}>
                {member.status ?? t("Offline")}
              </p>
            </div> */}
          </div>
        ))}
      </div>
    )
  );
};

export { WorkspaceMembers };
