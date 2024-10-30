import { useRouterState } from "@tanstack/react-router";

import { RunsSection, EndSection } from "./components";

export const routeOptions = [
  "projects",
  "deployments",
  "general",
  "integrations",
  "members",
];

export type RouteOption = (typeof routeOptions)[number];

const LeftPanel: React.FC = () => {
  const {
    location: { pathname },
  } = useRouterState();

  const route: RouteOption = pathname.includes("deployments")
    ? "deployments"
    : pathname.includes("general")
      ? "general"
      : pathname.includes("integrations")
        ? "integrations"
        : pathname.includes("members")
          ? "members"
          : "projects";

  return (
    <div className="flex w-[250px] flex-col justify-between gap-[8px] border-r bg-secondary">
      <div className="flex flex-1 flex-col">
        <RunsSection />
        <EndSection route={route} />
      </div>
    </div>
  );
};

export default LeftPanel;
