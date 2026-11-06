import WorkflowVariablesRunDialog from "@flow/features/WorkflowVariablesRunDialog";
import type { AnyWorkflowVariable } from "@flow/types";

type Props = {
  debugRunWorkflowVariables?: AnyWorkflowVariable[];
  workflowVariableDefaults?: AnyWorkflowVariable[];
  onDebugRunStart: (variables?: AnyWorkflowVariable[]) => Promise<void>;
  onDialogClose: () => void;
};

const DebugWorkflowVariablesDialog: React.FC<Props> = ({
  debugRunWorkflowVariables,
  workflowVariableDefaults,
  onDebugRunStart,
  onDialogClose,
}) => (
  <WorkflowVariablesRunDialog
    variables={debugRunWorkflowVariables}
    defaults={workflowVariableDefaults}
    onStart={onDebugRunStart}
    onDialogClose={onDialogClose}
  />
);

export default DebugWorkflowVariablesDialog;
