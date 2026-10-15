import type { ApiResponse } from "./api";
import type { Deployment } from "./deployment";
import type { Job } from "./job";
import type { Workflow } from "./workflow";

export enum ProjectOrderBy {
  CreatedAt = "createdAt",
  Name = "name",
  UpdatedAt = "updatedAt",
}

export type ProjectVisibility = "public" | "private";

// Mirrors the server's project rules (server/api/pkg/project/metadata.go).
export const PROJECT_ALIAS_PATTERN = /^[a-zA-Z0-9_-]{5,32}$/;
export const PROJECT_ALIAS_MIN_LENGTH = 5;
export const PROJECT_ALIAS_MAX_LENGTH = 32;
// Reserved for the default alias, p-<project ID>.
export const PROJECT_DEFAULT_ALIAS_PREFIX = "p-";
export const PROJECT_MAX_TOPICS = 200;
export const PROJECT_MAX_TOPIC_LENGTH = 64;

export type Project = {
  id: string;
  name: string;
  createdAt: string;
  updatedAt: string;
  description: string;
  workspaceId: string;
  workflows?: Workflow[];
  sharedToken?: string;
  deployment?: Deployment;
  isLocked?: boolean;
  alias: string;
  visibility: ProjectVisibility;
  readme: string;
  license: string;
  topics: string[];
  starCount: number;
  starred: boolean;
};

export type ProjectToImport = {
  name: string;
  description: string;
};

export type GetWorkspaceProjects = {
  pages?: ({ projects?: Project[] } | undefined)[];
  hasNextPage: boolean;
  fetchNextPage: () => void;
  isLoading: boolean;
  isFetching: boolean;
} & ApiResponse;

export type GetProject = {
  project?: Project;
  isLoading: boolean;
} & ApiResponse;

export type CreateProject = {
  project?: Project;
  // The reason the server gave, translated, when the project was not created.
  errorMessage?: string;
} & ApiResponse;

export type UpdateProject = {
  project?: Project;
  // The reason the server gave, translated, when the project was not updated.
  errorMessage?: string;
} & ApiResponse;

export type DeleteProject = {
  projectId?: string;
} & ApiResponse;

export type RunProject = {
  job?: Job;
} & ApiResponse;

export type ShareProject = {
  projectId?: string;
  sharingUrl?: string;
} & ApiResponse;

export type UnshareProject = {
  projectId?: string;
} & ApiResponse;
