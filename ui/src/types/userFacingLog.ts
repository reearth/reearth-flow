export enum UserFacingLogLevel {
  Error = "ERROR",
  Info = "INFO",
  Success = "SUCCESS",
}

export type UserFacingLog = {
  jobId: string;
  timestamp: string;
  nodeId?: string;
  nodeName?: string;
  level: UserFacingLogLevel;
  message: string;
};
