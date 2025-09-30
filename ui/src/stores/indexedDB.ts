import { CLIPBOARD_EXPIRATION_TIME } from "@flow/global-constants";
import { JobStatus, NodeExecution } from "@flow/types";

export type GeneralState = {
  clipboard: any | undefined;
};

export type SelectedIntermediateData = {
  edgeId: string;
  url: string;
  displayName?: string;
  sourceName?: string;
  targetName?: string;
};

export type JobState = {
  projectId: string;
  jobId: string;
  status: JobStatus;
  nodeExecutions?: NodeExecution[];
  selectedIntermediateData?: SelectedIntermediateData[]; // undefined = never touched, [] = user has selected/deselected
};

export type DebugRunState = {
  jobs: JobState[];
};

export type PreferencesState = {
  theme: string;
};

export type AppState = {
  [GENERAL_KEY]: GeneralState;
  [DEBUG_RUN_KEY]: DebugRunState;
  [PREFERENCES_KEY]: PreferencesState;
};

const DB_NAME = window.REEARTH_CONFIG?.brandName || "ReEarthFlowDB";
const DB_VERSION = 1;

export const STORE_NAME = "appState";

const GENERAL_KEY = "general";
const DEBUG_RUN_KEY = "debugRun";
const PREFERENCES_KEY = "preferences";
const KEYS = [GENERAL_KEY, DEBUG_RUN_KEY, PREFERENCES_KEY];

const initialState = {
  [GENERAL_KEY]: {
    clipboard: undefined,
  },
  [DEBUG_RUN_KEY]: {
    jobs: [],
  },
  [PREFERENCES_KEY]: {
    theme: "dark",
  },
};

export type InitialStateKeys = keyof typeof initialState;

export async function openDatabase(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open(DB_NAME, DB_VERSION);

    request.onupgradeneeded = (event) => {
      const db = (event.target as IDBOpenDBRequest).result;
      if (!db.objectStoreNames.contains(STORE_NAME)) {
        db.createObjectStore(STORE_NAME);
      }
    };

    request.onsuccess = async (event) => {
      const db = (event.target as IDBOpenDBRequest).result;
      await ensureInitialState(db);
      const general = await loadStateFromIndexedDB("general", db);
      const shouldClearClipboard = await isClipboardTimeoutExpired(general);

      if (shouldClearClipboard && general?.clipboard) {
        saveStateToIndexedDB({ clipboard: undefined }, "general");
      }

      resolve(db);
    };

    request.onerror = () => reject(request.error);
  });
}

async function isClipboardTimeoutExpired(
  state: GeneralState | null,
): Promise<boolean> {
  if (!state) {
    return false;
  }
  if (
    state?.clipboard?.copiedAt &&
    Date.now() - state.clipboard.copiedAt > CLIPBOARD_EXPIRATION_TIME
  ) {
    return true;
  }
  return false;
}

async function ensureInitialState(db: IDBDatabase) {
  return Promise.all(
    KEYS.map(
      (key) =>
        new Promise<void>((resolve, reject) => {
          const transaction = db.transaction(STORE_NAME, "readwrite");
          const store = transaction.objectStore(STORE_NAME);
          const request = store.get(key);

          request.onsuccess = () => {
            if (!request.result) {
              store.put(initialState[key as InitialStateKeys], key);
            }
            resolve();
          };

          request.onerror = () => reject(request.error);
        }),
    ),
  );
}

async function saveStateToIndexedDB<T extends InitialStateKeys>(
  partialData: Partial<AppState[T]>,
  key: T,
) {
  const db = await openDatabase();
  const transaction = db.transaction(STORE_NAME, "readwrite");
  const store = transaction.objectStore(STORE_NAME);

  return new Promise<void>((resolve, reject) => {
    const request = store.get(key);

    request.onsuccess = () => {
      const existingData = request.result || {};
      const newData = { ...existingData, ...partialData };

      store.put(newData, key);

      transaction.oncomplete = () => resolve();
      transaction.onerror = () => reject(transaction.error);
    };

    request.onerror = () => reject(request.error);
  });
}

async function loadStateFromIndexedDB<T extends InitialStateKeys>(
  key: T,
  db: IDBDatabase,
): Promise<AppState[T] | null> {
  const transaction = db.transaction(STORE_NAME, "readonly");
  const store = transaction.objectStore(STORE_NAME);

  return new Promise((resolve, reject) => {
    const request = store.get(key);
    request.onsuccess = () => resolve(request.result || null);
    request.onerror = () => reject(request.error);
  });
}
