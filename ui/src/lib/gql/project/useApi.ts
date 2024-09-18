import { useToast } from "@flow/features/NotificationSystem/useToast";
import { useT } from "@flow/lib/i18n";
import {
  CreateProject,
  DeleteProject,
  GetProject,
  GetWorkspaceProjects,
  RunProject,
  UpdateProject,
  Workflow,
} from "@flow/types";

import {
  CreateProjectInput,
  InputWorkflow,
  InputWorkflowEdge,
  InputWorkflowNode,
  UpdateProjectInput,
} from "../__gen__/graphql";

import { useQueries } from "./useQueries";

const DEFAULT_PORT = "default";

export const useProject = () => {
  const { toast } = useToast();
  const t = useT();

  const {
    createProjectMutation,
    deleteProjectMutation,
    updateProjectMutation,
    runProjectMutation,
    useGetProjectsInfiniteQuery,
    useGetProjectByIdQuery,
  } = useQueries();

  const createProject = async (
    input: CreateProjectInput,
  ): Promise<CreateProject> => {
    const { mutateAsync, ...rest } = createProjectMutation;
    try {
      const project = await mutateAsync(input);
      toast({
        title: t("Project Created"),
        description: t("Project has been successfully created."),
      });
      return { project, ...rest };
    } catch (_err) {
      return { project: undefined, ...rest };
    }
  };

  const useGetWorkspaceProjectsInfinite = (
    workspaceId?: string,
  ): GetWorkspaceProjects => {
    const { data, ...rest } = useGetProjectsInfiniteQuery(workspaceId);
    return {
      pages: data?.pages,
      ...rest,
    };
  };

  const useGetProject = (projectId?: string): GetProject => {
    const { data, ...rest } = useGetProjectByIdQuery(projectId);
    return {
      project: data,
      ...rest,
    };
  };

  const updateProject = async (
    input: UpdateProjectInput,
  ): Promise<UpdateProject> => {
    const { mutateAsync, ...rest } = updateProjectMutation;
    try {
      const project = await mutateAsync(input);
      return { project, ...rest };
    } catch (_err) {
      return { project: undefined, ...rest };
    }
  };

  const deleteProject = async (
    projectId: string,
    workspaceId: string,
  ): Promise<DeleteProject> => {
    const { mutateAsync, ...rest } = deleteProjectMutation;
    try {
      const data = await mutateAsync({ projectId, workspaceId });
      toast({
        title: t("Successful Deletion"),
        description: t(
          "Project has been successfully deleted from your workspace.",
        ),
        variant: "destructive",
      });
      return { projectId: data.projectId, ...rest };
    } catch (_err) {
      return { projectId: undefined, ...rest };
    }
  };

  const runProject = async (
    projectId: string,
    workspaceId: string,
    workflows: Workflow[],
  ): Promise<RunProject> => {
    const { mutateAsync, ...rest } = runProjectMutation;

    const gqlWorkflow: InputWorkflow = {
      id: projectId,
      name: "test",
      graphs: [],
      entryGraphId: workflows[0].id,
    };
    for (const w of workflows) {
      const nodes: InputWorkflowNode[] =
        w.nodes?.map((node) => ({
          id: node.id,
          name: node.data.name ?? "undefined node",
          type: node.type,
          position: node.position,
          parameters: node.data.params,
        })) ?? [];
      const edges: InputWorkflowEdge[] =
        w.edges?.map((edge) => ({
          id: edge.id,
          from: edge.source,
          to: edge.target,
          fromPort: edge.sourceHandle ?? DEFAULT_PORT,
          toPort: edge.targetHandle ?? DEFAULT_PORT,
        })) ?? [];
      gqlWorkflow.graphs.push({
        id: w.id,
        name: w.name ?? "undefined",
        nodes: nodes,
        edges: edges,
      });
    }

    console.log("gqlWorkflow", gqlWorkflow);

    try {
      const data = await mutateAsync({
        projectId,
        workspaceId,
        workflow: gqlWorkflow,
      });
      toast({
        title: t("Successful Deletion"),
        description: t(
          "Project has been successfully deleted from your workspace.",
        ),
        variant: "destructive",
      });
      console.log("data", data);
      return { projectId: data.projectId, started: data.started, ...rest };
    } catch (_err) {
      console.log("Errror", _err);
      return { projectId: undefined, ...rest };
    }
  };

  return {
    useGetWorkspaceProjectsInfinite,
    useGetProject,
    createProject,
    updateProject,
    deleteProject,
    runProject,
  };
};
