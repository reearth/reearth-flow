/* eslint-disable */

// @ts-nocheck

// noinspection JSUnusedGlobalSymbols

// This file was automatically generated by TanStack Router.
// You should NOT make any changes in this file as it will be overwritten.
// Additionally, you should also exclude this file from your linter and/or formatter to prevent it from being checked or modified.

import { createFileRoute } from '@tanstack/react-router'

// Import Routes

import { Route as rootRoute } from './routes/__root'
import { Route as WorkspacesImport } from './routes/workspaces'
import { Route as IndexImport } from './routes/index'
import { Route as WorkspacesWorkspaceIdImport } from './routes/workspaces.$workspaceId'
import { Route as WorkspacesWorkspaceIdTriggersTabImport } from './routes/workspaces.$workspaceId_.triggers.$tab'
import { Route as WorkspacesWorkspaceIdSettingsTabImport } from './routes/workspaces.$workspaceId_.settings.$tab'
import { Route as WorkspacesWorkspaceIdJobsTabImport } from './routes/workspaces.$workspaceId_.jobs.$tab'
import { Route as WorkspacesWorkspaceIdDeploymentsTabImport } from './routes/workspaces.$workspaceId_.deployments.$tab'

// Create Virtual Routes

const WorkspacesWorkspaceIdProjectsProjectIdLazyImport = createFileRoute(
  '/workspaces_/$workspaceId_/projects_/$projectId',
)()

// Create/Update Routes

const WorkspacesRoute = WorkspacesImport.update({
  id: '/workspaces',
  path: '/workspaces',
  getParentRoute: () => rootRoute,
} as any)

const IndexRoute = IndexImport.update({
  id: '/',
  path: '/',
  getParentRoute: () => rootRoute,
} as any)

const WorkspacesWorkspaceIdRoute = WorkspacesWorkspaceIdImport.update({
  id: '/$workspaceId',
  path: '/$workspaceId',
  getParentRoute: () => WorkspacesRoute,
} as any)

const WorkspacesWorkspaceIdProjectsProjectIdLazyRoute =
  WorkspacesWorkspaceIdProjectsProjectIdLazyImport.update({
    id: '/workspaces_/$workspaceId_/projects_/$projectId',
    path: '/workspaces/$workspaceId/projects/$projectId',
    getParentRoute: () => rootRoute,
  } as any).lazy(() =>
    import('./routes/workspaces_.$workspaceId_.projects_.$projectId.lazy').then(
      (d) => d.Route,
    ),
  )

const WorkspacesWorkspaceIdTriggersTabRoute =
  WorkspacesWorkspaceIdTriggersTabImport.update({
    id: '/$workspaceId_/triggers/$tab',
    path: '/$workspaceId/triggers/$tab',
    getParentRoute: () => WorkspacesRoute,
  } as any)

const WorkspacesWorkspaceIdSettingsTabRoute =
  WorkspacesWorkspaceIdSettingsTabImport.update({
    id: '/$workspaceId_/settings/$tab',
    path: '/$workspaceId/settings/$tab',
    getParentRoute: () => WorkspacesRoute,
  } as any)

const WorkspacesWorkspaceIdJobsTabRoute =
  WorkspacesWorkspaceIdJobsTabImport.update({
    id: '/$workspaceId_/jobs/$tab',
    path: '/$workspaceId/jobs/$tab',
    getParentRoute: () => WorkspacesRoute,
  } as any)

const WorkspacesWorkspaceIdDeploymentsTabRoute =
  WorkspacesWorkspaceIdDeploymentsTabImport.update({
    id: '/$workspaceId_/deployments/$tab',
    path: '/$workspaceId/deployments/$tab',
    getParentRoute: () => WorkspacesRoute,
  } as any)

// Populate the FileRoutesByPath interface

declare module '@tanstack/react-router' {
  interface FileRoutesByPath {
    '/': {
      id: '/'
      path: '/'
      fullPath: '/'
      preLoaderRoute: typeof IndexImport
      parentRoute: typeof rootRoute
    }
    '/workspaces': {
      id: '/workspaces'
      path: '/workspaces'
      fullPath: '/workspaces'
      preLoaderRoute: typeof WorkspacesImport
      parentRoute: typeof rootRoute
    }
    '/workspaces/$workspaceId': {
      id: '/workspaces/$workspaceId'
      path: '/$workspaceId'
      fullPath: '/workspaces/$workspaceId'
      preLoaderRoute: typeof WorkspacesWorkspaceIdImport
      parentRoute: typeof WorkspacesImport
    }
    '/workspaces/$workspaceId_/deployments/$tab': {
      id: '/workspaces/$workspaceId_/deployments/$tab'
      path: '/$workspaceId/deployments/$tab'
      fullPath: '/workspaces/$workspaceId/deployments/$tab'
      preLoaderRoute: typeof WorkspacesWorkspaceIdDeploymentsTabImport
      parentRoute: typeof WorkspacesImport
    }
    '/workspaces/$workspaceId_/jobs/$tab': {
      id: '/workspaces/$workspaceId_/jobs/$tab'
      path: '/$workspaceId/jobs/$tab'
      fullPath: '/workspaces/$workspaceId/jobs/$tab'
      preLoaderRoute: typeof WorkspacesWorkspaceIdJobsTabImport
      parentRoute: typeof WorkspacesImport
    }
    '/workspaces/$workspaceId_/settings/$tab': {
      id: '/workspaces/$workspaceId_/settings/$tab'
      path: '/$workspaceId/settings/$tab'
      fullPath: '/workspaces/$workspaceId/settings/$tab'
      preLoaderRoute: typeof WorkspacesWorkspaceIdSettingsTabImport
      parentRoute: typeof WorkspacesImport
    }
    '/workspaces/$workspaceId_/triggers/$tab': {
      id: '/workspaces/$workspaceId_/triggers/$tab'
      path: '/$workspaceId/triggers/$tab'
      fullPath: '/workspaces/$workspaceId/triggers/$tab'
      preLoaderRoute: typeof WorkspacesWorkspaceIdTriggersTabImport
      parentRoute: typeof WorkspacesImport
    }
    '/workspaces_/$workspaceId_/projects_/$projectId': {
      id: '/workspaces_/$workspaceId_/projects_/$projectId'
      path: '/workspaces/$workspaceId/projects/$projectId'
      fullPath: '/workspaces/$workspaceId/projects/$projectId'
      preLoaderRoute: typeof WorkspacesWorkspaceIdProjectsProjectIdLazyImport
      parentRoute: typeof rootRoute
    }
  }
}

// Create and export the route tree

interface WorkspacesRouteChildren {
  WorkspacesWorkspaceIdRoute: typeof WorkspacesWorkspaceIdRoute
  WorkspacesWorkspaceIdDeploymentsTabRoute: typeof WorkspacesWorkspaceIdDeploymentsTabRoute
  WorkspacesWorkspaceIdJobsTabRoute: typeof WorkspacesWorkspaceIdJobsTabRoute
  WorkspacesWorkspaceIdSettingsTabRoute: typeof WorkspacesWorkspaceIdSettingsTabRoute
  WorkspacesWorkspaceIdTriggersTabRoute: typeof WorkspacesWorkspaceIdTriggersTabRoute
}

const WorkspacesRouteChildren: WorkspacesRouteChildren = {
  WorkspacesWorkspaceIdRoute: WorkspacesWorkspaceIdRoute,
  WorkspacesWorkspaceIdDeploymentsTabRoute:
    WorkspacesWorkspaceIdDeploymentsTabRoute,
  WorkspacesWorkspaceIdJobsTabRoute: WorkspacesWorkspaceIdJobsTabRoute,
  WorkspacesWorkspaceIdSettingsTabRoute: WorkspacesWorkspaceIdSettingsTabRoute,
  WorkspacesWorkspaceIdTriggersTabRoute: WorkspacesWorkspaceIdTriggersTabRoute,
}

const WorkspacesRouteWithChildren = WorkspacesRoute._addFileChildren(
  WorkspacesRouteChildren,
)

export interface FileRoutesByFullPath {
  '/': typeof IndexRoute
  '/workspaces': typeof WorkspacesRouteWithChildren
  '/workspaces/$workspaceId': typeof WorkspacesWorkspaceIdRoute
  '/workspaces/$workspaceId/deployments/$tab': typeof WorkspacesWorkspaceIdDeploymentsTabRoute
  '/workspaces/$workspaceId/jobs/$tab': typeof WorkspacesWorkspaceIdJobsTabRoute
  '/workspaces/$workspaceId/settings/$tab': typeof WorkspacesWorkspaceIdSettingsTabRoute
  '/workspaces/$workspaceId/triggers/$tab': typeof WorkspacesWorkspaceIdTriggersTabRoute
  '/workspaces/$workspaceId/projects/$projectId': typeof WorkspacesWorkspaceIdProjectsProjectIdLazyRoute
}

export interface FileRoutesByTo {
  '/': typeof IndexRoute
  '/workspaces': typeof WorkspacesRouteWithChildren
  '/workspaces/$workspaceId': typeof WorkspacesWorkspaceIdRoute
  '/workspaces/$workspaceId/deployments/$tab': typeof WorkspacesWorkspaceIdDeploymentsTabRoute
  '/workspaces/$workspaceId/jobs/$tab': typeof WorkspacesWorkspaceIdJobsTabRoute
  '/workspaces/$workspaceId/settings/$tab': typeof WorkspacesWorkspaceIdSettingsTabRoute
  '/workspaces/$workspaceId/triggers/$tab': typeof WorkspacesWorkspaceIdTriggersTabRoute
  '/workspaces/$workspaceId/projects/$projectId': typeof WorkspacesWorkspaceIdProjectsProjectIdLazyRoute
}

export interface FileRoutesById {
  __root__: typeof rootRoute
  '/': typeof IndexRoute
  '/workspaces': typeof WorkspacesRouteWithChildren
  '/workspaces/$workspaceId': typeof WorkspacesWorkspaceIdRoute
  '/workspaces/$workspaceId_/deployments/$tab': typeof WorkspacesWorkspaceIdDeploymentsTabRoute
  '/workspaces/$workspaceId_/jobs/$tab': typeof WorkspacesWorkspaceIdJobsTabRoute
  '/workspaces/$workspaceId_/settings/$tab': typeof WorkspacesWorkspaceIdSettingsTabRoute
  '/workspaces/$workspaceId_/triggers/$tab': typeof WorkspacesWorkspaceIdTriggersTabRoute
  '/workspaces_/$workspaceId_/projects_/$projectId': typeof WorkspacesWorkspaceIdProjectsProjectIdLazyRoute
}

export interface FileRouteTypes {
  fileRoutesByFullPath: FileRoutesByFullPath
  fullPaths:
    | '/'
    | '/workspaces'
    | '/workspaces/$workspaceId'
    | '/workspaces/$workspaceId/deployments/$tab'
    | '/workspaces/$workspaceId/jobs/$tab'
    | '/workspaces/$workspaceId/settings/$tab'
    | '/workspaces/$workspaceId/triggers/$tab'
    | '/workspaces/$workspaceId/projects/$projectId'
  fileRoutesByTo: FileRoutesByTo
  to:
    | '/'
    | '/workspaces'
    | '/workspaces/$workspaceId'
    | '/workspaces/$workspaceId/deployments/$tab'
    | '/workspaces/$workspaceId/jobs/$tab'
    | '/workspaces/$workspaceId/settings/$tab'
    | '/workspaces/$workspaceId/triggers/$tab'
    | '/workspaces/$workspaceId/projects/$projectId'
  id:
    | '__root__'
    | '/'
    | '/workspaces'
    | '/workspaces/$workspaceId'
    | '/workspaces/$workspaceId_/deployments/$tab'
    | '/workspaces/$workspaceId_/jobs/$tab'
    | '/workspaces/$workspaceId_/settings/$tab'
    | '/workspaces/$workspaceId_/triggers/$tab'
    | '/workspaces_/$workspaceId_/projects_/$projectId'
  fileRoutesById: FileRoutesById
}

export interface RootRouteChildren {
  IndexRoute: typeof IndexRoute
  WorkspacesRoute: typeof WorkspacesRouteWithChildren
  WorkspacesWorkspaceIdProjectsProjectIdLazyRoute: typeof WorkspacesWorkspaceIdProjectsProjectIdLazyRoute
}

const rootRouteChildren: RootRouteChildren = {
  IndexRoute: IndexRoute,
  WorkspacesRoute: WorkspacesRouteWithChildren,
  WorkspacesWorkspaceIdProjectsProjectIdLazyRoute:
    WorkspacesWorkspaceIdProjectsProjectIdLazyRoute,
}

export const routeTree = rootRoute
  ._addFileChildren(rootRouteChildren)
  ._addFileTypes<FileRouteTypes>()

/* ROUTE_MANIFEST_START
{
  "routes": {
    "__root__": {
      "filePath": "__root.tsx",
      "children": [
        "/",
        "/workspaces",
        "/workspaces_/$workspaceId_/projects_/$projectId"
      ]
    },
    "/": {
      "filePath": "index.tsx"
    },
    "/workspaces": {
      "filePath": "workspaces.tsx",
      "children": [
        "/workspaces/$workspaceId",
        "/workspaces/$workspaceId_/deployments/$tab",
        "/workspaces/$workspaceId_/jobs/$tab",
        "/workspaces/$workspaceId_/settings/$tab",
        "/workspaces/$workspaceId_/triggers/$tab"
      ]
    },
    "/workspaces/$workspaceId": {
      "filePath": "workspaces.$workspaceId.tsx",
      "parent": "/workspaces"
    },
    "/workspaces/$workspaceId_/deployments/$tab": {
      "filePath": "workspaces.$workspaceId_.deployments.$tab.tsx",
      "parent": "/workspaces"
    },
    "/workspaces/$workspaceId_/jobs/$tab": {
      "filePath": "workspaces.$workspaceId_.jobs.$tab.tsx",
      "parent": "/workspaces"
    },
    "/workspaces/$workspaceId_/settings/$tab": {
      "filePath": "workspaces.$workspaceId_.settings.$tab.tsx",
      "parent": "/workspaces"
    },
    "/workspaces/$workspaceId_/triggers/$tab": {
      "filePath": "workspaces.$workspaceId_.triggers.$tab.tsx",
      "parent": "/workspaces"
    },
    "/workspaces_/$workspaceId_/projects_/$projectId": {
      "filePath": "workspaces_.$workspaceId_.projects_.$projectId.lazy.tsx"
    }
  }
}
ROUTE_MANIFEST_END */
