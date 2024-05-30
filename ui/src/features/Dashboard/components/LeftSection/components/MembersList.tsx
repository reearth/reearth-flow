import { User } from "@phosphor-icons/react";

import { useCurrentWorkspace } from "@flow/stores";

type Props = {};

const MembersList: React.FC<Props> = () => {
  const [currentWorkspace] = useCurrentWorkspace();

  return (
    currentWorkspace?.members && (
      <div className="flex flex-col flex-1 gap-4 overflow-auto px-4 max-h-[80%]">
        <div className="flex flex-col gap-2 overflow-auto rounded">
          {currentWorkspace.members.map(member => (
            <div
              className="flex justify-between items-center bg-zinc-700/30 border border-zinc-600/75 text-zinc-300 rounded py-1 px-2"
              key={member.userId}>
              <div className="flex gap-2 items-center truncate">
                <div>
                  <User weight="thin" />
                </div>
                <p key={member.userId} className="font-thin truncate">
                  {member.user.name}
                </p>
              </div>
              <p key={member.userId} className="font-thin text-sm truncate text-center">
                {member.role === "admin"
                  ? "Admin"
                  : member.role === "writer"
                    ? "Writer"
                    : member.role === "reader"
                      ? "Reader"
                      : "Unknown"}
              </p>
            </div>
          ))}
        </div>
      </div>
    )
  );
};

export { MembersList };
