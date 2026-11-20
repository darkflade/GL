import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import ts from 'typescript';
const source = fs.readFileSync(new URL('../src/lib/features/tags/graph.ts', import.meta.url), 'utf8');
const js = ts.transpileModule(source, { compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.ES2022 } }).outputText;
const { wouldCycle, relationChanges } = await import(`data:text/javascript;base64,${Buffer.from(js).toString('base64')}`);
test('graph rejects self-links and transitive cycles, allows multiple parents', () => {
    const edges=[{parent:'a',child:'b'},{parent:'b',child:'c'}];
    assert.equal(wouldCycle(edges,'a','a'),true);
    assert.equal(wouldCycle(edges,'c','a'),true);
    assert.equal(wouldCycle(edges,'d','c'),false);
    assert.equal(wouldCycle(edges,'a','c'),false);
});
test('graph batch unlinks before linking to permit reversing a branch', () => {
    const original=[{parent:'a',child:'b'},{parent:'b',child:'c'}];
    const current=[{parent:'b',child:'a'},{parent:'b',child:'c'}];
    assert.deepEqual(relationChanges(original,current),[
        {op:'unlink',parent_id:'a',child_ids:['b']},
        {op:'link',parent_id:'b',child_ids:['a']},
    ]);
    assert.deepEqual(relationChanges(original,[...original]),[]);
    assert.deepEqual(relationChanges([],[]),[]);
});
