const mockLogs = [
  {
    workflowId: "workflow-123",
    jobId: "job-456",
    ts: new Date().toISOString(),
    level: "ERROR",
    msg: "An error occurred while processing the task.",
  },
  {
    workflowId: "workflow-123",
    jobId: "job-456",
    ts: new Date(Date.now() - 1000 * 60).toISOString(),
    level: "WARN",
    msg: "The system is approaching resource limits.",
  },
  {
    workflowId: "workflow-124",
    jobId: "job-457",
    ts: new Date(Date.now() - 1000 * 120).toISOString(),
    level: "INFO",
    msg: "Task started successfully.",
  },
  {
    workflowId: "workflow-125",
    jobId: "job-458",
    ts: new Date(Date.now() - 1000 * 180).toISOString(),
    level: "DEBUG",
    msg: "Debugging the task execution flow.",
  },
  {
    workflowId: "workflow-126",
    jobId: "job-459",
    nodeId: "node-792",
    ts: new Date(Date.now() - 1000 * 240).toISOString(),
    level: "TRACE",
    msg: "Trace log for deep diagnostics.",
  },
  {
    workflowId: "workflow-123",
    jobId: "job-456",
    ts: new Date().toISOString(),
    level: "ERROR",
    msg: "An error occurred while processing the task.",
  },
  {
    workflowId: "workflow-123",
    jobId: "job-456",
    ts: new Date(Date.now() - 1000 * 60).toISOString(),
    level: "WARN",
    msg: "The system is approaching resource limits.",
  },
  {
    workflowId: "workflow-124",
    jobId: "job-457",
    ts: new Date(Date.now() - 1000 * 120).toISOString(),
    level: "INFO",
    msg: "Task started successfully.",
  },
  {
    workflowId: "workflow-125",
    jobId: "job-458",
    ts: new Date(Date.now() - 1000 * 180).toISOString(),
    level: "DEBUG",
    msg: "Debugging the task execution flow.",
  },
  {
    workflowId: "workflow-126",
    jobId: "job-459",
    nodeId: "node-792",
    ts: new Date(Date.now() - 1000 * 240).toISOString(),
    level: "TRACE",
    msg: "Trace log for deep diagnostics.",
  },
  {
    workflowId: "workflow-123",
    jobId: "job-456",
    ts: new Date().toISOString(),
    level: "ERROR",
    msg: "An error occurred while processing the task.",
  },
  {
    workflowId: "workflow-123",
    jobId: "job-456",
    ts: new Date(Date.now() - 1000 * 60).toISOString(),
    level: "WARN",
    msg: "The system is approaching resource limits.",
  },
  {
    workflowId: "workflow-124",
    jobId: "job-457",
    ts: new Date(Date.now() - 1000 * 120).toISOString(),
    level: "INFO",
    msg: "Task started successfully.",
  },
  {
    workflowId: "workflow-125",
    jobId: "job-458",
    ts: new Date(Date.now() - 1000 * 180).toISOString(),
    level: "DEBUG",
    msg: "Debugging the task execution flow.",
  },
  {
    workflowId: "workflow-126",
    jobId: "job-459",
    nodeId: "node-792",
    ts: new Date(Date.now() - 1000 * 240).toISOString(),
    level: "TRACE",
    msg: "Trace log for deep diagnostics.",
  },
  {
    workflowId: "workflow-123",
    jobId: "job-456",
    ts: new Date().toISOString(),
    level: "ERROR",
    msg: "An error occurred while processing the task.",
  },
  {
    workflowId: "workflow-123",
    jobId: "job-456",
    ts: new Date(Date.now() - 1000 * 60).toISOString(),
    level: "WARN",
    msg: "The system is approaching resource limits.",
  },
  {
    workflowId: "workflow-124",
    jobId: "job-457",
    ts: new Date(Date.now() - 1000 * 120).toISOString(),
    level: "INFO",
    msg: "Task started successfully.",
  },
  {
    workflowId: "workflow-125",
    jobId: "job-458",
    ts: new Date(Date.now() - 1000 * 180).toISOString(),
    level: "DEBUG",
    msg: "Debugging the task execution flow.",
  },
  {
    workflowId: "workflow-126",
    jobId: "job-459",
    nodeId: "node-792",
    ts: new Date(Date.now() - 1000 * 240).toISOString(),
    level: "TRACE",
    msg: "Trace log for deep diagnostics.",
  },
  {
    workflowId: "workflow-123",
    jobId: "job-456",
    ts: new Date().toISOString(),
    level: "ERROR",
    msg: "An error occurred while processing the task.",
  },
  {
    workflowId: "workflow-123",
    jobId: "job-456",
    ts: new Date(Date.now() - 1000 * 60).toISOString(),
    level: "WARN",
    msg: "The system is approaching resource limits.",
  },
  {
    workflowId: "workflow-124",
    jobId: "job-457",
    ts: new Date(Date.now() - 1000 * 120).toISOString(),
    level: "INFO",
    msg: "Task started successfully.",
  },
  {
    workflowId: "workflow-125",
    jobId: "job-458",
    ts: new Date(Date.now() - 1000 * 180).toISOString(),
    level: "DEBUG",
    msg: "Debugging the task execution flow.",
  },
  {
    workflowId: "workflow-126",
    jobId: "job-459",
    nodeId: "node-792",
    ts: new Date(Date.now() - 1000 * 240).toISOString(),
    level: "TRACE",
    msg: "Trace log for deep diagnostics.",
  },
  {
    workflowId: "workflow-123",
    jobId: "job-456",
    ts: new Date().toISOString(),
    level: "ERROR",
    msg: "An error occurred while processing the task.",
  },
  {
    workflowId: "workflow-123",
    jobId: "job-456",
    ts: new Date(Date.now() - 1000 * 60).toISOString(),
    level: "WARN",
    msg: "The system is approaching resource limits.",
  },
  {
    workflowId: "workflow-124",
    jobId: "job-457",
    ts: new Date(Date.now() - 1000 * 120).toISOString(),
    level: "INFO",
    msg: "Task started successfully.",
  },
  {
    workflowId: "workflow-125",
    jobId: "job-458",
    ts: new Date(Date.now() - 1000 * 180).toISOString(),
    level: "DEBUG",
    msg: "Debugging the task execution flow.",
  },
  {
    workflowId: "workflow-126",
    jobId: "job-459",
    nodeId: "node-792",
    ts: new Date(Date.now() - 1000 * 240).toISOString(),
    level: "TRACE",
    msg: "Trace log for deep diagnostics.",
  },
];

export default mockLogs;
