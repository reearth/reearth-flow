import { createLazyFileRoute } from '@tanstack/react-router'

import { Runs } from '@flow/features/Runs'

export const Route = createLazyFileRoute('/workspaces/$workspaceId_/runs/$tab')(
  {
    component: () => <Runs />,
  },
)
