import { useEffect, useState } from "react";

import {
  AppState,
  InitialStateKeys,
  openDatabase,
  STORE_NAME,
} from "@flow/stores";

// Simple type for subscribers
type Subscriber<T> = (value: T) => void;
type Subscription = {
  unsubscribe: () => void;
};

// Global subscribers object
const subscribers: Record<string, Subscriber<any>[]> = {};

// Publish changes to subscribers
export function publishChange<T>(key: string | number, value: T): void {
  const id = `${STORE_NAME}-${key}`;
  if (subscribers[id]) {
    subscribers[id].forEach((callback) => callback(value));
  }
}

// Subscribe to changes
export function subscribeToChanges<T>(
  key: string | number,
  callback: Subscriber<T>,
): Subscription {
  const id = `${STORE_NAME}-${key}`;
  if (!subscribers[id]) {
    subscribers[id] = [];
  }
  subscribers[id].push(callback);

  return {
    unsubscribe: () => {
      subscribers[id] = subscribers[id].filter((cb) => cb !== callback);
    },
  };
}

// Update function for IndexedDB
async function updateInDB<T>(key: string | number, value: T): Promise<void> {
  const db = await openDatabase();
  const transaction = db.transaction(STORE_NAME, "readwrite");
  const store = transaction.objectStore(STORE_NAME);
  store.put(value, key);
  publishChange(key, value);
}

// The main hook
export function useIndexedDB<T extends InitialStateKeys>(
  key: T,
): {
  value: Partial<AppState[T]> | null;
  loading: boolean;
  error: Error | null;
  updateValue: (newValue: Partial<AppState[T]>) => Promise<void>;
} {
  const [value, setValue] = useState<Partial<AppState[T]> | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<Error | null>(null);

  // Initial data fetch
  useEffect(() => {
    let isMounted = true;

    const fetchData = async (): Promise<void> => {
      try {
        const db = await openDatabase();
        const transaction = db.transaction(STORE_NAME, "readonly");
        const store = transaction.objectStore(STORE_NAME);
        const request = store.get(key);

        request.onsuccess = () => {
          if (isMounted) {
            setValue(request.result);
            setLoading(false);
          }
        };

        request.onerror = () => {
          if (isMounted) {
            setError(request.error);
            setLoading(false);
          }
        };
      } catch (err) {
        if (isMounted) {
          setError(err instanceof Error ? err : new Error(String(err)));
          setLoading(false);
        }
      }
    };

    fetchData();

    // Set up listener for database changes
    const subscription = subscribeToChanges<Partial<AppState[T]>>(
      key,
      (newValue) => {
        if (isMounted) {
          setValue(newValue);
        }
      },
    );

    return () => {
      isMounted = false;
      subscription.unsubscribe();
    };
  }, [key]);

  // Function to update value (exposed in the hook result)
  const updateValue = async (newValue: Partial<AppState[T]>): Promise<void> => {
    try {
      await updateInDB(key, newValue);
    } catch (err) {
      setError(err instanceof Error ? err : new Error(String(err)));
      throw err;
    }
  };

  return { value, loading, error, updateValue };
}
