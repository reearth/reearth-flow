# Streaming Debug Data Implementation

This document describes the streaming JSONL implementation for Re:Earth Flow's debug panel, which handles large intermediate data files without memory issues.

## Overview

The streaming system allows users to preview large JSONL intermediate data files (1MB+) by streaming and displaying the first 2000 features while counting total features in the background. It supports multiple file caching and graceful switching between files.

## Architecture

### Core Components

1. **`useStreamingDebugRunQuery`** - Main hook for streaming JSONL data
2. **`useStreamingDataColumnizer`** - Dynamic table column discovery and data transformation
3. **`streamJsonl` utility** - Low-level JSONL streaming parser
4. **`TableViewer` component** - UI component with streaming support
5. **`VirtualizedTable`** - Table with column width constraints

### Data Flow

```
JSONL File → streamJsonl → useStreamingDebugRunQuery → useStreamingDataColumnizer → VirtualizedTable
```

## Key Features

### 🚀 **Streaming Performance**
- **Display Limit**: Shows first 2000 features for UI responsiveness
- **Background Counting**: Continues streaming to count total features
- **Chunk Processing**: Processes 64KB chunks with 1000-feature batches
- **Memory Management**: Smart LRU cache with max 8 files

### 📁 **Multi-File Support**
- **React Query Caching**: Per-file caching with 30min stale time
- **Graceful Switching**: Aborts ongoing streams when switching files
- **File Identity Detection**: Compares first feature content to detect file changes
- **Mixed Data Types**: Handles intermediate data with various formats (GML, XML, MD, PDF)

### 🔍 **Geometry Type Detection**
- **FlowGeometry2D/3D**: Detects Flow-specific geometry types
- **CityGmlGeometry**: Supports CityGML geometry with case variations
- **Mixed File Handling**: Returns `null` for files with <50% recognizable geometry

### 📊 **Dynamic Table Features**
- **Column Discovery**: Auto-discovers columns from streaming data
- **Width Constraints**: Enforces min(100px), default(200px), max(400px)
- **Virtualization**: Handles large datasets efficiently
- **Feature Details**: Double-click to view full feature properties

## Implementation Details

### File Size Decision Logic

```typescript
// In DebugPanel hooks.ts
const shouldUseTraditionalLoading = useMemo(() => {
  const isIntermediateData = intermediateDataURLs?.includes(metadataUrl);
  const isOutputData = outputURLs?.includes(metadataUrl);
  
  // Only use streaming for JSONL intermediate data
  if (!isIntermediateData || isOutputData) return true;
  
  // Default to streaming for unknown size JSONL
  if (!contentLength) return false;
  
  const sizeInMB = parseInt(contentLength) / (1024 * 1024);
  return sizeInMB < 10; // Use traditional for <10MB
}, [fileMetadata, metadataUrl, intermediateDataURLs, outputURLs]);
```

### Streaming State Management

```typescript
type StreamingState = {
  data: any[];              // First 2000 features for display
  isStreaming: boolean;     // Currently streaming
  isComplete: boolean;      // Stream finished
  progress: {
    bytesProcessed: number;
    featuresProcessed: number;
  };
  error: Error | null;
  totalFeatures: number;    // Full count from background streaming
};
```

### Cache Structure

```typescript
// React Query cache format
{
  data: any[];                    // Display data (2000 features)
  fileContent: any[];             // Same as data
  progress: StreamingProgress;    // Final progress
  isComplete: boolean;            // Completion status
  totalFeatures: number;          // Full feature count
  detectedGeometryType: GeometryType;
  cachedAt: number;              // Timestamp for LRU
}
```

## Configuration

### Default Settings
```typescript
const defaultOptions = {
  batchSize: 1000,           // Features per batch
  chunkSize: 64 * 1024,      // 64KB per chunk
  displayLimit: 2000,        // Max features shown
  maxCachedFiles: 8,         // LRU cache limit
};
```

### File Size Thresholds
- **< 10MB**: Traditional loading (full fetch)
- **≥ 10MB**: Streaming with 2000 feature limit
- **Unknown size**: Defaults to streaming

## Error Handling

### Graceful Degradation
- **AbortError**: Silently handled during file switching
- **Network Error**: Shows error state with retry option
- **Parse Error**: Falls back to traditional loading
- **Memory Issues**: LRU cache eviction prevents OOM

### Stream Interruption
- **File Switch**: Immediately aborts current stream
- **Tab Switch**: Preserves stream state
- **Component Unmount**: Cleans up abort controllers

## Performance Considerations

### Memory Usage
- **Display Data**: ~2000 features × avg feature size
- **Cache Storage**: 8 files × 2000 features each
- **Streaming Buffer**: ~64KB text buffer
- **Total Estimate**: ~50-100MB for typical use

### Network Efficiency
- **Chunked Reading**: Progressive loading
- **Background Processing**: Non-blocking UI
- **Cache Reuse**: Avoids re-downloading switched files

## Known Limitations

### Current Constraints
1. **Display Limit**: Only first 2000 features shown (not configurable)
2. **No Resume**: Can't resume interrupted streams
3. **No Search/Filter**: Search only works on displayed 2000 features
4. **Column Discovery**: Based on first 2000 features only
5. **Geometry Detection**: Limited to Flow/CityGML types

### Future Improvements
- **Server-side Search**: Full-file search capabilities
- **Configurable Limits**: User-adjustable display limits
- **Resume Support**: Continue interrupted streams
- **Progressive Column Discovery**: Update columns as streaming continues

## Debugging

### Console Monitoring
```javascript
// Check streaming state
console.log('Streaming state:', streamingQuery);
// {data: Array(2000), totalFeatures: 50000, isStreaming: false, ...}

// Monitor cache
queryClient.getQueryCache().getAll()
  .filter(q => q.queryKey[0] === 'streamingDataUrl');
```

### Common Issues

**Data Mixing Between Files**
- **Cause**: File change detection failed
- **Fix**: Check `prevFileContentRef` comparison logic

**Column Width Not Enforced**
- **Cause**: TanStack Table not applying maxSize
- **Fix**: Verify `Math.min(getSize(), maxSize)` in table styles

**Memory Leaks**
- **Cause**: Abort controllers not cleaned up
- **Fix**: Ensure `abortControllerRef.current = null` after abort

**Cache Corruption**
- **Cause**: Partial streams cached during abort
- **Fix**: Only cache complete streams (`isComplete: true`)

## File Structure

```
src/
├── hooks/
│   ├── useStreamingDebugRunQuery.ts     # Main streaming hook
│   └── useStreamingDataColumnizer.ts    # Table data transformation
├── utils/streaming/
│   ├── streamJsonl.ts                   # JSONL parser
│   └── types.ts                         # Streaming type definitions
├── features/Editor/components/OverlayUI/components/DebugPanel/
│   ├── hooks.ts                         # File size decision logic
│   └── DebugPreview/components/TableViewer/
│       └── index.tsx                    # Streaming table UI
└── components/visualizations/VirtualizedTable/
    └── index.tsx                        # Column width constraints
```

## Testing Scenarios

### Manual Testing
1. **Large File (>10MB)**: Verify streaming behavior
2. **Small File (<10MB)**: Verify traditional loading
3. **File Switching**: Test during active stream
4. **Mixed Data Types**: Test geometry detection
5. **Memory Usage**: Monitor with dev tools over time
6. **Network Issues**: Test with throttled connection

### Edge Cases
- Empty files
- Single feature files
- Malformed JSONL
- Network interruption
- Very large features (>1MB each)
- Files with no geometry data

---

*Last updated: 2025-01-04 - Implementation complete with graceful streaming, multi-file caching, and column width constraints*