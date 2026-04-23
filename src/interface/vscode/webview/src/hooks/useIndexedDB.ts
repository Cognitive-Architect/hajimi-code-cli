import { useState, useEffect, useCallback } from 'react';
import type { ChatMessage } from '../types/webview';

const DB_NAME = 'hajimi-chat';
const DB_VERSION = 1;
const STORE_NAME = 'messages';

function openDB(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const req = indexedDB.open(DB_NAME, DB_VERSION);
    req.onerror = () => reject(req.error);
    req.onsuccess = () => resolve(req.result);
    req.onupgradeneeded = (event) => {
      const db = (event.target as IDBOpenDBRequest).result;
      if (!db.objectStoreNames.contains(STORE_NAME)) {
        db.createObjectStore(STORE_NAME, { keyPath: 'id' });
      }
    };
  });
}

export function useIndexedDB() {
  const [db, setDb] = useState<IDBDatabase | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    openDB()
      .then(setDb)
      .catch((err) => setError(err instanceof Error ? err.message : String(err)));
  }, []);

  const saveMessages = useCallback(
    async (messages: ChatMessage[]): Promise<void> => {
      if (!db) return;
      const tx = db.transaction(STORE_NAME, 'readwrite');
      const store = tx.objectStore(STORE_NAME);
      await Promise.all(
        messages.map(
          (msg) =>
            new Promise<void>((resolve, reject) => {
              const req = store.put(msg);
              req.onsuccess = () => resolve();
              req.onerror = () => reject(req.error);
            })
        )
      );
    },
    [db]
  );

  const loadMessages = useCallback(async (): Promise<ChatMessage[]> => {
    if (!db) return [];
    if (error) return []; // fallback on DB error
    return new Promise((resolve, reject) => {
      const tx = db.transaction(STORE_NAME, 'readonly');
      const store = tx.objectStore(STORE_NAME);
      const req = store.getAll();
      req.onsuccess = () => {
        const result = (req.result as ChatMessage[]).sort((a, b) => a.timestamp - b.timestamp);
        resolve(result);
      };
      req.onerror = () => reject(req.error);
    });
  }, [db, error]);

  const clearMessages = useCallback(async (): Promise<void> => {
    if (!db) return;
    return new Promise((resolve, reject) => {
      const tx = db.transaction(STORE_NAME, 'readwrite');
      const store = tx.objectStore(STORE_NAME);
      const req = store.clear();
      req.onsuccess = () => resolve();
      req.onerror = () => reject(req.error);
    });
  }, [db]);

  return { saveMessages, loadMessages, clearMessages, error };
}
