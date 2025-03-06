import { ColumnDef } from "@tanstack/react-table";

import { FlowLogo } from "@flow/components";
import BasicBoiler from "@flow/components/BasicBoiler";
import { LogsTable } from "@flow/components/LogsTable";
import { useT } from "@flow/lib/i18n";
import type { Log } from "@flow/types";
import { formatTimestamp } from "@flow/utils";

type LogsConsoleProps = {
  data: Log[];
};

const LogsConsole: React.FC<LogsConsoleProps> = ({ data }) => {
  const t = useT();
  const columns: ColumnDef<Log>[] = [
    {
      accessorKey: "timeStamp",
      header: t("Timestamp"),
      cell: ({ getValue }) => formatTimestamp(getValue<string>()),
    },
    {
      accessorKey: "status",
      header: t("Status"),
    },
    {
      accessorKey: "message",
      header: t("Message"),
    },
  ];

  const hasValidLogs = data.some(
    (log) => log.timeStamp || log.status || log.message,
  );

  if (!hasValidLogs) {
    return (
      <BasicBoiler
        className="h-full"
        text={t("No Logs Available")}
        icon={<FlowLogo className="size-16 text-accent" />}
      />
    );
  }

  return (
    <LogsTable columns={columns} data={data} selectColumns showFiltering />
  );
};

export { LogsConsole };
