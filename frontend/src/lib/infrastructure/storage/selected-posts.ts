import type { UUID } from "$lib/domain";

const DB_NAME = "gl-feed-selection";
const STORE_NAME = "state";
const KEY = "selected-post-ids";

function openDb(): Promise<IDBDatabase> {
    return new Promise((resolve, reject) => {
        if (typeof indexedDB === "undefined") {
            reject(new Error("IndexedDB is not available"));
            return;
        }

        const request = indexedDB.open(DB_NAME, 1);
        request.onupgradeneeded = () => {
            const db = request.result;
            if (!db.objectStoreNames.contains(STORE_NAME)) {
                db.createObjectStore(STORE_NAME);
            }
        };
        request.onsuccess = () => resolve(request.result);
        request.onerror = () => reject(request.error ?? new Error("Failed to open IndexedDB"));
    });
}

async function withStore<T>(mode: IDBTransactionMode, run: (store: IDBObjectStore) => Promise<T>): Promise<T> {
    const db = await openDb();
    try {
        const tx = db.transaction(STORE_NAME, mode);
        const store = tx.objectStore(STORE_NAME);
        const result = await run(store);
        await new Promise<void>((resolve, reject) => {
            tx.oncomplete = () => resolve();
            tx.onerror = () => reject(tx.error ?? new Error("IndexedDB transaction failed"));
            tx.onabort = () => reject(tx.error ?? new Error("IndexedDB transaction aborted"));
        });
        return result;
    } finally {
        db.close();
    }
}

export async function readSelectedPostIds(): Promise<UUID[]> {
    if (typeof window === "undefined") return [];

    try {
        return await withStore("readonly", async (store) => {
            const request = store.get(KEY);
            const value = await new Promise<unknown>((resolve, reject) => {
                request.onsuccess = () => resolve(request.result);
                request.onerror = () => reject(request.error ?? new Error("Failed to read selected ids"));
            });

            if (!Array.isArray(value)) return [];
            return value.filter((item): item is UUID => typeof item === "string");
        });
    } catch {
        return [];
    }
}

export async function writeSelectedPostIds(ids: UUID[]): Promise<void> {
    if (typeof window === "undefined") return;

    try {
        await withStore("readwrite", async (store) => {
            const request = store.put(ids, KEY);
            await new Promise<void>((resolve, reject) => {
                request.onsuccess = () => resolve();
                request.onerror = () => reject(request.error ?? new Error("Failed to write selected ids"));
            });
        });
    } catch {
        // no-op
    }
}
