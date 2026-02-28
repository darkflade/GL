import type { SearchPostsQuery } from "$lib/domain";

export function serializeQuery(query: SearchPostsQuery): string {
    const params = new URLSearchParams();

    for (const key of ["must", "should", "must_not"] as const) {
        const values = query.tag_query[key];
        if (!values || values.length === 0) {
            continue;
        }

        for (const item of values) {
            params.append(key, item);
        }
    }

    if (query.text_query) {
        params.set("q", query.text_query);
    }

    params.set("mode", query.cursor.mode);
    if (query.cursor.mode === "offset") {
        if (query.cursor.page > 0) {
            params.set("page", String(query.cursor.page));
        }
        if (query.cursor.page_size !== undefined) {
            params.set("page_size", String(query.cursor.page_size));
        }
    } else {
        if (query.cursor.last_id) {
            params.set("last_id", query.cursor.last_id);
        }
        if (query.cursor.last_score !== undefined) {
            params.set("last_score", String(query.cursor.last_score));
        }
        if (query.cursor.limit !== undefined) {
            params.set("limit", String(query.cursor.limit));
        }
    }

    return params.toString();
}

export function deserializeQuery(params: URLSearchParams): SearchPostsQuery {
    const readValues = (key: "must" | "should" | "must_not") => {
        const all = params.getAll(key).filter(Boolean);
        if (all.length > 0) {
            return all;
        }

        // Backward compatibility with old CSV links.
        return params.get(key)?.split(",").map((item) => item.trim()).filter(Boolean) ?? [];
    };

    const mode = params.get("mode") === "keyset" ? "keyset" : "offset";
    const rawPage = Number.parseInt(params.get("page") ?? "0", 10);
    const page = Number.isFinite(rawPage) && rawPage > 0 ? rawPage : 0;

    return {
        tag_query: {
            must: readValues("must"),
            should: readValues("should"),
            must_not: readValues("must_not"),
        },
        text_query: params.get("q") ?? "",
        cursor:
            mode === "keyset"
                ? {
                      mode,
                      last_id: params.get("last_id") ?? undefined,
                      last_score: params.get("last_score") ? Number(params.get("last_score")) : undefined,
                      limit: params.get("limit") ? Number(params.get("limit")) : undefined,
                  }
                : {
                      mode,
                      page,
                      page_size: params.get("page_size") ? Number(params.get("page_size")) : undefined,
                  },
    }
}

export function parseSearch(tagString: string): SearchPostsQuery {
    const tokens = tagString
        .trim()
        .split(/\s+/)
        .filter(Boolean)

    const searchPostsQuery: SearchPostsQuery = {
        tag_query: {
            must: [],
            should: [],
            must_not: [],
        },
        text_query: "",
        cursor: { mode: "offset", page: 0 },
    }

    for (const token of tokens) {
        if (token === "~" || token === "-") {
            continue
        }

        switch (token[0]) {
            case "~":
                const shouldTag = token.slice(1).replaceAll("_", " ").trim();
                if (shouldTag.length > 0) searchPostsQuery.tag_query.should.push(shouldTag);
                break
            case "-":
                const mustNotTag = token.slice(1).replaceAll("_", " ").trim();
                if (mustNotTag.length > 0) searchPostsQuery.tag_query.must_not.push(mustNotTag);
                break
            default:
                searchPostsQuery.tag_query.must.push(token.replaceAll("_", " "))
                break
        }
    }

    return searchPostsQuery
}

export function toSearchInput(query: SearchPostsQuery): string {
    const must = query.tag_query.must.map((value) => value.replaceAll(" ", "_"))
    const should = query.tag_query.should.map((value) => `~${value.replaceAll(" ", "_")}`)
    const mustNot = query.tag_query.must_not.map((value) => `-${value.replaceAll(" ", "_")}`)

    return [...must, ...should, ...mustNot].join(" ").trim()
}

export function escapeTagForSearch(tag: string): string {
    return tag.trim().replaceAll(" ", "_")
}
