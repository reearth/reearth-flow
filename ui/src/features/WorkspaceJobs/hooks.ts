import { useNavigate, useRouterState } from "@tanstack/react-router";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { useJob } from "@flow/lib/gql/job";
import { useCurrentWorkspace } from "@flow/stores";
import type { Job } from "@flow/types";
import { lastOfUrl as getJobId } from "@flow/utils";

import { RouteOption } from "../WorkspaceLeftPanel";

export default () => {
  const ref = useRef<HTMLDivElement>(null);
  const navigate = useNavigate();

  const [openJobRunDialog, setOpenJobRunDialog] = useState(false);
  const [currentWorkspace] = useCurrentWorkspace();

  const { useGetJobsInfinite } = useJob();

  const { pages, hasNextPage, isFetching, fetchNextPage } = useGetJobsInfinite(
    currentWorkspace?.id,
  );

  const {
    location: { pathname },
  } = useRouterState();

  const tab = getTab(pathname);

  const jobs: Job[] | undefined = useMemo(
    () =>
      pages?.reduce((jobs, page) => {
        if (page?.jobs) {
          jobs.push(...page.jobs);
        }
        return jobs;
      }, [] as Job[]),
    [pages],
  );

  const selectedJob = useMemo(
    () => jobs?.find((job) => job.id === tab),
    [tab, jobs],
  );

  const handleJobSelect = useCallback(
    (job: Job) =>
      navigate({
        to: `/workspaces/${currentWorkspace?.id}/jobs/${job.id}`,
      }),
    [currentWorkspace, navigate],
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
    ref,
    jobs,
    selectedJob,
    openJobRunDialog,
    setOpenJobRunDialog,
    handleJobSelect,
  };
};

const getTab = (pathname: string): RouteOption =>
  pathname.includes("all") ? "all" : getJobId(pathname);
