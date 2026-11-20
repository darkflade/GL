# Tag management

The tags page has a searchable list and three workspaces: Tag editor, Hierarchy graph and Aliases. Search matches name substrings or UUID (first 50 results; narrow the query for more). The editor shows UUID, category, post count and direct parent/child/alias counts.

Edit saves name and category through `PATCH /api/tags`. Delete asks for explicit confirmation and removes tag assignments, aliases and branches, while leaving posts intact. Merge into lets the user select the tag to keep and explicitly confirm deletion of the source.

Merge event:

```json
{"events":[{"op":"merge","source_id":"source-uuid","target_id":"target-uuid"}]}
```

The source and target must exist and differ. The transaction unions post and collection assignments without duplicates, redirects direct hierarchy edges and aliases, removes self-links, deletes the source, recalculates target post count and rebuilds the transitive closure. A hierarchy cycle rejects and rolls back the whole operation with HTTP 409. The target retains its own name/category. Tag/relation mutation batches share a transaction advisory lock to prevent concurrent graph edits bypassing cycle checks.

`GET /api/tags/{id}/neighborhood` returns `{tag, parents, children, aliases, roots}`. Roots are top-level ancestors, parents/children are direct links and aliases are symmetric direct pairs. The graph initially shows up to four parents and eight children; Show all reveals additional neighbors, and Explore changes the focus. Search adds other tags as nodes. Drag title bars to move nodes, or drag a blue output onto another node to add parent → child links. The Parent/Child selectors provide a keyboard alternative. Remove a branch in the Connections section. Save graph commits the changed links in one batch (unlinks first); node positions are temporary presentation layout.

Aliases are managed separately: add a bidirectional pair or remove an existing pair using existing `alias`/`unalias` relation events. They retain both tag records and are different from a merge.

No schema migration is required for tag management or the playback shortcut.

Verification: `npm --prefix frontend test` includes cycle and graph-batch tests. Database merge tests are opt-in: set `TAG_TEST_DATABASE_URL`, then run `SQLX_OFFLINE=true cargo test --manifest-path backend/Cargo.toml merge_tests -- --ignored`. They create only temporary tables with `search_path=pg_temp` and roll back, testing actual repository SQL without modifying existing data.
