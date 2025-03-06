export type GeneralState = {
  clipboard: any | undefined;
};

type Job = {
  projectId: string;
  jobId: string;
};

export type DebugRunState = {
  jobs: Job[];
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

const STORE_NAME = "appState";

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

type InitialStateKeys = keyof typeof initialState;

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
      resolve(db);
    };

    request.onerror = () => reject(request.error);
  });
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

export async function loadStateFromIndexedDB<T extends InitialStateKeys>(
  key: T,
): Promise<AppState[T] | null> {
  const db = await openDatabase();
  const transaction = db.transaction(STORE_NAME, "readonly");
  const store = transaction.objectStore(STORE_NAME);

  return new Promise((resolve, reject) => {
    const request = store.get(key);
    request.onsuccess = () => resolve(request.result || null);
    request.onerror = () => reject(request.error);
  });
}

export async function updateClipboardState(newCopyingState: any) {
  await saveStateToIndexedDB({ clipboard: newCopyingState }, GENERAL_KEY);
}

export async function updateJobs({
  projectId,
  jobId,
}: {
  projectId: string;
  jobId?: string;
}) {
  const existingState = await loadStateFromIndexedDB(DEBUG_RUN_KEY);
  let jobs: Job[] = existingState?.jobs || [];

  if (!jobId) {
    jobs =
      existingState?.jobs.filter((job) => job.projectId !== projectId) || [];
  } else if (
    jobId &&
    existingState?.jobs.some((job) => job.projectId === projectId)
  ) {
    jobs = existingState.jobs.map((job) => {
      if (job.projectId === projectId) {
        return { projectId, jobId };
      }
      return job;
    });
  }

  await saveStateToIndexedDB({ jobs }, DEBUG_RUN_KEY);
}

// async function updatePreferences(newPreferences: Partial<Preferences>) {
//   const existingState = (await loadStateFromIndexedDB()) || {};
//   const updatedPreferences = { ...existingState.preferences, ...newPreferences };
//   await saveStateToIndexedDB({ preferences: updatedPreferences });
// }
