import { api } from '$lib/infrastructure/http/client';

const base = '/admin/files';

export interface MediaEntry {
    path: string;
    name: string;
    directory: boolean;
}

export type Job<T> =
    | { state: 'idle' | 'running' }
    | { state: 'done'; report: T }
    | { state: 'failed'; error: string };

export interface ImportReport {
    imported: number;
    skipped: number;
    failed: number;
    errors: string[];
}

export interface UnavailableFile {
    id: string;
    path: string;
    reason: string;
}

export interface ScanReport {
    checked: number;
    unavailable: UnavailableFile[];
}

export const adminFiles = {
    browse(path: string, directoriesOnly: boolean) {
        const query = new URLSearchParams({ directories_only: String(directoriesOnly) });
        if (path) query.set('path', path);
        return api.get<MediaEntry[]>(`${base}/browse?${query}`);
    },
    startImport: (path: string) => api.post<void>(`${base}/import`, { path }),
    importStatus: () => api.get<Job<ImportReport>>(`${base}/import`),
    startScan: () => api.post<void>(`${base}/scan`),
    scanStatus: () => api.get<Job<ScanReport>>(`${base}/scan`),
    repair: (file: UnavailableFile, path: string) =>
        api.patch<void>(`${base}/${file.id}/path`, { path, previous_path: file.path })
};
