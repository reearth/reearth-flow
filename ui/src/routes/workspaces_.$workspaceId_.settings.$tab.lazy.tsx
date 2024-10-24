import { createLazyFileRoute } from '@tanstack/react-router'

import { WorkspaceIdWrapper } from '@flow/features/PageWrapper'
import { WorkspaceSettings as Settings } from '@flow/features/WorkspaceSettings'

export const Route = createLazyFileRoute(
  '/workspaces_/$workspaceId_/settings/$tab',
)({
  component: () => (
    <WorkspaceIdWrapper>
      <Settings />
    </WorkspaceIdWrapper>
  ),
})
