import { useRouterState } from "@tanstack/react-router";

import { BottomSection, TopSection } from "./components";

export const routeOptions = [
  "projects",
  "deployments",
  "general",
  "integrations",
  "members",
  "runnning",
  "queued",
  "completed",
  "new",
  "all",
];

export type RouteOption = (typeof routeOptions)[number];

const LeftPanel: React.FC = () => {
  const {
    location: { pathname },
  } = useRouterState();

  const route: RouteOption = getRoute(pathname);

  return (
    <div className="flex w-[240px] flex-col justify-between gap-[8px] bg-secondary">
      <div className="flex flex-1 flex-col">
        <TopSection route={route} />
        <BottomSection route={route} />
      </div>
    </div>
  );
};

export default LeftPanel;

const getRoute = (pathname: string): RouteOption => {
  return pathname.includes("deployments")
    ? "deployments"
    : pathname.includes("general")
      ? "general"
      : pathname.includes("integrations")
        ? "integrations"
        : pathname.includes("members")
          ? "members"
          : pathname.includes("jobs")
            ? "jobs"
            : pathname.includes("triggers")
              ? "triggers"
              : "projects";
};
