import { NavigationMenu, NavigationMenuList } from "@flow/components";

import { UserNavigation } from "../../../../../../Dashboard/components/UserNavigation";

import { WorkspaceNavigation } from "./components/WorkspaceNavigation";

const WelcomeDialogHeader: React.FC = () => {
  return (
    <div className="flex justify-between mb-6 py-4 text-zinc-400">
      <WorkspaceNavigation />
      <NavigationMenu>
        {/* <NavigationMenu className="justify-end flex mb-4 py-4"> */}
        <NavigationMenuList>
          <UserNavigation />
        </NavigationMenuList>
      </NavigationMenu>
    </div>
  );
};

export { WelcomeDialogHeader };
