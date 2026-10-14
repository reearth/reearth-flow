import { useCallback, useEffect, useState } from "react";

import { useProject, useWorkspace } from "@flow/lib/gql";
import type {
  CreateProjectInput,
  UpdateProjectInput,
} from "@flow/lib/gql/__gen__/graphql";
import { toGqlProjectVisibility } from "@flow/lib/gql/convert";
import { useT } from "@flow/lib/i18n";
import {
  PROJECT_ALIAS_PATTERN,
  PROJECT_DEFAULT_ALIAS_PREFIX,
} from "@flow/types";
import type { Project, ProjectVisibility } from "@flow/types";

export type ProjectFormValues = {
  name: string;
  description: string;
  alias: string;
  visibility: ProjectVisibility;
  readme: string;
  license: string;
  topics: string[];
};

export type AliasStatus =
  // Creating with no alias: the server assigns p-<project ID>.
  | { kind: "default" }
  | { kind: "unchanged" }
  | { kind: "invalid"; message: string }
  | { kind: "checking" }
  | { kind: "available" }
  | { kind: "taken" }
  | { kind: "error"; message: string };

const ALIAS_CHECK_DELAY_MS = 400;

const initialValues = (project?: Project): ProjectFormValues => ({
  name: project?.name ?? "",
  description: project?.description ?? "",
  alias: project?.alias ?? "",
  // New projects are public.
  visibility: project?.visibility ?? "public",
  readme: project?.readme ?? "",
  license: project?.license ?? "",
  topics: project?.topics ?? [],
});

const sameTopics = (a: string[], b: string[]) =>
  a.length === b.length && a.every((topic, i) => topic === b[i]);

/**
 * Form state shared by the create and edit dialogs. Pass `project` to edit it;
 * without one the form creates a project in `workspaceId`.
 */
export default ({
  workspaceId,
  project,
}: {
  workspaceId?: string;
  project?: Project;
}) => {
  const t = useT();
  const { useCheckProjectAlias } = useProject();
  const { useGetWorkspacePolicy } = useWorkspace();

  const [values, setValues] = useState<ProjectFormValues>(() =>
    initialValues(project),
  );

  const setValue = useCallback(
    <K extends keyof ProjectFormValues>(key: K, value: ProjectFormValues[K]) =>
      setValues((current) => ({ ...current, [key]: value })),
    [],
  );

  const reset = useCallback(() => setValues(initialValues(project)), [project]);

  // Alias

  const { alias } = values;
  const aliasChanged = alias !== (project?.alias ?? "");

  const localAliasError = (() => {
    if (!aliasChanged || (!project && alias === "")) return undefined;
    if (!PROJECT_ALIAS_PATTERN.test(alias)) {
      return t("Use 5 to 32 letters, numbers, hyphens (-) or underscores (_).");
    }
    // The prefix is free only as the project's own default alias.
    if (
      alias.startsWith(PROJECT_DEFAULT_ALIAS_PREFIX) &&
      (!project || alias !== PROJECT_DEFAULT_ALIAS_PREFIX + project.id)
    ) {
      return t('Aliases starting with "p-" are reserved.');
    }
    return undefined;
  })();

  const [debouncedAlias, setDebouncedAlias] = useState(alias);
  useEffect(() => {
    const timeout = setTimeout(
      () => setDebouncedAlias(alias),
      ALIAS_CHECK_DELAY_MS,
    );
    return () => clearTimeout(timeout);
  }, [alias]);

  const shouldCheckAlias =
    aliasChanged &&
    alias !== "" &&
    !localAliasError &&
    debouncedAlias === alias;

  const { availability, errorMessage: aliasCheckError } = useCheckProjectAlias(
    workspaceId,
    shouldCheckAlias ? alias : undefined,
    project?.id,
  );

  const aliasStatus: AliasStatus = (() => {
    if (!project && alias === "") return { kind: "default" };
    if (!aliasChanged) return { kind: "unchanged" };
    if (localAliasError) return { kind: "invalid", message: localAliasError };
    if (!shouldCheckAlias) return { kind: "checking" };
    if (aliasCheckError) return { kind: "error", message: aliasCheckError };
    if (availability?.alias !== alias) return { kind: "checking" };
    return availability.available ? { kind: "available" } : { kind: "taken" };
  })();

  const aliasReady =
    aliasStatus.kind === "default" ||
    aliasStatus.kind === "unchanged" ||
    aliasStatus.kind === "available";

  // Visibility

  const { policy } = useGetWorkspacePolicy(workspaceId);
  // Unknown until the policy loads, and treated as not allowed meanwhile.
  const canCreatePrivate = policy?.canCreatePrivateProject;
  const wasPrivate = project?.visibility === "private";
  // A project made private before a downgrade stays private; only a change to
  // private needs the plan to allow it.
  const privateDisabled = !canCreatePrivate && !wasPrivate;
  const privateNeedsPaidPlan = canCreatePrivate === false && !wasPrivate;
  const losingPrivate =
    wasPrivate && canCreatePrivate === false && values.visibility === "public";

  const visibilityReady = !(values.visibility === "private" && privateDisabled);

  const canSubmit = !!values.name && aliasReady && visibilityReady;

  const toCreateInput = (createIn: string): CreateProjectInput => ({
    workspaceId: createIn,
    name: values.name,
    description: values.description,
    // Left out, the server assigns the default alias.
    alias: alias || undefined,
    visibility: toGqlProjectVisibility(values.visibility),
    readme: values.readme || undefined,
    license: values.license || undefined,
    topics: values.topics.length ? values.topics : undefined,
  });

  // Sends only what changed: a missing field is left as it is on the server.
  const toUpdateInput = (editing: Project): UpdateProjectInput => ({
    projectId: editing.id,
    name: values.name,
    description: values.description,
    alias: alias !== editing.alias ? alias : undefined,
    visibility:
      values.visibility !== editing.visibility
        ? toGqlProjectVisibility(values.visibility)
        : undefined,
    readme: values.readme !== editing.readme ? values.readme : undefined,
    license: values.license !== editing.license ? values.license : undefined,
    topics: sameTopics(values.topics, editing.topics)
      ? undefined
      : values.topics,
  });

  return {
    values,
    aliasStatus,
    privateDisabled,
    privateNeedsPaidPlan,
    losingPrivate,
    canSubmit,
    setValue,
    reset,
    toCreateInput,
    toUpdateInput,
  };
};
