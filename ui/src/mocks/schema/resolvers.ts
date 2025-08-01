import {
  AssetFragment,
  CmsItemFragment,
  CmsModelFragment,
  CmsProjectFragment,
  DeploymentFragment,
  JobFragment,
  JobStatus as GraphqlJobStatus,
  LogFragment,
  LogLevel as GraphqlLogLevel,
  ProjectFragment,
  WorkspaceFragment,
  Role as GraphqlRole,
  User as GraphqlUser,
} from "@flow/lib/gql/__gen__/graphql";

import { mockAssets } from "../data/asset";
import {
  mockCmsProjects,
  mockCmsModels,
  mockCmsItems,
} from "../data/cmsIntegration";
import { mockDeployments } from "../data/deployments";
import { mockJobs, mockLogs } from "../data/jobs";
import { mockProjects } from "../data/projects";
import {
  mockUsers,
  getCurrentUser,
  getCurrentMe,
  type MockMe,
} from "../data/users";
import { mockWorkspaces } from "../data/workspaces";

// In-memory storage for mutations
let users = [...mockUsers];
let workspaces = [...mockWorkspaces];
const assets = [...mockAssets];
let projects = [...mockProjects];
const jobs = [...mockJobs];
let deployments = [...mockDeployments];
const logs = [...mockLogs];
const cmsProjects = [...mockCmsProjects];
const cmsModels = [...mockCmsModels];
const cmsItems = [...mockCmsItems];

// Helper functions
const generateId = (prefix: string) =>
  `${prefix}-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;

const paginateResults = <T>(
  items: T[],
  pagination: {
    page: number;
    pageSize: number;
    orderBy?: string;
    orderDir?: "ASC" | "DESC";
  },
  keyword?: string,
) => {
  const { page, pageSize, orderBy, orderDir = "ASC" } = pagination;

  // Filter by keyword if provided
  let filteredItems = [...items];
  if (keyword && keyword.trim() !== "") {
    filteredItems = filteredItems.filter((item) => {
      // Search through all string properties of the item
      return Object.entries(item as any).some(([_, value]) => {
        if (typeof value === "string") {
          return value.toLowerCase().includes(keyword.toLowerCase());
        }
        return false;
      });
    });
  }

  // Sort if orderBy is specified
  if (orderBy) {
    filteredItems.sort((a, b) => {
      const aVal = (a as any)[orderBy];
      const bVal = (b as any)[orderBy];
      const comparison = aVal < bVal ? -1 : aVal > bVal ? 1 : 0;
      return orderDir === "DESC" ? -comparison : comparison;
    });
  }

  const startIndex = (page - 1) * pageSize;
  const endIndex = startIndex + pageSize;
  const paginatedItems = filteredItems.slice(startIndex, endIndex);

  return {
    nodes: paginatedItems,
    pageInfo: {
      totalCount: filteredItems.length,
      currentPage: page,
      totalPages: Math.ceil(filteredItems.length / pageSize) || 1, // Ensure at least 1 page
    },
    totalCount: filteredItems.length,
  };
};

export const resolvers = {
  // Scalar resolvers
  DateTime: {
    serialize: (value: string) => value,
    parseValue: (value: string) => value,
    parseLiteral: (ast: any) => ast.value,
  },

  URL: {
    serialize: (value: string) => value,
    parseValue: (value: string) => value,
    parseLiteral: (ast: any) => ast.value,
  },

  JSON: {
    serialize: (value: any) => value,
    parseValue: (value: any) => value,
    parseLiteral: (ast: any) => ast.value,
  },

  FileSize: {
    serialize: (value: number) => value,
    parseValue: (value: number) => value,
    parseLiteral: (ast: any) => parseInt(ast.value),
  },

  Any: {
    serialize: (value: any) => value,
    parseValue: (value: any) => value,
    parseLiteral: (ast: any) => ast.value,
  },

  Lang: {
    serialize: (value: string) => value,
    parseValue: (value: string) => value,
    parseLiteral: (ast: any) => ast.value,
  },

  // Interface resolvers
  Node: {
    __resolveType: (obj: any) => {
      if (obj.email && obj.auths) return "Me";
      if (obj.email) return "User";
      if (obj.personal !== undefined) return "Workspace";
      if (obj.workspaceId && obj.name && obj.description !== undefined)
        return "Project";
      if (
        obj.status &&
        ["PENDING", "RUNNING", "COMPLETED", "FAILED", "CANCELLED"].includes(
          obj.status,
        )
      )
        return "Job";
      if (obj.workflowUrl) return "Deployment";
      if (obj.contentType) return "Asset";
      if (obj.schedule || obj.webhook) return "Trigger";
      return null;
    },
  },

  // Type resolvers
  User: {
    id: (user: GraphqlUser) => user.id,
    name: (user: GraphqlUser) => user.name,
    email: (user: GraphqlUser) => user.email,
    host: (user: GraphqlUser) => user.host,
  },

  Me: {
    id: (me: MockMe) => me.id,
    name: (me: MockMe) => me.name,
    email: (me: MockMe) => me.email,
    lang: (me: MockMe) => me.lang,
    auths: (me: MockMe) => me.auths,
    myWorkspaceId: (me: MockMe) => me.myWorkspaceId,
    myWorkspace: (me: MockMe) =>
      workspaces.find((w) => w.id === me.myWorkspaceId),
    workspaces: () => workspaces,
  },

  Workspace: {
    id: (workspace: WorkspaceFragment) => workspace.id,
    name: (workspace: WorkspaceFragment) => workspace.name,
    personal: (workspace: WorkspaceFragment) => workspace.personal,
    members: (workspace: WorkspaceFragment) =>
      workspace.members.map((member) => ({
        ...member,
        user: users.find((u) => u.id === member.userId),
      })),
    projects: (workspace: WorkspaceFragment, args: any) => {
      const workspaceProjects = projects.filter(
        (p) => p.workspaceId === workspace.id,
      );

      if (args.pagination) {
        return paginateResults(workspaceProjects, args.pagination);
      }

      return {
        nodes: workspaceProjects,
        pageInfo: {
          totalCount: workspaceProjects.length,
          currentPage: 1,
          totalPages: 1,
        },
        totalCount: workspaceProjects.length,
      };
    },
    assets: (workspace: WorkspaceFragment, args: any) => {
      const workspaceAssets = assets.filter(
        (asset) => asset.workspaceId === workspace.id,
      );

      if (args.pagination) {
        return paginateResults(workspaceAssets, args.pagination);
      }

      return {
        nodes: workspaceAssets,
        pageInfo: {
          totalCount: workspaceAssets.length,
          currentPage: 1,
          totalPages: 1,
        },
        totalCount: workspaceAssets.length,
      };
    },
  },

  Project: {
    id: (project: ProjectFragment) => project.id,
    name: (project: ProjectFragment) => project.name,
    description: (project: ProjectFragment) => project.description,
    workspaceId: (project: ProjectFragment) => project.workspaceId,
    sharedToken: (project: ProjectFragment) => project.sharedToken,
    createdAt: (project: ProjectFragment) => project.createdAt,
    updatedAt: (project: ProjectFragment) => project.updatedAt,
    workspace: (project: ProjectFragment) =>
      workspaces.find((w) => w.id === project.workspaceId),
    deployment: (project: ProjectFragment) =>
      deployments.find((d) => d.projectId === project.id),
  },

  // Parameter: {
  //   id: (param: MockParameter) => param.id,
  //   name: (param: MockParameter) => param.name,
  //   type: (param: MockParameter) => param.type,
  //   value: (param: MockParameter) => param.value,
  //   required: (param: MockParameter) => param.required,
  //   index: (param: MockParameter) => param.index,
  //   projectId: (param: MockParameter) => param.projectId,
  //   createdAt: (param: MockParameter) => param.createdAt,
  //   updatedAt: (param: MockParameter) => param.updatedAt,
  // },

  Job: {
    id: (job: JobFragment) => job.id,
    workspaceId: (job: JobFragment) => job.workspaceId,
    status: (job: JobFragment) => job.status,
    debug: (job: JobFragment) => job.debug,
    startedAt: (job: JobFragment) => job.startedAt,
    completedAt: (job: JobFragment) => job.completedAt,
    logsURL: (job: JobFragment) => job.logsURL,
    outputURLs: (job: JobFragment) => job.outputURLs,
    deployment: (job: JobFragment) =>
      deployments.find((d) => d.id === job.deployment?.id),
    workspace: (job: JobFragment) =>
      workspaces.find((w) => w.id === job.workspaceId),
    logs: (job: JobFragment, args: { since: string }) => {
      return logs.filter(
        (log) => log.jobId === job.id && log.timestamp >= args.since,
      );
    },
  },

  Log: {
    jobId: (log: LogFragment) => log.jobId,
    nodeId: (log: LogFragment) => log.nodeId,
    timestamp: (log: LogFragment) => log.timestamp,
    logLevel: (log: LogFragment) => log.logLevel,
    message: (log: LogFragment) => log.message,
  },

  Deployment: {
    id: (deployment: DeploymentFragment) => deployment.id,
    projectId: (deployment: DeploymentFragment) => deployment.projectId,
    workspaceId: (deployment: DeploymentFragment) => deployment.workspaceId,
    version: (deployment: DeploymentFragment) => deployment.version,
    description: (deployment: DeploymentFragment) => deployment.description,
    workflowUrl: (deployment: DeploymentFragment) => deployment.workflowUrl,
    createdAt: (deployment: DeploymentFragment) => deployment.createdAt,
    updatedAt: (deployment: DeploymentFragment) => deployment.updatedAt,
    project: (deployment: DeploymentFragment) =>
      projects.find((p) => p.id === deployment.projectId),
    workspace: (deployment: DeploymentFragment) =>
      workspaces.find((w) => w.id === deployment.workspaceId),
  },

  Asset: {
    id: (asset: AssetFragment) => asset.id,
    name: (asset: AssetFragment) => asset.name,
    workspaceId: (asset: AssetFragment) => asset.workspaceId,
    createdAt: (asset: AssetFragment) => asset.createdAt,
    contentType: (asset: AssetFragment) => asset.contentType,
    size: (asset: AssetFragment) => asset.size,
    url: (asset: AssetFragment) => asset.url,
    // workspace: (asset: MockAsset) =>
    //   workspaces.find((w) => w.id === asset.workspaceId),
  },

  // CMS Type resolvers
  CMSProject: {
    id: (cmsProject: CmsProjectFragment) => cmsProject.id,
    name: (cmsProject: CmsProjectFragment) => cmsProject.name,
    alias: (cmsProject: CmsProjectFragment) => cmsProject.alias,
    description: (cmsProject: CmsProjectFragment) => cmsProject.description,
    license: (cmsProject: CmsProjectFragment) => cmsProject.license,
    readme: (cmsProject: CmsProjectFragment) => cmsProject.readme,
    workspaceId: (cmsProject: CmsProjectFragment) => cmsProject.workspaceId,
    visibility: (cmsProject: CmsProjectFragment) => cmsProject.visibility,
    createdAt: (cmsProject: CmsProjectFragment) => cmsProject.createdAt,
    updatedAt: (cmsProject: CmsProjectFragment) => cmsProject.updatedAt,
  },

  CMSModel: {
    id: (cmsModel: CmsModelFragment) => cmsModel.id,
    projectId: (cmsModel: CmsModelFragment) => cmsModel.projectId,
    name: (cmsModel: CmsModelFragment) => cmsModel.name,
    description: (cmsModel: CmsModelFragment) => cmsModel.description,
    key: (cmsModel: CmsModelFragment) => cmsModel.key,
    schema: (cmsModel: CmsModelFragment) => cmsModel.schema,
    publicApiEp: (cmsModel: CmsModelFragment) => cmsModel.publicApiEp,
    editorUrl: (cmsModel: CmsModelFragment) => cmsModel.editorUrl,
    createdAt: (cmsModel: CmsModelFragment) => cmsModel.createdAt,
    updatedAt: (cmsModel: CmsModelFragment) => cmsModel.updatedAt,
  },

  CMSSchema: {
    schemaId: (schema: any) => schema.schemaId,
    fields: (schema: any) => schema.fields,
  },

  CMSSchemaField: {
    fieldId: (field: any) => field.fieldId,
    name: (field: any) => field.name,
    type: (field: any) => field.type,
    key: (field: any) => field.key,
    description: (field: any) => field.description,
  },

  CMSItem: {
    id: (cmsItem: CmsItemFragment) => cmsItem.id,
    fields: (cmsItem: CmsItemFragment) => cmsItem.fields,
    createdAt: (cmsItem: CmsItemFragment) => cmsItem.createdAt,
    updatedAt: (cmsItem: CmsItemFragment) => cmsItem.updatedAt,
  },

  // Query resolvers
  Query: {
    node: (_: any, args: { id: string; type: string }) => {
      const { id, type } = args;
      switch (type) {
        case "USER":
          return users.find((u) => u.id === id);
        case "WORKSPACE":
          return workspaces.find((w) => w.id === id);
        case "PROJECT":
          return projects.find((p) => p.id === id);
        case "ASSET":
          return assets.find((a) => a.id === id);
        default:
          return null;
      }
    },

    nodes: (_: any, args: { id: string[]; type: string }) => {
      const { id: ids, type } = args;
      switch (type) {
        case "USER":
          return users.filter((u) => ids.includes(u.id));
        case "WORKSPACE":
          return workspaces.filter((w) => ids.includes(w.id));
        case "PROJECT":
          return projects.filter((p) => ids.includes(p.id));
        case "ASSET":
          return assets.filter((a) => ids.includes(a.id));
        default:
          return [];
      }
    },

    me: () => getCurrentMe(),

    searchUser: (_: any, args: { nameOrEmail: string }) => {
      const { nameOrEmail } = args;
      return users.find(
        (u) => u.name.includes(nameOrEmail) || u.email.includes(nameOrEmail),
      );
    },

    projects: (
      _: any,
      args: { workspaceId: string; includeArchived?: boolean; pagination: any },
    ) => {
      const workspaceProjects = projects.filter(
        (p) => p.workspaceId === args.workspaceId,
      );
      return paginateResults(workspaceProjects, args.pagination);
    },

    projectSharingInfo: (_: any, args: { projectId: string }) => {
      const project = projects.find((p) => p.id === args.projectId);
      return {
        projectId: args.projectId,
        enabled: !!project?.sharedToken,
        token: project?.sharedToken,
      };
    },

    sharedProject: (_: any, args: { token: string }) => {
      const project = projects.find((p) => p.sharedToken === args.token);
      if (!project) throw new Error("Project not found");
      return {
        project,
        sharedToken: args.token,
      };
    },

    assets: (
      _: any,
      args: {
        workspaceId: string;
        pagination: any;
        keyword?: string;
        sort?: string;
      },
    ) => {
      const workspaceAssets = assets.filter(
        (a) => a.workspaceId === args.workspaceId,
      );
      return paginateResults(workspaceAssets, args.pagination, args.keyword);
    },

    deployments: (_: any, args: { workspaceId: string; pagination: any }) => {
      const workspaceDeployments = deployments.filter(
        (d) => d.workspaceId === args.workspaceId,
      );
      return paginateResults(workspaceDeployments, args.pagination);
    },

    deploymentByVersion: (_: any, args: { input: any }) => {
      const { workspaceId, projectId, version } = args.input;
      return deployments.find(
        (d) =>
          d.workspaceId === workspaceId &&
          d.projectId === projectId &&
          d.version === version,
      );
    },

    deploymentHead: (_: any, args: { input: any }) => {
      const { workspaceId, projectId } = args.input;
      return deployments.find(
        (d) => d.workspaceId === workspaceId && d.projectId === projectId,
      );
    },

    deploymentVersions: (
      _: any,
      args: { workspaceId: string; projectId?: string },
    ) => {
      return deployments.filter(
        (d) =>
          d.workspaceId === args.workspaceId &&
          (!args.projectId || d.projectId === args.projectId),
      );
    },

    jobs: (_: any, args: { workspaceId: string; pagination: any }) => {
      const workspaceJobs = jobs.filter(
        (j) => j.workspaceId === args.workspaceId,
      );
      return paginateResults(workspaceJobs, args.pagination);
    },

    job: (_: any, args: { id: string }) => jobs.find((j) => j.id === args.id),

    nodeExecution: (_: any, args: { jobId: string; nodeId: string }) => {
      // Mock node execution data
      return {
        id: `exec-${args.jobId}-${args.nodeId}`,
        nodeId: args.nodeId,
        jobId: args.jobId,
        status: GraphqlJobStatus.Completed,
        startedAt: "2024-01-28T10:00:00Z",
        completedAt: "2024-01-28T10:05:00Z",
        logs: logs.filter(
          (l) => l.jobId === args.jobId && l.nodeId === args.nodeId,
        ),
      };
    },

    latestProjectSnapshot: (_: any, args: { projectId: string }) => {
      // Mock project document
      return {
        id: `doc-${args.projectId}`,
        projectId: args.projectId,
        content: { nodes: [], edges: [] },
        createdAt: "2024-01-28T10:00:00Z",
        updatedAt: "2024-01-28T10:00:00Z",
      };
    },

    projectSnapshot: (_: any, args: { projectId: string; version: string }) => {
      // Mock project snapshot
      return {
        id: `snapshot-${args.projectId}-${args.version}`,
        projectId: args.projectId,
        content: { nodes: [], edges: [] },
        createdAt: "2024-01-28T10:00:00Z",
        metadata: {
          id: `meta-${args.projectId}-${args.version}`,
          version: args.version,
          description: "Project snapshot",
          createdAt: "2024-01-28T10:00:00Z",
        },
      };
    },

    projectHistory: (_: any, args: { projectId: string; pagination: any }) => {
      // Mock project history
      const history = [
        {
          id: `meta-${args.projectId}-1`,
          version: "1.0.0",
          description: "Initial version",
          createdAt: "2024-01-01T10:00:00Z",
        },
      ];
      return paginateResults(history, args.pagination).nodes;
    },

    triggers: (_: any, args: { workspaceId: string; pagination: any }) => {
      // No triggers in mock data yet
      return paginateResults([], args.pagination);
    },

    // CMS queries
    cmsProject: (_: any, args: { projectIdOrAlias: string }) => {
      return cmsProjects.find(
        (p) =>
          p.id === args.projectIdOrAlias || p.alias === args.projectIdOrAlias,
      );
    },

    cmsProjects: (
      _: any,
      args: { workspaceId: string; publicOnly?: boolean },
    ) => {
      return cmsProjects.filter((p) => {
        if (p.workspaceId !== args.workspaceId) return false;
        if (args.publicOnly && p.visibility !== "PUBLIC") return false;
        return true;
      });
    },

    cmsModels: (_: any, args: { projectId: string }) => {
      return cmsModels.filter((m) => m.projectId === args.projectId);
    },

    cmsItems: (
      _: any,
      args: {
        projectId: string;
        modelId: string;
        page?: number;
        pageSize?: number;
      },
    ) => {
      // Filter items by projectId and modelId
      const filteredItems = cmsItems.filter(
        (item) =>
          item.projectId === args.projectId && item.modelId === args.modelId,
      );

      const page = args.page || 1;
      const pageSize = args.pageSize || 10;
      const startIndex = (page - 1) * pageSize;
      const endIndex = startIndex + pageSize;

      return {
        items: filteredItems.slice(startIndex, endIndex),
        totalCount: filteredItems.length,
      };
    },

    cmsModelExportUrl: (
      _: any,
      args: { projectId: string; modelId: string },
    ) => {
      return `https://cms.reearth-flow.com/api/export/${args.projectId}/${args.modelId}`;
    },
  },

  // Mutation resolvers
  Mutation: {
    // User mutations
    signup: () => {
      const newUser: GraphqlUser = {
        id: generateId("user"),
        name: "New User",
        email: "newuser@reearth.io",
        host: "reearth.io",
      };

      const newWorkspace = {
        id: generateId("workspace"),
        name: "Personal Workspace",
        personal: true,
        members: [{ userId: newUser.id, role: GraphqlRole.Owner }],
        createdAt: new Date().toISOString(),
      };

      users.push(newUser);
      workspaces.push(newWorkspace);

      return { user: newUser, workspace: newWorkspace };
    },

    updateMe: () => {
      // Mock update current user
      const me = getCurrentMe();
      return { me };
    },

    removeMyAuth: () => {
      const me = getCurrentMe();
      return { me };
    },

    deleteMe: (_: any, args: { input: { userId: string } }) => {
      const { input } = args;
      users = users.filter((u) => u.id !== input.userId);
      return { userId: input.userId };
    },

    // Workspace mutations
    createWorkspace: (_: any, args: { input: { name: string } }) => {
      const { input } = args;
      const currentUser = getCurrentUser();

      const newWorkspace = {
        id: generateId("workspace"),
        name: input.name,
        personal: false,
        members: [
          {
            userId: currentUser.id,
            role: GraphqlRole.Owner,
          },
        ],
        createdAt: new Date().toISOString(),
      };

      workspaces.push(newWorkspace);
      return { workspace: newWorkspace };
    },

    updateWorkspace: (_: any, args: { input: any }) => {
      const { input } = args;
      const workspaceIndex = workspaces.findIndex(
        (w) => w.id === input.workspaceId,
      );

      if (workspaceIndex === -1) {
        throw new Error("Workspace not found");
      }

      const updatedWorkspace = {
        ...workspaces[workspaceIndex],
        name: input.name,
      };

      workspaces[workspaceIndex] = updatedWorkspace;
      return { workspace: updatedWorkspace };
    },

    deleteWorkspace: (_: any, args: { input: { workspaceId: string } }) => {
      const { input } = args;
      workspaces = workspaces.filter((w) => w.id !== input.workspaceId);
      projects = projects.filter((p) => p.workspaceId !== input.workspaceId);
      return { workspaceId: input.workspaceId };
    },

    addMemberToWorkspace: (_: any, args: { input: any }) => {
      const { input } = args;
      const workspaceIndex = workspaces.findIndex(
        (w) => w.id === input.workspaceId,
      );

      if (workspaceIndex === -1) {
        throw new Error("Workspace not found");
      }

      workspaces[workspaceIndex].members.push({
        userId: input.userId,
        role: input.role,
      });

      return { workspace: workspaces[workspaceIndex] };
    },

    removeMemberFromWorkspace: (_: any, args: { input: any }) => {
      const { input } = args;
      const workspaceIndex = workspaces.findIndex(
        (w) => w.id === input.workspaceId,
      );

      if (workspaceIndex === -1) {
        throw new Error("Workspace not found");
      }

      workspaces[workspaceIndex].members = workspaces[
        workspaceIndex
      ].members.filter((m) => m.userId !== input.userId);

      return { workspace: workspaces[workspaceIndex] };
    },

    updateMemberOfWorkspace: (_: any, args: { input: any }) => {
      const { input } = args;
      const workspaceIndex = workspaces.findIndex(
        (w) => w.id === input.workspaceId,
      );

      if (workspaceIndex === -1) {
        throw new Error("Workspace not found");
      }

      const memberIndex = workspaces[workspaceIndex].members.findIndex(
        (m) => m.userId === input.userId,
      );

      if (memberIndex !== -1) {
        workspaces[workspaceIndex].members[memberIndex].role = input.role;
      }

      return { workspace: workspaces[workspaceIndex] };
    },

    // Project mutations
    createProject: (_: any, args: { input: any }) => {
      const { input } = args;
      const newProject: ProjectFragment = {
        id: generateId("project"),
        name: input.name || "New Project",
        description: input.description || "",
        workspaceId: input.workspaceId,
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
      };

      projects.push(newProject);
      return { project: newProject };
    },

    updateProject: (_: any, args: { input: any }) => {
      const { input } = args;
      const projectIndex = projects.findIndex((p) => p.id === input.projectId);

      if (projectIndex === -1) {
        throw new Error("Project not found");
      }

      const updatedProject = {
        ...projects[projectIndex],
        ...(input.name && { name: input.name }),
        ...(input.description && { description: input.description }),
        ...(input.archived !== undefined && { isArchived: input.archived }),
        ...(input.isBasicAuthActive !== undefined && {
          isBasicAuthActive: input.isBasicAuthActive,
        }),
        ...(input.basicAuthUsername && {
          basicAuthUsername: input.basicAuthUsername,
        }),
        ...(input.basicAuthPassword && {
          basicAuthPassword: input.basicAuthPassword,
        }),
        updatedAt: new Date().toISOString(),
      };

      projects[projectIndex] = updatedProject;
      return { project: updatedProject };
    },

    deleteProject: (_: any, args: { input: { projectId: string } }) => {
      const { input } = args;
      projects = projects.filter((p) => p.id !== input.projectId);
      return { projectId: input.projectId };
    },

    runProject: (_: any, args: { input: any }) => {
      const { input } = args;
      const project = projects.find((p) => p.id === input.projectId);

      if (!project) {
        throw new Error("Project not found");
      }

      // Create a deployment if it doesn't exist
      let deployment = deployments.find((d) => d.projectId === input.projectId);
      if (!deployment) {
        deployment = {
          id: generateId("deployment"),
          projectId: input.projectId,
          workspaceId: input.workspaceId,
          description: "Auto-generated deployment",
          workflowUrl: `https://workflow-${project.id}.reearth-flow.com`,
          version: "1.0.0",
          createdAt: new Date().toISOString(),
          updatedAt: new Date().toISOString(),
        };
        if (deployment) {
          deployments.push(deployment);
        }
      }

      // Create a new job
      const newJob: JobFragment = {
        id: generateId("job"),
        workspaceId: input.workspaceId,
        status: GraphqlJobStatus.Pending,
        debug: false,
        startedAt: new Date().toISOString(),
        outputURLs: [],
      };

      jobs.push(newJob);

      // Add initial log
      logs.push({
        jobId: newJob.id,
        timestamp: new Date().toISOString(),
        logLevel: GraphqlLogLevel.Info,
        message: "Job queued for execution",
      });

      // Simulate job progression
      setTimeout(() => {
        const jobIndex = jobs.findIndex((j) => j.id === newJob.id);
        if (jobIndex !== -1) {
          jobs[jobIndex].status = GraphqlJobStatus.Running;
          logs.push({
            jobId: newJob.id,
            timestamp: new Date().toISOString(),
            logLevel: GraphqlLogLevel.Info,
            message: "Job started",
          });
        }
      }, 2000);

      return { job: newJob };
    },

    // Parameter mutations
    // declareParameter: (_: any, args: { projectId: string; input: any }) => {
    //   const { projectId, input } = args;
    //   const project = projects.find((p) => p.id === projectId);

    //   if (!project) {
    //     throw new Error("Project not found");
    //   }

    //   const newParameter: {MockParameter} = {
    //     id: generateId("param"),
    //     name: input.name,
    //     type: input.type,
    //     value: input.value,
    //     required: input.required,
    //     index: input.index || project.parameters.length,
    //     projectId,
    //     createdAt: new Date().toISOString(),
    //     updatedAt: new Date().toISOString(),
    //   };

    //   project.parameters.push(newParameter);
    //   return newParameter;
    // },

    // updateParameterValue: (_: any, args: { paramId: string; input: any }) => {
    //   const { paramId, input } = args;

    //   for (const project of projects) {
    //     const paramIndex = project.parameters.findIndex(
    //       (p) => p.id === paramId,
    //     );
    //     if (paramIndex !== -1) {
    //       project.parameters[paramIndex].value = input.value;
    //       project.parameters[paramIndex].updatedAt = new Date().toISOString();
    //       return project.parameters[paramIndex];
    //     }
    //   }

    //   throw new Error("Parameter not found");
    // },

    // updateParameterOrder: (_: any, args: { projectId: string; input: any }) => {
    //   const { projectId, input } = args;
    //   const project = projects.find((p) => p.id === projectId);

    //   if (!project) {
    //     throw new Error("Project not found");
    //   }

    //   const paramIndex = project.parameters.findIndex(
    //     (p) => p.id === input.paramId,
    //   );
    //   if (paramIndex !== -1) {
    //     project.parameters[paramIndex].index = input.newIndex;
    //   }

    //   return project.parameters;
    // },

    // removeParameter: (_: any, args: { input: { paramId: string } }) => {
    //   const { input } = args;

    //   for (const project of projects) {
    //     const paramIndex = project.parameters.findIndex(
    //       (p) => p.id === input.paramId,
    //     );
    //     if (paramIndex !== -1) {
    //       project.parameters.splice(paramIndex, 1);
    //       return true;
    //     }
    //   }

    //   return false;
    // },

    // Asset mutations
    createAsset: (_: any, args: { input: any }) => {
      // Mock asset creation
      const newAsset: AssetFragment = {
        id: generateId("asset"),
        name: "New Asset",
        contentType: "image/png",
        fileName: "asset-1.png",
        size: 1024,
        url: "https://assets.reearth.io/asset-1.png",
        workspaceId: args.input.workspaceId,
        createdAt: new Date().toISOString(),
        uuid: "uuid-1",
        flatFiles: false,
        public: true,
      };

      assets.push(newAsset);
      return { asset: newAsset };
    },

    updateAsset: (_: any, args: { input: any }) => {
      const { input } = args;
      const assetIndex = assets.findIndex((a) => a.id === input.assetId);

      if (assetIndex === -1) {
        throw new Error("Asset not found");
      }

      const updatedAsset = {
        ...assets[assetIndex],
        name: input.name,
      };

      assets[assetIndex] = updatedAsset;
      return { asset: updatedAsset };
    },

    deleteAsset: (_: any, args: { input: { assetId: string } }) => {
      const { input } = args;
      const assetIndex = assets.findIndex((a) => a.id === input.assetId);

      if (assetIndex !== -1) {
        assets.splice(assetIndex, 1);
      }

      return { assetId: input.assetId };
    },

    // Deployment mutations
    createDeployment: (_: any, args: { input: any }) => {
      const { input } = args;
      const newDeployment: DeploymentFragment = {
        id: generateId("deployment"),
        projectId: input.projectId,
        workspaceId: input.workspaceId,
        version: "1.0.0",
        description: input.description,
        workflowUrl: `https://workflow-${generateId("flow")}.reearth-flow.com`,
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
      };

      deployments.push(newDeployment);
      return { deployment: newDeployment };
    },

    updateDeployment: (_: any, args: { input: any }) => {
      const { input } = args;
      const deploymentIndex = deployments.findIndex(
        (d) => d.id === input.deploymentId,
      );

      if (deploymentIndex === -1) {
        throw new Error("Deployment not found");
      }

      if (input.description) {
        deployments[deploymentIndex].description = input.description;
      }
      deployments[deploymentIndex].updatedAt = new Date().toISOString();

      return { deployment: deployments[deploymentIndex] };
    },

    deleteDeployment: (_: any, args: { input: { deploymentId: string } }) => {
      const { input } = args;
      deployments = deployments.filter((d) => d.id !== input.deploymentId);
      return { deploymentId: input.deploymentId };
    },

    executeDeployment: (_: any, args: { input: { deploymentId: string } }) => {
      const { input } = args;
      const deployment = deployments.find((d) => d.id === input.deploymentId);

      if (!deployment) {
        throw new Error("Deployment not found");
      }

      const newJob: JobFragment = {
        id: generateId("job"),
        workspaceId: deployment.workspaceId,
        status: GraphqlJobStatus.Pending,
        debug: false,
        startedAt: new Date().toISOString(),
        outputURLs: [],
      };

      jobs.push(newJob);
      return { job: newJob };
    },

    // Job mutations
    cancelJob: (_: any, args: { input: { jobId: string } }) => {
      const { input } = args;
      const jobIndex = jobs.findIndex((j) => j.id === input.jobId);

      if (jobIndex === -1) {
        throw new Error("Job not found");
      }

      if (
        jobs[jobIndex].status === GraphqlJobStatus.Pending ||
        jobs[jobIndex].status === GraphqlJobStatus.Running
      ) {
        jobs[jobIndex].status = GraphqlJobStatus.Cancelled;
        jobs[jobIndex].completedAt = new Date().toISOString();

        logs.push({
          jobId: input.jobId,
          timestamp: new Date().toISOString(),
          logLevel: GraphqlLogLevel.Warn,
          message: "Job cancelled by user request",
        });
      }

      return { job: jobs[jobIndex] };
    },
  },

  // Subscription resolvers (simplified for mock)
  Subscription: {
    jobStatus: {
      subscribe: (_: any, args: { jobId: string }) => {
        const job = jobs.find((j) => j.id === args.jobId);
        return {
          [Symbol.asyncIterator]: async function* () {
            if (job) {
              yield { jobStatus: job.status };

              // Simulate status changes
              if (job.status === "PENDING") {
                await new Promise((resolve) => setTimeout(resolve, 2000));
                job.status = GraphqlJobStatus.Running;
                yield { jobStatus: job.status };

                await new Promise((resolve) => setTimeout(resolve, 5000));
                job.status = GraphqlJobStatus.Completed;
                job.completedAt = new Date().toISOString();
                yield { jobStatus: job.status };
              }
            }
          },
        };
      },
    },

    logs: {
      subscribe: (_: any, args: { jobId: string }) => {
        return {
          [Symbol.asyncIterator]: async function* () {
            const jobLogs = logs.filter((l) => l.jobId === args.jobId);
            for (const log of jobLogs) {
              yield { logs: log };
              await new Promise((resolve) => setTimeout(resolve, 1000));
            }
          },
        };
      },
    },
  },
};
