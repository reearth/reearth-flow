import { useEffect, useState } from "react";

import {
  AppState,
  InitialStateKeys,
  openDatabase,
  STORE_NAME,
} from "@flow/stores";
import { generateUUID } from "@flow/utils";

// Simple type for subscribers
type Subscriber<T> = (value: T) => void;
type Subscription = {
  unsubscribe: () => void;
};

// Global subscribers object
const subscribers: Record<string, Subscriber<any>[]> = {};

// Simple update queues for each key
const updateQueues: Record<string, Promise<any>> = {};

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

// Get from DB with standard pattern
async function getFromDB<T extends InitialStateKeys>(
  key: T,
): Promise<AppState[T]> {
  const db = await openDatabase();
  const transaction = db.transaction(STORE_NAME, "readonly");
  const store = transaction.objectStore(STORE_NAME);
  return new Promise((resolve, reject) => {
    const request = store.get(key);
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error);
  });
}

// Update in DB with standard pattern
async function updateInDB<T>(key: string | number, value: T): Promise<void> {
  const db = await openDatabase();
  const transaction = db.transaction(STORE_NAME, "readwrite");
  const store = transaction.objectStore(STORE_NAME);

  return new Promise<void>((resolve, reject) => {
    const request = store.put(value, key);

    request.onsuccess = () => {
      publishChange(key, value);
      resolve();
    };

    request.onerror = () => reject(request.error);
  });
}

// The main hook
export function useIndexedDB<T extends InitialStateKeys>(
  key: T,
): {
  value: Partial<AppState[T]> | null;
  loading: boolean;
  error: Error | null;
  updateValue: (
    newValueOrUpdater:
      | Partial<AppState[T]>
      | ((prevState: AppState[T]) => AppState[T]),
  ) => Promise<void>;
} {
  const [value, setValue] = useState<Partial<AppState[T]> | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<Error | null>(null);

  // Initial data fetch
  useEffect(() => {
    let isMounted = true;

    const fetchData = async (): Promise<void> => {
      try {
        const data = await getFromDB(key);
        if (isMounted) {
          setValue(data);
          setLoading(false);
        }
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

  // Function to update value with queuing
  const updateValue = async (
    newValueOrUpdater:
      | Partial<AppState[T]>
      | ((prevState: AppState[T]) => AppState[T]),
  ): Promise<void> => {
    // Get queue ID
    const queueId = generateUUID();

    // Initialize the queue if it doesn't exist
    if (!updateQueues[queueId]) {
      updateQueues[queueId] = Promise.resolve();
    }

    // Create a new promise that will execute after the current queue
    const updatePromise = updateQueues[queueId].then(async () => {
      try {
        // Get the current value from DB
        const currentValue = await getFromDB(key);

        // Determine the final value to store
        let finalValue: AppState[T];

        // Use type predicate to help TypeScript distinguish function from object
        const isUpdaterFunction = (
          val: Partial<AppState[T]> | ((prevState: AppState[T]) => AppState[T]),
        ): val is (prevState: AppState[T]) => AppState[T] => {
          return typeof val === "function";
        };

        // Check if the updater is a function using the type predicate
        if (isUpdaterFunction(newValueOrUpdater)) {
          // Call the updater function with the current value
          finalValue = newValueOrUpdater(currentValue);
        } else {
          // If it's a direct value, merge with current value
          finalValue = {
            ...currentValue,
            ...newValueOrUpdater,
          } as AppState[T];
        }

        // Update the database with the final value
        await updateInDB(key, finalValue);
        // Don't return the value, to match Promise<void> return type
      } catch (err) {
        setError(err instanceof Error ? err : new Error(String(err)));
        throw err;
      }
    });

    // Update the queue reference to include this operation
    updateQueues[queueId] = updatePromise.catch(() => {
      // Catch errors to allow the queue to continue
      return;
    });

    // Return the promise for this specific update
    return updatePromise;
  };

  return { value, loading, error, updateValue };
}
