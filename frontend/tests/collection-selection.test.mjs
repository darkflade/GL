import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import ts from 'typescript';
const source = fs.readFileSync(new URL('../src/lib/features/collections/selection.ts', import.meta.url), 'utf8');
const js = ts.transpileModule(source, { compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.ES2022 } }).outputText;
const { selectRange, moveSelection, parsePostLinks } = await import(`data:text/javascript;base64,${Buffer.from(js).toString('base64')}`);
const order = ['a', 'b', 'c', 'd', 'e'];
const click = { shiftKey: false, ctrlKey: false, metaKey: false };
test('plain, Ctrl/Cmd and Shift selection and deselection', () => {
    let selected = selectRange(order, new Set(), null, 'b', click);
    selected = selectRange(order, selected, 'b', 'd', { ...click, shiftKey: true });
    assert.deepEqual([...selected], ['b', 'c', 'd']);
    selected = selectRange(order, selected, 'b', 'b', { ...click, ctrlKey: true });
    selected = selectRange(order, selected, 'b', 'd', { ...click, shiftKey: true });
    assert.equal(selected.size, 0);
    selected = selectRange(order, selected, null, 'e', { ...click, metaKey: true });
    selected = selectRange(order, selected, 'e', 'e', click);
    assert.equal(selected.size, 0);
});
test('dragging preserves relative order of a noncontiguous selection', () => {
    const items = order.map(key => ({ key }));
    const selected = new Set(['d', 'b']);
    assert.deepEqual(moveSelection(items, selected, 0).map(i => i.key), ['b', 'd', 'a', 'c', 'e']);
    assert.deepEqual(moveSelection(items, selected, 5).map(i => i.key), ['a', 'c', 'e', 'b', 'd']);
    assert.deepEqual(moveSelection(items, selected, 3).map(i => i.key), ['a', 'c', 'b', 'd', 'e']);
});
test('import accepts arbitrary domains and preserves link order and duplicates', () => {
    const a = '019d5f0c-9974-79a1-ba95-1aaa28dd91c0', b = '019d5f0c-9974-79a1-ba95-1aaa28dd9123';
    assert.deepEqual(parsePostLinks(`http://localhost/post?id=${a}\nhttps://other.example/post?id=${b}\nhttp://localhost/post?id=${a}`), [a, b, a]);
    for (const invalid of ['', 'nonsense', `https://example.com/feed?id=${a}`, 'https://example.com/post?id=bad', `file:///post?id=${a}`]) {
        assert.throws(() => parsePostLinks(invalid));
    }
});
