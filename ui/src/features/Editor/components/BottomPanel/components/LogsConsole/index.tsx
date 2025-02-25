import { LogsTable } from "@flow/components/LogsTable";
import type { Log } from "@flow/types";

type LogsConsoleProps = {
  data: Log[];
};

const LogsConsole: React.FC<LogsConsoleProps> = ({ data }) => {
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
  return <LogsTable {...props} />;
};

export { LogsConsole };
