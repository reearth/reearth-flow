import { FlowLogo } from "@flow/components";
import BasicBoiler from "@flow/components/BasicBoiler";
import { LogsTable } from "@flow/components/LogsTable";
import { useT } from "@flow/lib/i18n";
import type { Log } from "@flow/types";

type LogsConsoleProps = {
  data: Log[];
};

const LogsConsole: React.FC<LogsConsoleProps> = ({ data }) => {
  const t = useT();
  const props = {
    columns: [
      {
        accessorKey: "timeStamp",
        header: "Timestamp",
      },
      {
        accessorKey: "status",
        header: "Status",
      },
      {
        accessorKey: "message",
        header: "message",
      },
    ],
    data,
    selectColumns: true,
    showFiltering: true,
  };

  const hasValidLogs = data.some(
    (log) => log.timeStamp || log.status || log.message,
  );

  if (!hasValidLogs) {
    return (
      <BasicBoiler
        text={t("No Logs Available")}
        icon={<FlowLogo className="size-16 text-accent" />}
      />
    );
  }

  return <LogsTable {...props} />;
};

export { LogsConsole };
