import React, { useState } from "react";
import { useDocumentApi } from "@flow/lib/fetch/transformers/useDocumentApi";

export const DocumentApiTest: React.FC = () => {
  const [testDocId, setTestDocId] = useState("01jwxzsr5f80n8exm21n43ghay");
  const [manualTestResults, setManualTestResults] = useState<string>("");
  
  const {
    useGetLatestDocument,
    useGetDocumentHistory,
    useCreateSnapshot,
  } = useDocumentApi();

  // Use the hooks
  const { data: documentData, error: documentError, isLoading: documentLoading } = useGetLatestDocument(testDocId);
  const { data: historyData, error: historyError, isLoading: historyLoading } = useGetDocumentHistory(testDocId);
  const createSnapshotMutation = useCreateSnapshot();

  // Manual test function
  const testManualRequest = async () => {
    try {
      const response = await fetch(`https://api.flow.test.reearth.dev/api/document/${testDocId}`, {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
        },
      });
      
      const text = await response.text();
      setManualTestResults(`
Status: ${response.status}
Headers: ${JSON.stringify(Object.fromEntries(response.headers.entries()), null, 2)}
Body: ${text}
      `);
    } catch (error) {
      setManualTestResults(`Error: ${error}`);
    }
  };

  const testCreateSnapshot = async () => {
    try {
      const result = await createSnapshotMutation.mutateAsync({
        docId: testDocId,
        version: 1,
        name: "test-snapshot"
      });
      console.log("Snapshot created:", result);
    } catch (error) {
      console.error("Snapshot creation failed:", error);
    }
  };

  return (
    <div style={{ padding: '20px', fontFamily: 'monospace' }}>
      <h2>Document API Test</h2>
      
      <div style={{ marginBottom: '20px' }}>
        <label>
          Document ID: 
          <input 
            type="text" 
            value={testDocId} 
            onChange={(e) => setTestDocId(e.target.value)}
            style={{ marginLeft: '10px', width: '300px' }}
          />
        </label>
      </div>

      {/* React Query Tests */}
      <div style={{ marginBottom: '20px', border: '1px solid #ccc', padding: '10px' }}>
        <h3>React Query Tests</h3>
        
        <div style={{ marginBottom: '10px' }}>
          <h4>Get Latest Document:</h4>
          {documentLoading && <p>Loading...</p>}
          {documentError && <p style={{ color: 'red' }}>Error: {documentError.message}</p>}
          {documentData && <pre>{JSON.stringify(documentData, null, 2)}</pre>}
        </div>

        <div style={{ marginBottom: '10px' }}>
          <h4>Get Document History:</h4>
          {historyLoading && <p>Loading...</p>}
          {historyError && <p style={{ color: 'red' }}>Error: {historyError.message}</p>}
          {historyData && <pre>{JSON.stringify(historyData, null, 2)}</pre>}
        </div>

        <div>
          <button onClick={testCreateSnapshot} disabled={createSnapshotMutation.isPending}>
            {createSnapshotMutation.isPending ? 'Creating...' : 'Test Create Snapshot'}
          </button>
          {createSnapshotMutation.error && (
            <p style={{ color: 'red' }}>Snapshot Error: {createSnapshotMutation.error.message}</p>
          )}
          {createSnapshotMutation.data && (
            <pre>{JSON.stringify(createSnapshotMutation.data, null, 2)}</pre>
          )}
        </div>
      </div>

      {/* Manual Test */}
      <div style={{ border: '1px solid #ccc', padding: '10px' }}>
        <h3>Manual Request Test</h3>
        <button onClick={testManualRequest}>Test Manual Request</button>
        {manualTestResults && (
          <pre style={{ backgroundColor: '#f5f5f5', padding: '10px', marginTop: '10px' }}>
            {manualTestResults}
          </pre>
        )}
      </div>
    </div>
  );
}; 