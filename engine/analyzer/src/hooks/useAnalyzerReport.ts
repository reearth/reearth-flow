import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import type {
  AnalyzerReport,
  ReportInfo,
  NodeSummary,
  NodeMemoryReport,
  NodeQueueReport,
} from "../types";

export function useReportsList() {
  const [reports, setReports] = useState<ReportInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<ReportInfo[]>("list_reports");
      setReports(result);
    } catch (e) {
      setError(e as string);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  return { reports, loading, error, refresh };
}

export function useAnalyzerReport(filename: string | null) {
  const [report, setReport] = useState<AnalyzerReport | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!filename) {
      setReport(null);
      return;
    }

    setLoading(true);
    setError(null);

    invoke<AnalyzerReport>("load_report", { filename })
      .then((result) => {
        setReport(result);
      })
      .catch((e) => {
        setError(e as string);
        setReport(null);
      })
      .finally(() => {
        setLoading(false);
      });
  }, [filename]);

  return { report, loading, error };
}

export function useReportNodes(filename: string | null) {
  const [nodes, setNodes] = useState<NodeSummary[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!filename) {
      setNodes([]);
      return;
    }

    setLoading(true);
    setError(null);

    invoke<NodeSummary[]>("get_report_nodes", { filename })
      .then((result) => {
        setNodes(result);
      })
      .catch((e) => {
        setError(e as string);
        setNodes([]);
      })
      .finally(() => {
        setLoading(false);
      });
  }, [filename]);

  return { nodes, loading, error };
}

export function useNodeMemoryData(filename: string | null, nodeId: string | null) {
  const [data, setData] = useState<NodeMemoryReport | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!filename || !nodeId) {
      setData(null);
      return;
    }

    setLoading(true);
    setError(null);

    invoke<NodeMemoryReport>("get_node_memory_data", { filename, nodeId })
      .then((result) => {
        setData(result);
      })
      .catch((e) => {
        setError(e as string);
        setData(null);
      })
      .finally(() => {
        setLoading(false);
      });
  }, [filename, nodeId]);

  return { data, loading, error };
}

export function useNodeQueueData(filename: string | null, nodeId: string | null) {
  const [data, setData] = useState<NodeQueueReport | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!filename || !nodeId) {
      setData(null);
      return;
    }

    setLoading(true);
    setError(null);

    invoke<NodeQueueReport>("get_node_queue_data", { filename, nodeId })
      .then((result) => {
        setData(result);
      })
      .catch((e) => {
        setError(e as string);
        setData(null);
      })
      .finally(() => {
        setLoading(false);
      });
  }, [filename, nodeId]);

  return { data, loading, error };
}

export function useReportsDirectory() {
  const [directory, setDirectory] = useState<string>("");
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    invoke<string>("get_reports_directory")
      .then(setDirectory)
      .finally(() => setLoading(false));
  }, []);

  const setReportsDirectory = useCallback(async (path: string) => {
    await invoke("set_reports_directory", { path });
    setDirectory(path);
  }, []);

  return { directory, loading, setReportsDirectory };
}
