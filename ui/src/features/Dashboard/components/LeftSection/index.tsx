import { Gear } from "@phosphor-icons/react";
import { useNavigate } from "@tanstack/react-router";

import { IconButton } from "@flow/components";

import { RunsSection } from "./components";

const LeftSection: React.FC = () => {
  const navigate = useNavigate();
  return (
    <div className="flex flex-col justify-between bg-zinc-900/50 border-r border-zinc-700 w-[250px] gap-[8px]">
      <RunsSection />
      {/* <MembersSection /> */}
      {/* <div className=""> */}
      <IconButton
        className="m-2"
        icon={<Gear className="h-8 w-8 text-zinc-400" weight="thin" />}
        onClick={() => navigate({ to: `settings/general` })}
      />
      {/* </div> */}
    </div>
  );
};

export { LeftSection };
