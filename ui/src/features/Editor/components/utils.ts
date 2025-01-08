import { Action, Segregated } from "@flow/types";
// Functions below are used to filter actions based on the workflow type
export const isActionAllowedInMainWorkflow = (
  action: Action,
  isMainWorkflow?: boolean,
) => {
  if (!isMainWorkflow) return true;
  const actionName = action.name.toLowerCase();
  return !actionName.includes("router");
};

export const filterActionsForMainWorkflow = (
  actions: Action[] | undefined,
  isMainWorkflow: boolean,
): Action[] | undefined => {
  if (!isMainWorkflow) return actions;
  return actions?.filter((action) =>
    isActionAllowedInMainWorkflow(action, isMainWorkflow),
  );
};

export const filterSegregatedActionsForMainWorkflow = (
  segregatedData: Segregated | undefined,
  isMainWorkflow: boolean,
): Segregated | undefined => {
  if (!segregatedData || !isMainWorkflow) return segregatedData;
  return {
    ...segregatedData,
    byCategory: Object.fromEntries(
      Object.entries(segregatedData.byCategory).map(([key, actions]) => [
        key,
        filterActionsForMainWorkflow(actions, isMainWorkflow),
      ]),
    ),
    byType: Object.fromEntries(
      Object.entries(segregatedData.byType).map(([key, actions]) => [
        key,
        filterActionsForMainWorkflow(actions, isMainWorkflow),
      ]),
    ),
  };
};
