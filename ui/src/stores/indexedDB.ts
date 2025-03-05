import { Edge } from "@flow/types";

type CopyingState = {
  nodeIds: string[];
  edges: Edge[];
};

// interface Preferences {
//   theme: string;
//   autoSave: boolean;
//   gridSize: number;
// }

type AppState = {
  copying: CopyingState;
  jobId: string;
  // preferences?: Preferences;
};

const DB_NAME = window.REEARTH_CONFIG?.brandName || "ReEarthFlowDB";
const DB_VERSION = 1;
const STORE_NAME = "appState";

function openDatabase(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open(DB_NAME, DB_VERSION);

    request.onupgradeneeded = (event) => {
      const db = (event.target as IDBOpenDBRequest).result;
      if (!db.objectStoreNames.contains(STORE_NAME)) {
        db.createObjectStore(STORE_NAME);
      }
    };

    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error);
  });
}

export async function saveStateToIndexedDB(partialData: Partial<AppState>) {
  const db = await openDatabase();
  const transaction = db.transaction(STORE_NAME, "readwrite");
  const store = transaction.objectStore(STORE_NAME);

  const existingData = await loadStateFromIndexedDB();
  const newData = { ...existingData, ...partialData };

  store.put(newData, "state");

  return new Promise<void>((resolve, reject) => {
    transaction.oncomplete = () => resolve();
    transaction.onerror = () => reject(transaction.error);
  });
}

export async function loadStateFromIndexedDB(): Promise<AppState | null> {
  const db = await openDatabase();
  const transaction = db.transaction(STORE_NAME, "readonly");
  const store = transaction.objectStore(STORE_NAME);

  return new Promise((resolve, reject) => {
    const request = store.get("state");
    request.onsuccess = () => resolve(request.result || null);
    request.onerror = () => reject(request.error);
  });
}

export async function updateCopyingState(newCopyingState: CopyingState) {
  await saveStateToIndexedDB({ copying: newCopyingState });
}

export async function updateJobId(newJobId: string) {
  await saveStateToIndexedDB({ jobId: newJobId });
}

// async function updatePreferences(newPreferences: Partial<Preferences>) {
//   const existingState = (await loadStateFromIndexedDB()) || {};
//   const updatedPreferences = { ...existingState.preferences, ...newPreferences };
//   await saveStateToIndexedDB({ preferences: updatedPreferences });
// }
