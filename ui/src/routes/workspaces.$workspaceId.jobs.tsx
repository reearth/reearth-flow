import { createFileRoute } from '@tanstack/react-router'

import { JobsManager } from '@flow/features/WorkspaceJobs'

export const Route = createFileRoute('/workspaces/$workspaceId/jobs')({
  component: () => <JobsManager />,
})
