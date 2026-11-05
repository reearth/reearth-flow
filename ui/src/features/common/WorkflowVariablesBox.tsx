import { useMemo } from "react";

import { useT } from "@flow/lib/i18n";

import { DetailsBox } from "./DetailsBox";
import type { DetailsBoxContent } from "./DetailsBox";
import { formatVariableValue } from "./formatVariableValue";

export type WorkflowVariableValue = {
  name: string;
  value: unknown;
};

type Props = {
  variables?: WorkflowVariableValue[];
};

const WorkflowVariablesBox: React.FC<Props> = ({ variables }) => {
  const t = useT();

  const content: DetailsBoxContent[] = useMemo(
    () =>
      (variables ?? []).map(({ name, value }) => ({
        id: name,
        name,
        value: formatVariableValue(value, t("N/A")),
      })),
    [variables, t],
  );

  if (!content.length) return null;

  return (
    <DetailsBox
      collapsible
      defaultCollapsed
      title={t("Workflow Variables")}
      content={content}
    />
  );
};

export { WorkflowVariablesBox };
