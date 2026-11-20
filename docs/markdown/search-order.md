# UUID search and order

Posts and collection search accept `sort`: `relevance` (default), `uuid_asc` or `uuid_desc`. UUID order applies before pagination and is supported by both post pagination modes and collection keyset pagination. Ascending UUIDv7 normally shows older objects first; descending shows newer first. Sorting does not change the stored order of items inside a collection.

Use `id:<UUID`, `id:>UUID` or `id:=UUID` in the existing search input. Optional braces around UUID are accepted. Filters are strict comparisons and combine with AND, so a range can be entered as `id:>lower-uuid id:<upper-uuid`. Filters combine with existing required, optional and excluded tags. Malformed ID filters return HTTP 400.

HTTP clients can place these tokens in `text_query` or `tag_query.must`. Remaining `text_query` words filter title/description. UUID filters are removed from tag and text matching. The two existing search controls remain separate. URL parameters preserve both sort and ID filters; choosing a new sort starts pagination again.

The collections index includes New collection (empty), Edit details (title/description, including clearing a description), and Delete with confirmation. Deleting a collection removes its items and preferences/position through cascading foreign keys; source posts remain.
