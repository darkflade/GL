<script lang="ts">
    import { onMount } from 'svelte';
    import Header from '$lib/shared/components/layout/Header.svelte';
    import MediaPicker from '$lib/features/admin/MediaPicker.svelte';
    import { adminFiles, type ImportReport, type Job, type ScanReport, type UnavailableFile } from '$lib/features/admin/files';

    let directory = $state('');
    let picker = $state<'directory' | 'file' | null>(null);
    let repairFile = $state<UnavailableFile | null>(null);
    let repairing = $state(false);
    let importStarting = $state(false);
    let scanStarting = $state(false);
    let importJob = $state<Job<ImportReport>>({ state: 'idle' });
    let scanJob = $state<Job<ScanReport>>({ state: 'idle' });
    let importError = $state('');
    let scanError = $state('');
    let repairMessage = $state('');
    let limit = $state(50);
    let active = true;
    let importTimer: ReturnType<typeof setTimeout>;
    let scanTimer: ReturnType<typeof setTimeout>;

    function message(error: unknown) {
        return error instanceof Error ? error.message : 'The request failed. Try again.';
    }

    async function pollImport() {
        try {
            const job = await adminFiles.importStatus();
            if (!active) return;
            importJob = job;
            importError = '';
            if (job.state === 'running') importTimer = setTimeout(pollImport, 1200);
        } catch (err) {
            if (!active) return;
            importError = message(err);
            if (importJob.state === 'running') importTimer = setTimeout(pollImport, 3000);
        }
    }

    async function pollScan() {
        try {
            const job = await adminFiles.scanStatus();
            if (!active) return;
            scanJob = job;
            scanError = '';
            if (job.state === 'running') scanTimer = setTimeout(pollScan, 1200);
        } catch (err) {
            if (!active) return;
            scanError = message(err);
            if (scanJob.state === 'running') scanTimer = setTimeout(pollScan, 3000);
        }
    }

    onMount(() => {
        void pollImport();
        void pollScan();
        return () => {
            active = false;
            clearTimeout(importTimer);
            clearTimeout(scanTimer);
        };
    });

    async function startImport() {
        importStarting = true;
        importError = '';
        try {
            await adminFiles.startImport(directory);
            importJob = { state: 'running' };
            await pollImport();
        } catch (err) {
            importError = message(err);
        } finally {
            importStarting = false;
        }
    }

    async function startScan() {
        scanStarting = true;
        scanError = '';
        repairMessage = '';
        limit = 50;
        try {
            await adminFiles.startScan();
            scanJob = { state: 'running' };
            await pollScan();
        } catch (err) {
            scanError = message(err);
        } finally {
            scanStarting = false;
        }
    }

    async function onSelected(path: string) {
        picker = null;
        const file = repairFile;
        if (!file) {
            directory = path;
            return;
        }
        repairFile = null;
        repairMessage = '';
        scanError = '';
        repairing = true;
        try {
            await adminFiles.repair(file, path);
            await pollScan();
            repairMessage = `Path updated to ${path}`;
        } catch (err) {
            scanError = message(err);
        } finally {
            repairing = false;
        }
    }
</script>

<svelte:head><title>Admin settings · GL</title></svelte:head>

<Header />
<main>
    <h1>Admin settings</h1>
    <section aria-labelledby="import-title">
        <h2 id="import-title">Files importer</h2>
        <p>Register media already on the server. The importer scans subfolders and reads tag and note sidecars.</p>
        <div class="actions">
            <button type="button" disabled={importStarting || importJob.state === 'running'}
                onclick={() => picker = 'directory'}>Choose folder</button>
            {#if directory}<code>{directory}</code>{/if}
            <button type="button" class="primary" disabled={!directory || importStarting || importJob.state === 'running'}
                onclick={startImport}>Start import</button>
        </div>
        {#if importError}<p class="error" role="alert">{importError}</p>{/if}
        {#if importJob.state === 'running'}
            <p role="status">Import in progress…</p>
        {:else if importJob.state === 'failed'}
            <p class="error" role="alert">{importJob.error}</p>
        {:else if importJob.state === 'done'}
            <p role="status">Imported {importJob.report.imported} · Skipped {importJob.report.skipped} duplicates · Failed {importJob.report.failed}</p>
            {#if importJob.report.errors.length}
                <details>
                    <summary>Import errors ({importJob.report.errors.length} shown)</summary>
                    <ul>{#each importJob.report.errors as error}<li>{error}</li>{/each}</ul>
                </details>
            {/if}
        {/if}
    </section>

    <section aria-labelledby="scan-title">
        <h2 id="scan-title">File availability</h2>
        <p>Check the stored media paths and choose a new location for a missing file.</p>
        <div class="actions">
            <button type="button" class="primary" disabled={scanStarting || scanJob.state === 'running' || repairing}
                onclick={startScan}>Run scan</button>
        </div>
        {#if scanError}<p class="error" role="alert">{scanError}</p>{/if}
        {#if repairMessage}<p role="status">{repairMessage}</p>{/if}
        {#if repairing}<p role="status">Checking and saving the selected file…</p>{/if}
        {#if scanJob.state === 'running'}
            <p role="status">Scan in progress…</p>
        {:else if scanJob.state === 'failed'}
            <p class="error" role="alert">{scanJob.error}</p>
        {:else if scanJob.state === 'done'}
            <p role="status">Checked {scanJob.report.checked} · Unavailable {scanJob.report.unavailable.length}</p>
            {#if scanJob.report.unavailable.length}
                <ul class="missing">
                    {#each scanJob.report.unavailable.slice(0, limit) as file (file.id)}
                        <li>
                            <div>
                                <code>{file.path}</code>
                                <small>{file.reason}</small>
                            </div>
                            <button type="button" disabled={repairing || scanStarting}
                                onclick={() => { repairFile = file; picker = 'file'; }}>Choose file</button>
                        </li>
                    {/each}
                </ul>
                {#if limit < scanJob.report.unavailable.length}
                    <button type="button" onclick={() => limit += 50}>Show more</button>
                {/if}
            {/if}
        {/if}
    </section>
</main>

{#if picker}
    <MediaPicker mode={picker} onSelect={onSelected} onCancel={() => { picker = null; repairFile = null; }} />
{/if}

<style>
    main { width: min(70rem, 100%); margin: 0 auto; padding: 1.25rem; display: grid; gap: 1rem; color: #0f172a; }
    h1 { font-size: 1.5rem; font-weight: 700; }
    h2 { font-size: 1.2rem; font-weight: 650; }
    section { background: white; border: 1px solid #e2e8f0; border-radius: 12px; padding: 1.1rem; display: grid; gap: .75rem; }
    p, small { color: #475569; }
    .actions { display: flex; flex-wrap: wrap; align-items: center; gap: .65rem; }
    button { width: fit-content; border: 1px solid #cbd5e1; border-radius: 7px; padding: .45rem .7rem; cursor: pointer; background: white; }
    button:disabled { opacity: .55; cursor: default; }
    .primary { background: #1d4ed8; color: white; }
    .error { color: #b91c1c; }
    .missing { display: grid; gap: .45rem; }
    .missing li { display: flex; flex-wrap: wrap; justify-content: space-between; align-items: center; gap: .5rem; background: #f8fafc; border-radius: 7px; padding: .6rem; }
    .missing small { display: block; }
    li, p, code { overflow-wrap: anywhere; }
    details ul { list-style: disc; padding-left: 1.25rem; }
</style>
