import { Logs } from "@flow/components/Logs";
import mockLogs from "@flow/mock_data/logsv2Data";

const LogsConsole: React.FC = () => {
  console.log("mockLogs", mockLogs);

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
  return <Logs {...props} />;
};

export { LogsConsole };
