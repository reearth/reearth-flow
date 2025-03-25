import { createFileRoute, useParams } from "@tanstack/react-router";
import { useEffect, useState } from "react";

import { JobDetails } from "@flow/features/WorkspaceJobs/components";
import { useAuth } from "@flow/lib/auth";

export const Route = createFileRoute("/workspaces/$workspaceId/jobs_/$jobId")({
  component: RouteComponent,
});

function RouteComponent() {
  const { jobId } = useParams({ strict: false });

  const [accessToken, setAccessToken] = useState<string | undefined>(undefined);

  const { getAccessToken } = useAuth();

  useEffect(() => {
    if (!accessToken) {
      (async () => {
        const token = await getAccessToken();
        setAccessToken(token);
      })();
    }
  }, [accessToken, getAccessToken]);

  return jobId && accessToken ? (
    <div className="flex flex-1">
      <JobDetails jobId={jobId} accessToken={accessToken} />
    </div>
  ) : null;
}
