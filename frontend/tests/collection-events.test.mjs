import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import ts from 'typescript';
const source = fs.readFileSync(new URL('../src/lib/features/collections/edit-collection.ts', import.meta.url), 'utf8');
const js = ts.transpileModule(source, { compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.ES2022 } }).outputText;
const { collectionEvents, itemContent } = await import(`data:text/javascript;base64,${Buffer.from(js).toString('base64')}`);
const original = ['a', 'b', 'c', 'd'].map(id => ({ id, position: 0, content: { Note: id } }));
function verify(draft) {
    const events = collectionEvents(original, draft);
    const rows = original.map(item => ({ id: item.id, content: itemContent(item.content) }));
    for (const event of events) {
        const index = rows.findIndex(item => item.id === event.item_id);
        if (event.op === 'remove') rows.splice(index, 1);
        if (event.op === 'edit') rows[index].content = event.content;
        if (event.op === 'move') {
            const [item] = rows.splice(index, 1);
            rows.splice(event.after_id === null ? 0 : rows.findIndex(row => row.id === event.after_id) + 1, 0, item);
        }
        if (event.op === 'add') rows.splice(event.after_id === null ? 0 : rows.findIndex(row => row.id === event.after_id) + 1, 0, { content: event.content });
    }
    assert.deepEqual(rows.map(row => row.content), draft.map(row => itemContent(row.content)));
}
test('unchanged collection emits no events', () => assert.deepEqual(collectionEvents(original, original), []));
test('mixed replacements, removals, reordering and adjacent additions preserve draft order', () => {
    for (let mask = 0; mask < 16; mask++) {
        for (let shift = 0; shift < 4; shift++) {
            const kept = original.filter((_, i) => mask & (1 << i)).toReversed();
            const draft = [...kept.slice(shift), ...kept.slice(0, shift)].map(item => ({ ...item, content: { Note: item.id + ' edited' } }));
            for (let index = draft.length; index >= 0; index--) {
                draft.splice(index, 0, { key: `n${index}`, content: { Note: `new ${index}` } }, { key: `p${index}`, content: { Post: { id: `post-${index}` } } });
            }
            verify(draft);
        }
    }
    verify([]);
});
