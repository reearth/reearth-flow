import { MockAsset, mockAssets } from "../data/asset";
import {
  mockCmsProjects,
  mockCmsModels,
  mockCmsItems,
  type MockCMSProject,
  type MockCMSModel,
  type MockCMSItem,
} from "../data/cms";
import { mockDeployments, type MockDeployment } from "../data/deployments";
import { mockJobs, mockLogs, type MockJob, type MockLog } from "../data/jobs";
import {
  mockProjects,
  type MockProject,
  type MockParameter,
} from "../data/projects";
import {
  mockUsers,
  getCurrentUser,
  getCurrentMe,
  type MockUser,
  type MockMe,
} from "../data/users";
import { mockWorkspaces, type MockWorkspace } from "../data/workspaces";

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
      if (obj.workspaceId && obj.parameters) return "Project";
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
    id: (user: MockUser) => user.id,
    name: (user: MockUser) => user.name,
    email: (user: MockUser) => user.email,
    host: (user: MockUser) => user.host,
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
    id: (workspace: MockWorkspace) => workspace.id,
    name: (workspace: MockWorkspace) => workspace.name,
    personal: (workspace: MockWorkspace) => workspace.personal,
    members: (workspace: MockWorkspace) =>
      workspace.members.map((member) => ({
        ...member,
        user: users.find((u) => u.id === member.userId),
      })),
    projects: (workspace: MockWorkspace, args: any) => {
      const workspaceProjects = projects.filter(
        (p) =>
          p.workspaceId === workspace.id &&
          (args.includeArchived || !p.isArchived),
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
    assets: (workspace: MockWorkspace, args: any) => {
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
    id: (project: MockProject) => project.id,
    name: (project: MockProject) => project.name,
    description: (project: MockProject) => project.description,
    workspaceId: (project: MockProject) => project.workspaceId,
    isArchived: (project: MockProject) => project.isArchived,
    isBasicAuthActive: (project: MockProject) => project.isBasicAuthActive,
    basicAuthUsername: (project: MockProject) => project.basicAuthUsername,
    basicAuthPassword: (project: MockProject) => project.basicAuthPassword,
    sharedToken: (project: MockProject) => project.sharedToken,
    version: (project: MockProject) => project.version,
    parameters: (project: MockProject) => project.parameters,
    createdAt: (project: MockProject) => project.createdAt,
    updatedAt: (project: MockProject) => project.updatedAt,
    workspace: (project: MockProject) =>
      workspaces.find((w) => w.id === project.workspaceId),
    deployment: (project: MockProject) =>
      deployments.find((d) => d.projectId === project.id && d.isHead),
  },

  Parameter: {
    id: (param: MockParameter) => param.id,
    name: (param: MockParameter) => param.name,
    type: (param: MockParameter) => param.type,
    value: (param: MockParameter) => param.value,
    required: (param: MockParameter) => param.required,
    index: (param: MockParameter) => param.index,
    projectId: (param: MockParameter) => param.projectId,
    createdAt: (param: MockParameter) => param.createdAt,
    updatedAt: (param: MockParameter) => param.updatedAt,
  },

  Job: {
    id: (job: MockJob) => job.id,
    deploymentId: (job: MockJob) => job.deploymentId,
    workspaceId: (job: MockJob) => job.workspaceId,
    status: (job: MockJob) => job.status,
    debug: (job: MockJob) => job.debug,
    startedAt: (job: MockJob) => job.startedAt,
    completedAt: (job: MockJob) => job.completedAt,
    logsURL: (job: MockJob) => job.logsURL,
    workerLogsURL: (job: MockJob) => job.workerLogsURL,
    outputURLs: (job: MockJob) => job.outputURLs,
    deployment: (job: MockJob) =>
      deployments.find((d) => d.id === job.deploymentId),
    workspace: (job: MockJob) =>
      workspaces.find((w) => w.id === job.workspaceId),
    logs: (job: MockJob, args: { since: string }) => {
      return logs.filter(
        (log) => log.jobId === job.id && log.timestamp >= args.since,
      );
    },
  },

  Log: {
    jobId: (log: MockLog) => log.jobId,
    nodeId: (log: MockLog) => log.nodeId,
    timestamp: (log: MockLog) => log.timestamp,
    logLevel: (log: MockLog) => log.logLevel,
    message: (log: MockLog) => log.message,
  },

  Deployment: {
    id: (deployment: MockDeployment) => deployment.id,
    projectId: (deployment: MockDeployment) => deployment.projectId,
    workspaceId: (deployment: MockDeployment) => deployment.workspaceId,
    version: (deployment: MockDeployment) => deployment.version,
    description: (deployment: MockDeployment) => deployment.description,
    isHead: (deployment: MockDeployment) => deployment.isHead,
    headId: (deployment: MockDeployment) => deployment.headId,
    workflowUrl: (deployment: MockDeployment) => deployment.workflowUrl,
    createdAt: (deployment: MockDeployment) => deployment.createdAt,
    updatedAt: (deployment: MockDeployment) => deployment.updatedAt,
    project: (deployment: MockDeployment) =>
      projects.find((p) => p.id === deployment.projectId),
    workspace: (deployment: MockDeployment) =>
      workspaces.find((w) => w.id === deployment.workspaceId),
  },

  Asset: {
    id: (asset: MockAsset) => asset.id,
    name: (asset: MockAsset) => asset.name,
    workspaceId: (asset: MockAsset) => asset.workspaceId,
    createdAt: (asset: MockAsset) => asset.createdAt,
    contentType: (asset: MockAsset) => asset.contentType,
    size: (asset: MockAsset) => asset.size,
    url: (asset: MockAsset) => asset.url,
    // workspace: (asset: MockAsset) =>
    //   workspaces.find((w) => w.id === asset.workspaceId),
  },

  // CMS Type resolvers
  CMSProject: {
    id: (cmsProject: MockCMSProject) => cmsProject.id,
    name: (cmsProject: MockCMSProject) => cmsProject.name,
    alias: (cmsProject: MockCMSProject) => cmsProject.alias,
    description: (cmsProject: MockCMSProject) => cmsProject.description,
    license: (cmsProject: MockCMSProject) => cmsProject.license,
    readme: (cmsProject: MockCMSProject) => cmsProject.readme,
    workspaceId: (cmsProject: MockCMSProject) => cmsProject.workspaceId,
    visibility: (cmsProject: MockCMSProject) => cmsProject.visibility,
    createdAt: (cmsProject: MockCMSProject) => cmsProject.createdAt,
    updatedAt: (cmsProject: MockCMSProject) => cmsProject.updatedAt,
  },

  CMSModel: {
    id: (cmsModel: MockCMSModel) => cmsModel.id,
    projectId: (cmsModel: MockCMSModel) => cmsModel.projectId,
    name: (cmsModel: MockCMSModel) => cmsModel.name,
    description: (cmsModel: MockCMSModel) => cmsModel.description,
    key: (cmsModel: MockCMSModel) => cmsModel.key,
    schema: (cmsModel: MockCMSModel) => cmsModel.schema,
    publicApiEp: (cmsModel: MockCMSModel) => cmsModel.publicApiEp,
    editorUrl: (cmsModel: MockCMSModel) => cmsModel.editorUrl,
    createdAt: (cmsModel: MockCMSModel) => cmsModel.createdAt,
    updatedAt: (cmsModel: MockCMSModel) => cmsModel.updatedAt,
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
    id: (cmsItem: MockCMSItem) => cmsItem.id,
    fields: (cmsItem: MockCMSItem) => cmsItem.fields,
    createdAt: (cmsItem: MockCMSItem) => cmsItem.createdAt,
    updatedAt: (cmsItem: MockCMSItem) => cmsItem.updatedAt,
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
        (p) =>
          p.workspaceId === args.workspaceId &&
          (args.includeArchived || !p.isArchived),
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
        (d) =>
          d.workspaceId === workspaceId &&
          d.projectId === projectId &&
          d.isHead,
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
        status: "COMPLETED",
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
      // For simplicity, return all mock items regardless of projectId/modelId
      const page = args.page || 1;
      const pageSize = args.pageSize || 10;
      const startIndex = (page - 1) * pageSize;
      const endIndex = startIndex + pageSize;

      return {
        items: cmsItems.slice(startIndex, endIndex),
        totalCount: cmsItems.length,
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
      const newUser: MockUser = {
        id: generateId("user"),
        name: "New User",
        email: "newuser@reearth.io",
        host: "reearth.io",
      };

      const newWorkspace = {
        id: generateId("workspace"),
        name: "Personal Workspace",
        personal: true,
        members: [{ userId: newUser.id, role: "OWNER" as const }],
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
            role: "OWNER" as const,
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
      const newProject: MockProject = {
        id: generateId("project"),
        name: input.name || "New Project",
        description: input.description || "",
        workspaceId: input.workspaceId,
        isArchived: input.archived || false,
        isBasicAuthActive: false,
        basicAuthUsername: "",
        basicAuthPassword: "",
        version: 1,
        parameters: [],
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
      let deployment = deployments.find(
        (d) => d.projectId === input.projectId && d.isHead,
      );
      if (!deployment) {
        deployment = {
          id: generateId("deployment"),
          projectId: input.projectId,
          workspaceId: input.workspaceId,
          version: `${project.version}.0.0`,
          description: "Auto-generated deployment",
          isHead: true,
          workflowUrl: `https://workflow-${project.id}.reearth-flow.com`,
          createdAt: new Date().toISOString(),
          updatedAt: new Date().toISOString(),
        };
        deployments.push(deployment);
      }

      // Create a new job
      const newJob: MockJob = {
        id: generateId("job"),
        deploymentId: deployment.id,
        workspaceId: input.workspaceId,
        status: "PENDING",
        debug: false,
        startedAt: new Date().toISOString(),
        outputURLs: [],
      };

      jobs.push(newJob);

      // Add initial log
      logs.push({
        jobId: newJob.id,
        timestamp: new Date().toISOString(),
        logLevel: "INFO",
        message: "Job queued for execution",
      });

      // Simulate job progression
      setTimeout(() => {
        const jobIndex = jobs.findIndex((j) => j.id === newJob.id);
        if (jobIndex !== -1) {
          jobs[jobIndex].status = "RUNNING";
          logs.push({
            jobId: newJob.id,
            timestamp: new Date().toISOString(),
            logLevel: "INFO",
            message: "Job started",
          });
        }
      }, 2000);

      return { job: newJob };
    },

    // Parameter mutations
    declareParameter: (_: any, args: { projectId: string; input: any }) => {
      const { projectId, input } = args;
      const project = projects.find((p) => p.id === projectId);

      if (!project) {
        throw new Error("Project not found");
      }

      const newParameter: MockParameter = {
        id: generateId("param"),
        name: input.name,
        type: input.type,
        value: input.value,
        required: input.required,
        index: input.index || project.parameters.length,
        projectId,
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
      };

      project.parameters.push(newParameter);
      return newParameter;
    },

    updateParameterValue: (_: any, args: { paramId: string; input: any }) => {
      const { paramId, input } = args;

      for (const project of projects) {
        const paramIndex = project.parameters.findIndex(
          (p) => p.id === paramId,
        );
        if (paramIndex !== -1) {
          project.parameters[paramIndex].value = input.value;
          project.parameters[paramIndex].updatedAt = new Date().toISOString();
          return project.parameters[paramIndex];
        }
      }

      throw new Error("Parameter not found");
    },

    updateParameterOrder: (_: any, args: { projectId: string; input: any }) => {
      const { projectId, input } = args;
      const project = projects.find((p) => p.id === projectId);

      if (!project) {
        throw new Error("Project not found");
      }

      const paramIndex = project.parameters.findIndex(
        (p) => p.id === input.paramId,
      );
      if (paramIndex !== -1) {
        project.parameters[paramIndex].index = input.newIndex;
      }

      return project.parameters;
    },

    removeParameter: (_: any, args: { input: { paramId: string } }) => {
      const { input } = args;

      for (const project of projects) {
        const paramIndex = project.parameters.findIndex(
          (p) => p.id === input.paramId,
        );
        if (paramIndex !== -1) {
          project.parameters.splice(paramIndex, 1);
          return true;
        }
      }

      return false;
    },

    // Asset mutations
    createAsset: (_: any, args: { input: any }) => {
      // Mock asset creation
      const newAsset: MockAsset = {
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
      const newDeployment: MockDeployment = {
        id: generateId("deployment"),
        projectId: input.projectId,
        workspaceId: input.workspaceId,
        version: "1.0.0",
        description: input.description,
        isHead: true,
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

      const newJob: MockJob = {
        id: generateId("job"),
        deploymentId: input.deploymentId,
        workspaceId: deployment.workspaceId,
        status: "PENDING",
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
        jobs[jobIndex].status === "PENDING" ||
        jobs[jobIndex].status === "RUNNING"
      ) {
        jobs[jobIndex].status = "CANCELLED";
        jobs[jobIndex].completedAt = new Date().toISOString();

        logs.push({
          jobId: input.jobId,
          timestamp: new Date().toISOString(),
          logLevel: "WARN",
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
                job.status = "RUNNING";
                yield { jobStatus: job.status };

                await new Promise((resolve) => setTimeout(resolve, 5000));
                job.status = "COMPLETED";
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
