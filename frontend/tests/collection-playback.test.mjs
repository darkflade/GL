import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import ts from 'typescript';
const source = fs.readFileSync(new URL('../src/lib/features/collections/playback-order.ts', import.meta.url), 'utf8');
const js = ts.transpileModule(source, { compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.ES2022 } }).outputText;
const { nextIndex } = await import(`data:text/javascript;base64,${Buffer.from(js).toString('base64')}`);
const items = ['Picture', 'Audio', 'Video', 'Picture'].map((media_type, id) => ({ id: String(id), content: { Post: { file: { media_type } } } }));
const settings = { skip_audio: true, skip_video: false, loop_collection: true };
test('forward/backward navigation skips configured media and wraps', () => {
    assert.equal(nextIndex(items, 0, 1, settings), 2);
    assert.equal(nextIndex(items, 2, -1, settings), 0);
    assert.equal(nextIndex(items, 3, 1, settings), 0);
    assert.equal(nextIndex(items, 0, -1, settings), 3);
    assert.equal(nextIndex(items, 0, 1, { ...settings, skip_video: true }), 3);
});
test('finite playback stops at boundaries; skipped-only and empty lists terminate', () => {
    assert.equal(nextIndex(items, 3, 1, { ...settings, loop_collection: false }), null);
    assert.equal(nextIndex(items, 0, -1, { ...settings, loop_collection: false }), null);
    assert.equal(nextIndex([items[1]], 0, 1, settings), null);
    assert.equal(nextIndex([], 0, 1, settings), null);
    assert.equal(nextIndex([items[0]], 0, 1, settings), 0);
    assert.equal(nextIndex([items[1]], 0, 1, { ...settings, skip_audio: false }), 0);
});
