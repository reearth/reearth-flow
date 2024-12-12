import { Button } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { projectVariables } from "@flow/mock_data/projectVars";

import { ProjectVariable } from "./ProjectVariable";

const ProjectVariables: React.FC = () => {
  const t = useT();
  return (
    <div className="flex flex-col gap-4 px-1">
      <table className="w-full table-fixed border-collapse">
        <thead>
          <th className="pb-4 text-left font-thin">{t("Key")}</th>
          <th className="pb-4 text-left font-thin">{t("Value")}</th>
        </thead>
        <tbody>
          {projectVariables.map((variable, idx) => (
            <ProjectVariable
              className={`px-1 ${idx % 2 !== 0 ? "bg-card" : "bg-primary"}`}
              variable={variable}
            />
          ))}
        </tbody>
      </table>
      <Button className="self-end" size="sm" variant="outline">
        {t("Edit Project Variables")}
      </Button>
    </div>
  );
};

export { ProjectVariables };
