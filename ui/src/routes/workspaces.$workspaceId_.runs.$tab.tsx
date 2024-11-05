import { createFileRoute } from '@tanstack/react-router'

import { Runs } from '@flow/features/Runs'

export const Route = createFileRoute('/workspaces/$workspaceId_/runs/$tab')({
  component: () => <Runs />,
})
