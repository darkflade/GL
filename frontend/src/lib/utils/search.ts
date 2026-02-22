import type {SearchPostsQuery} from "$lib/domain";

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

    if (query.cursor?.page && query.cursor.page > 0) {
        params.set("page", String(query.cursor.page));
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

    const rawPage = Number.parseInt(params.get("page") ?? "0", 10);
    const page = Number.isFinite(rawPage) && rawPage > 0 ? rawPage : 0;

    return {
        tag_query: {
            must: readValues("must"),
            should: readValues("should"),
            must_not: readValues("must_not"),
        },
        cursor: {page},
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
        cursor: {page: 0},
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
