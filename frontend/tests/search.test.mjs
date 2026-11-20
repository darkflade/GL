import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import ts from 'typescript';
const source = fs.readFileSync(new URL('../src/lib/utils/search.ts', import.meta.url), 'utf8');
const js = ts.transpileModule(source, { compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.ES2022 } }).outputText;
const { parseSearch, serializeQuery, deserializeQuery, toSearchInput } = await import(`data:text/javascript;base64,${Buffer.from(js).toString('base64')}`);
test('UUID filters coexist with tags and survive URL and input round trips', () => {
    const id='019d5f0c-9974-79a1-ba95-1aaa28dd91c0';
    for (const op of ['<','>','=']) {
        const query=parseSearch(`landscape ~blue_sky -rain id:${op}${id}`);
        query.sort='uuid_asc';query.cursor={mode:'keyset',limit:20};
        assert.deepEqual(query.tag_query,{must:['landscape'],should:['blue sky'],must_not:['rain']});
        assert.equal(query.text_query,`id:${op}${id}`);
        const restored=deserializeQuery(new URLSearchParams(serializeQuery(query)));
        assert.equal(restored.sort,'uuid_asc');assert.equal(restored.text_query,query.text_query);
        const parsed=parseSearch(toSearchInput(restored));
        assert.deepEqual(parsed.tag_query,query.tag_query);assert.equal(parsed.text_query,query.text_query);
    }
});
test('multiple UUID bounds remain separate tokens', () => {
    const q=parseSearch('id:>019d5f0c-9974-79a1-ba95-1aaa28dd9123 id:<019d5f0c-9974-79a1-ba95-1aaa28dd91c0');
    assert.equal(q.tag_query.must.length,0);assert.equal(q.text_query.split(' ').length,2);
});
