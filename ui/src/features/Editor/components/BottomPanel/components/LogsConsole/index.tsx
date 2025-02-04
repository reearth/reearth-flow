import { Logs } from "@flow/components/Logs";
import type { Log } from "@flow/types";

type LogsConsoleProps = {
  data: Log[];
};

const LogsConsole: React.FC<LogsConsoleProps> = ({ data }) => {
  const props = {
    columns: [
      {
        accessorKey: "ts",
        header: "Timestamp",
      },
      {
        accessorKey: "level",
        header: "Status",
      },
      {
        accessorKey: "msg",
        header: "message",
      },
    ],
    data,
    selectColumns: true,
    showFiltering: true,
  };
  return <Logs {...props} />;
};

export { LogsConsole };
