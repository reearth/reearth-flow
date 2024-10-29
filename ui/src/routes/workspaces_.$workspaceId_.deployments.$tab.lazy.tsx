import { createLazyFileRoute } from '@tanstack/react-router'

import { WorkspaceIdWrapper } from '@flow/features/PageWrapper'
import { Runs } from '@flow/features/Runs'

export const Route = createLazyFileRoute(
  '/workspaces_/$workspaceId_/deployments/$tab',
)({
  component: () => (
    <WorkspaceIdWrapper>
      <Runs />
    </WorkspaceIdWrapper>
  ),
})
