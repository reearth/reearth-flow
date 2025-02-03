import { LogsTable } from "@flow/components/LogsTable";
import mockLogs from "@flow/mock_data/logsv2Data";

const LogsConsole: React.FC = () => {
  const props = {
    columns: [
      {
        accessorKey: "timestamp",
        header: "Timestamp",
      },
      {
        accessorKey: "logLevel",
        header: "Status",
      },
      {
        accessorKey: "transformer",
        header: "Transformer",
      },
      {
        accessorKey: "message",
        header: "message",
      },
    ],
    data: mockLogs,
    selectColumns: true,
    showFiltering: true,
  };
  return <LogsTable {...props} />;
};

export { LogsConsole };
