import { useEffect, useMemo, useRef } from "react";

import { useDeployment } from "@flow/lib/gql";
import { Deployment, Workspace } from "@flow/types";

export default ({ workspace }: { workspace: Workspace }) => {
  const ref = useRef<HTMLDivElement>(null);

  const { useGetDeploymentsInfinite } = useDeployment();

  const { pages, hasNextPage, isFetching, fetchNextPage } =
    useGetDeploymentsInfinite(workspace.id);

  const deployments: Deployment[] | undefined = useMemo(
    () =>
      pages?.reduce((deployments, page) => {
        if (page?.deployments) {
          deployments.push(...page.deployments);
        }
        return deployments;
      }, [] as Deployment[]),
    [pages],
  );

  // Auto fills the page
  useEffect(() => {
    if (
      ref.current &&
      ref.current?.scrollHeight <= document.documentElement.clientHeight &&
      hasNextPage &&
      !isFetching
    ) {
      fetchNextPage();
    }
  }, [isFetching, hasNextPage, ref, fetchNextPage]);

  // Loads more projects as scroll reaches the bottom
  useEffect(() => {
    const handleScroll = () => {
      if (
        window.innerHeight + document.documentElement.scrollTop + 5 >=
          document.documentElement.scrollHeight &&
        !isFetching &&
        hasNextPage
      ) {
        fetchNextPage();
      }
    };
    window.addEventListener("scroll", handleScroll);
    return () => window.removeEventListener("scroll", handleScroll);
  }, [isFetching, fetchNextPage, hasNextPage]);

  return {
    deployments,
    ref,
  };
};
