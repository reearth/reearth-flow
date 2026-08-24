import { useToast } from "@flow/features/NotificationSystem/useToast";
import { useT } from "@flow/lib/i18n";
import { CancelJob, Job } from "@flow/types";
import type { PaginationOptions } from "@flow/types/paginationOptions";

import { useQueries } from "./useQueries";

export const useJob = () => {
  const {
    useGetJobsQuery,
    useGetJobQuery,
    useGetNodeExecutionsQuery,
    cancelJobMutation,
  } = useQueries();
  const { toast } = useToast();
  const t = useT();
  const useGetJobs = (
    workspaceId?: string,
    keyword?: string,
    paginationOptions?: PaginationOptions,
  ) => {
    const { data, ...rest } = useGetJobsQuery(
      workspaceId,
      keyword,
      paginationOptions,
    );
    return {
      page: data,
      ...rest,
    };
  };

  const useGetJob = (jobId?: string) => {
    const { data, ...rest } = useGetJobQuery(jobId);
    return {
      job: data,
      ...rest,
    };
  };

  /**
   * `poll` should track whether the job is still running: node executions carry
   * diagnostics and feature counts that never arrive over a subscription, so a
   * live job has to be re-read on an interval to keep them current.
   */
  const useGetNodeExecutions = (jobId?: string, poll?: boolean) => {
    const { data, ...rest } = useGetNodeExecutionsQuery(jobId, poll);
    return {
      nodeExecutions: data,
      ...rest,
    };
  };

  const useJobCancel = async (jobId: string): Promise<CancelJob> => {
    const { mutateAsync, ...rest } = cancelJobMutation;
    try {
      const job: Job | undefined = await mutateAsync({
        jobId,
      });
      toast({
        title: t("Job Cancelled"),
        description: t("Job has been successfully cancelled."),
      });
      return { job, ...rest };
    } catch (_err) {
      toast({
        title: t("Job Could Not Be Cancelled"),
        description: t("There was an error when cancelling the job."),
        variant: "destructive",
      });
      return { job: undefined, ...rest };
    }
  };

  return {
    useGetJob,
    useGetJobs,
    useGetNodeExecutions,
    useJobCancel,
  };
};
