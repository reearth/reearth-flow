import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { SupportedDataTypes } from "@flow/utils/fetchAndReadGeoData";
import { streamJsonl } from "@flow/utils/streaming";
import type { StreamingState, StreamingProgress } from "@flow/utils/streaming";

type GeometryType = 'FlowGeometry2D' | 'FlowGeometry3D' | 'CityGmlGeometry' | 'Unknown' | null;

type UseStreamingDebugRunQueryOptions = {
  enabled?: boolean;
  batchSize?: number;
  chunkSize?: number;
  onProgress?: (progress: StreamingProgress) => void;
  onError?: (error: Error) => void;
};

function detectGeometryType(feature: any): GeometryType {
  const geometryValue = feature?.geometry?.value;
  
  if (!geometryValue) return null;
  
  if (geometryValue.FlowGeometry2D) return 'FlowGeometry2D';
  if (geometryValue.FlowGeometry3D) return 'FlowGeometry3D'; 
  if (geometryValue.CityGmlGeometry) return 'CityGmlGeometry';
  
  return 'Unknown';
}

function analyzeDataType(features: any[]): GeometryType {
  if (features.length === 0) return null;
  
  // Check first few features to determine predominant type
  const sampleSize = Math.min(10, features.length);
  const typeCounts: Record<string, number> = {};
  
  for (let i = 0; i < sampleSize; i++) {
    const type = detectGeometryType(features[i]);
    if (type) {
      typeCounts[type] = (typeCounts[type] || 0) + 1;
    }
  }
  
  // Return most common type
  const entries = Object.entries(typeCounts);
  if (entries.length === 0) return null;
  
  return entries.reduce((a, b) => 
    typeCounts[a[0]] > typeCounts[b[0]] ? a : b
  )[0] as GeometryType;
}

export const useStreamingDebugRunQuery = (
  dataUrl: string,
  options: UseStreamingDebugRunQueryOptions = {}
): {
  fileContent: any;
  fileType: SupportedDataTypes;
  isLoading: boolean;
  [key: string]: any;
} => {
  const {
    enabled = true,
    batchSize = 1000,
    chunkSize = 64 * 1024,
    onProgress,
    onError,
  } = options;

  const queryClient = useQueryClient();
  const queryKey = useMemo(() => ["streamingDataUrl", dataUrl], [dataUrl]);
  const abortControllerRef = useRef<AbortController>(null);
  const [detectedFileType, setDetectedFileType] = useState<GeometryType>(null);
  
  const [streamingState, setStreamingState] = useState<StreamingState<any>>({
    data: [],
    isStreaming: false,
    isComplete: false,
    progress: {
      bytesProcessed: 0,
      featuresProcessed: 0,
    },
    error: null,
    hasMore: false,
  });

  // Create a separate query for metadata/initial check
  const metadataQuery = useQuery({
    queryKey: [...queryKey, "metadata"],
    queryFn: async () => {
      if (!dataUrl) return null;
      
      // Just fetch headers to check content length and type
      const response = await fetch(dataUrl, { method: "HEAD" });
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }
      
      return {
        contentLength: response.headers.get("content-length"),
        contentType: response.headers.get("content-type"),
      };
    },
    enabled: enabled && !!dataUrl,
    staleTime: Infinity,
    gcTime: Infinity,
  });

  const startStreaming = useCallback(async () => {
    if (!dataUrl || streamingState.isStreaming) return;

    // Create new abort controller
    abortControllerRef.current = new AbortController();

    setStreamingState(prev => ({
      ...prev,
      isStreaming: true,
      error: null,
    }));

    try {
      const streamGenerator = await streamJsonl(dataUrl, {
        batchSize,
        chunkSize,
        signal: abortControllerRef.current.signal,
        onProgress: (progress) => {
          setStreamingState(prev => ({
            ...prev,
            progress,
          }));
          onProgress?.(progress);
        },
        onError: (error) => {
          setStreamingState(prev => ({
            ...prev,
            error,
            isStreaming: false,
          }));
          onError?.(error);
        },
      });

      // Process stream
      for await (const result of streamGenerator) {
        // Detect file type from first batch if not already detected
        if (!detectedFileType && result.data.length > 0) {
          const type = analyzeDataType(result.data);
          setDetectedFileType(type);
        }

        // Update local state
        setStreamingState(prev => ({
          ...prev,
          data: [...prev.data, ...result.data],
          progress: result.progress,
          isComplete: result.isComplete,
          hasMore: result.hasMore,
          isStreaming: !result.isComplete,
        }));

        // Also update the query cache so other components can access the data
        queryClient.setQueryData(queryKey, (prevData: any) => {
          const currentData = prevData?.data || [];
          const newData = [...currentData, ...result.data];
          return {
            data: newData,
            progress: result.progress,
            isComplete: result.isComplete,
            hasMore: result.hasMore,
            fileContent: newData, // For compatibility with existing code
            type: detectedFileType || analyzeDataType(newData) || "jsonl",
          };
        });

        // If streaming is complete, break the loop
        if (result.isComplete) {
          break;
        }
      }

    } catch (error) {
      if (error instanceof Error && error.name === 'AbortError') {
        // Stream was cancelled, don't treat as error
        setStreamingState(prev => ({
          ...prev,
          isStreaming: false,
        }));
      } else {
        const err = error as Error;
        setStreamingState(prev => ({
          ...prev,
          error: err,
          isStreaming: false,
        }));
        onError?.(err);
      }
    }
  }, [dataUrl, batchSize, chunkSize, onProgress, onError, queryClient, queryKey, streamingState.isStreaming, detectedFileType]);

  const loadMore = useCallback(() => {
    if (streamingState.hasMore && !streamingState.isStreaming) {
      startStreaming();
    }
  }, [streamingState.hasMore, streamingState.isStreaming, startStreaming]);

  const stopStreaming = useCallback(() => {
    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
    }
  }, []);

  const resetStreaming = useCallback(() => {
    stopStreaming();
    setStreamingState({
      data: [],
      isStreaming: false,
      isComplete: false,
      progress: {
        bytesProcessed: 0,
        featuresProcessed: 0,
      },
      error: null,
      hasMore: false,
    });
    setDetectedFileType(null);
    queryClient.removeQueries({ queryKey });
  }, [stopStreaming, queryClient, queryKey]);

  // Auto-start streaming when enabled and URL changes
  useEffect(() => {
    if (enabled && dataUrl && !streamingState.isStreaming && streamingState.data.length === 0) {
      startStreaming();
    }
  }, [enabled, dataUrl, startStreaming, streamingState.isStreaming, streamingState.data.length]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (abortControllerRef.current) {
        abortControllerRef.current.abort();
      }
    };
  }, []);

  // Also provide the cached data from React Query for compatibility
  const cachedQuery = useQuery({
    queryKey,
    queryFn: () => {
      // This function shouldn't actually run since we're manually setting the data
      return Promise.resolve(null);
    },
    enabled: false,
  });

  return {
    // Streaming-specific data
    ...streamingState,
    loadMore,
    stopStreaming,
    resetStreaming,
    
    // Metadata
    metadata: metadataQuery.data,
    detectedGeometryType: detectedFileType,
    
    // Compatibility with existing useFetchAndReadData interface
    fileContent: streamingState.data,
    fileType: "geojson", // All streaming data normalized to geojson format
    isLoading: streamingState.isStreaming || metadataQuery.isLoading,
    
    // Additional query states
    isFetching: streamingState.isStreaming,
    
    // React Query compatibility
    data: cachedQuery.data,
    isError: !!streamingState.error || metadataQuery.isError,
    error: streamingState.error || metadataQuery.error,
  };
};