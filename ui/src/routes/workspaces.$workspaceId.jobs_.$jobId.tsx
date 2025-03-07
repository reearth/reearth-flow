import { createFileRoute, useParams } from '@tanstack/react-router'

import { JobDetails } from '@flow/features/WorkspaceJobs/components'

export const Route = createFileRoute('/workspaces/$workspaceId/jobs_/$jobId')({
  component: RouteComponent,
})

function RouteComponent() {
  const { jobId } = useParams({ strict: false })
  console.log('jobId', jobId)

  return jobId ? (
    <div className="flex flex-1">
      <JobDetails jobId={jobId} />
    </div>
  ) : null
}
