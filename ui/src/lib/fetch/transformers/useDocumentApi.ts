import { useMutation, useQuery, UseQueryResult } from "@tanstack/react-query";

// Document API Types
export interface DocumentResponse {
  id: string;
  updates: number[];
  version: number;
  timestamp: string;
}

export interface HistoryResponse {
  updates: number[];
  version: number;
  timestamp: string;
}

export interface HistoryMetadataResponse {
  version: number;
  timestamp: string;
}

export interface RollbackRequest {
  doc_id: string;
  version: number;
}

export interface CreateSnapshotRequest {
  doc_id: string;
  version: number;
  name?: string;
}

// API Base URL - fixed to 127.0.0.1:8080
const getApiBaseUrl = () => {
  return "http://127.0.0.1:8080";
};

// Fetcher function for document API
const documentFetcher = async <T>(url: string, options?: RequestInit): Promise<T> => {
  const baseUrl = getApiBaseUrl();
  const fullUrl = `${baseUrl}/api${url}`;
  
  const response = await fetch(fullUrl, {
    headers: {
      "Content-Type": "application/json",
      ...options?.headers,
    },
    ...options,
  });

  if (!response.ok) {
    throw new Error(`HTTP error! status: ${response.status}`);
  }

  return response.json();
};

export const useDocumentApi = () => {
  // Get latest document
  const useGetLatestDocument = (docId: string): UseQueryResult<DocumentResponse> => {
    return useQuery({
      queryKey: ["document", docId, "latest"],
      queryFn: () => documentFetcher<DocumentResponse>(`/document/${docId}`),
      enabled: !!docId,
    });
  };

  // Get document history
  const useGetDocumentHistory = (docId: string): UseQueryResult<HistoryResponse[]> => {
    return useQuery({
      queryKey: ["document", docId, "history"],
      queryFn: () => documentFetcher<HistoryResponse[]>(`/document/${docId}/history`),
      enabled: !!docId,
    });
  };

  // Get document history metadata
  const useGetDocumentHistoryMetadata = (docId: string): UseQueryResult<HistoryMetadataResponse[]> => {
    return useQuery({
      queryKey: ["document", docId, "history", "metadata"],
      queryFn: () => documentFetcher<HistoryMetadataResponse[]>(`/document/${docId}/history/metadata`),
      enabled: !!docId,
    });
  };

  // Get document history by version
  const useGetDocumentHistoryByVersion = (docId: string, version: number): UseQueryResult<HistoryResponse> => {
    return useQuery({
      queryKey: ["document", docId, "history", "version", version],
      queryFn: () => documentFetcher<HistoryResponse>(`/document/${docId}/history/version/${version}`),
      enabled: !!docId && version > 0,
    });
  };

  // Rollback document mutation
  const useRollbackDocument = () => {
    return useMutation({
      mutationFn: async ({ docId, version }: { docId: string; version: number }) => {
        return documentFetcher<DocumentResponse>(`/document/${docId}/rollback`, {
          method: "POST",
          body: JSON.stringify({
            doc_id: docId,
            version,
          }),
        });
      },
    });
  };

  // Flush document to GCS mutation
  const useFlushDocumentToGcs = () => {
    return useMutation({
      mutationFn: (docId: string) => {
        return documentFetcher(`/document/${docId}/flush`, {
          method: "POST",
        });
      },
    });
  };

  // Create snapshot mutation
  const useCreateSnapshot = () => {
    return useMutation({
      mutationFn: async ({ docId, version, name }: { docId: string; version: number; name?: string }) => {
        return documentFetcher<DocumentResponse>("/document/snapshot", {
          method: "POST",
          body: JSON.stringify({
            doc_id: docId,
            version,
            name,
          }),
        });
      },
    });
  };

  return {
    useGetLatestDocument,
    useGetDocumentHistory,
    useGetDocumentHistoryMetadata,
    useGetDocumentHistoryByVersion,
    useRollbackDocument,
    useFlushDocumentToGcs,
    useCreateSnapshot,
  };
}; 