import { useToast } from "@flow/features/NotificationSystem/useToast";
import { useT } from "@flow/lib/i18n";
import { CancelJob, Job } from "@flow/types";
import type { PaginationOptions } from "@flow/types/paginationOptions";

import { useQueries } from "./useQueries";

export const useJob = () => {
  const {
    useGetJobsQuery,
    useGetJobQuery,
    useGetNodeExecutionQuery,
    cancelJobMutation,
  } = useQueries();
  const { toast } = useToast();
  const t = useT();
  const useGetJobs = (
    workspaceId?: string,
    paginationOptions?: PaginationOptions,
  ) => {
    const { data, ...rest } = useGetJobsQuery(workspaceId, paginationOptions);
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

  const useGetNodeExecution = (
    jobId?: string,
    nodeId?: string,
    disabled?: boolean,
  ) => {
    const { data, ...rest } = useGetNodeExecutionQuery(jobId, nodeId, disabled);
    return {
      nodeExecution: data,
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
    useJobCancel,
    useGetNodeExecution,
  };
};
