import { User } from "@flow/types/user";

export type YJsonPrimitive = string | number | boolean | null | Uint8Array;

export type YJsonValue =
  | YJsonPrimitive
  | YJsonValue[]
  | {
      [key: string]: YJsonValue;
    };

export type FlowMessage = {
  event: {
    tag: "Create" | "Join" | "Leave" | "Emit";
    content: {
      room_id?: string;
      data?: string;
    };
  };
  session_command?: SessionCommand;
};

export type SessionCommand = {
  tag:
    | "Start"
    | "End"
    | "Complete"
    | "CheckStatus"
    | "AddTask"
    | "RemoveTask"
    | "ListAllSnapshotsVersions"
    | "MergeUpdates";
  content: {
    user?: User;
    data?: Uint8Array;
  };
};
