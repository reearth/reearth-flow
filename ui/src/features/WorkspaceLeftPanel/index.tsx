import { useRouterState } from "@tanstack/react-router";

import { useCurrentUserRole } from "@flow/stores";
import { Role } from "@flow/types";

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
  const [currentUserRole] = useCurrentUserRole();
  return (
    <div className="m-2 flex w-[260px] flex-col justify-between gap-[8px] rounded-xl border bg-secondary px-2 shadow-md shadow-secondary backdrop-blur-xs">
      <div className="flex flex-1 flex-col">
        <TopSection route={route} />
        {currentUserRole !== Role.Reader && <BottomSection route={route} />}
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
              : pathname.includes("assets")
                ? "assets"
                : "projects";
};
