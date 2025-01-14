import { useRouterState } from "@tanstack/react-router";

import { JobsSection, EndSection } from "./components";

export const routeOptions = [
  "projects",
  "deployments",
  "triggers",
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
    <div className="flex min-w-[250px] flex-col justify-between gap-[8px] border-r bg-secondary">
      <div className="flex flex-1 flex-col">
        <JobsSection route={route} />
        <EndSection route={route} />
      </div>
    </div>
  );
};

export default LeftPanel;

const getRoute = (pathname: string): RouteOption => {
  return pathname.includes("deployments")
    ? "deployments"
    : pathname.includes("triggers")
      ? "triggers"
      : pathname.includes("general")
        ? "general"
        : pathname.includes("integrations")
          ? "integrations"
          : pathname.includes("members")
            ? "members"
            : pathname.includes("completed")
              ? "completed"
              : pathname.includes("running")
                ? "running"
                : pathname.includes("queued")
                  ? "queued"
                  : pathname.includes("all")
                    ? "all"
                    : pathname.includes("new")
                      ? "new"
                      : pathname.includes("jobs") // Since all the above jobs are not present in the routeOptions, we can assume that the route is job's details @KaWaite
                        ? "details"
                        : "projects";
};
