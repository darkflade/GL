import type {SearchPostsQuery} from "$lib/domain";

export function serializeQuery(query: SearchPostsQuery): string {
    const params = new URLSearchParams();

    Object.entries(query).forEach(([key, value]) => {
        if (!value || value.length === 0) {
            return;
        }

        for (const item of value) {
            params.append(key, item);
        }
    });

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

    return {
        must: readValues("must"),
        should: readValues("should"),
        must_not: readValues("must_not"),
    }
}

export function parseSearch(tagString: string): SearchPostsQuery {
    const tokens = tagString
        .trim()
        .split(/\s+/)
        .filter(Boolean)

    const searchPostsQuery: SearchPostsQuery = {
        must: [],
        should: [],
        must_not: [],
    }

    for (const token of tokens) {
        if (token === "~" || token === "-") {
            continue
        }

        switch (token[0]) {
            case "~":
                const shouldTag = token.slice(1).replaceAll("_", " ").trim();
                if (shouldTag.length > 0) searchPostsQuery.should.push(shouldTag);
                break
            case "-":
                const mustNotTag = token.slice(1).replaceAll("_", " ").trim();
                if (mustNotTag.length > 0) searchPostsQuery.must_not.push(mustNotTag);
                break
            default:
                searchPostsQuery.must.push(token.replaceAll("_", " "))
                break
        }
    }

    return searchPostsQuery
}

export function toSearchInput(query: SearchPostsQuery): string {
    const must = query.must.map((value) => value.replaceAll(" ", "_"))
    const should = query.should.map((value) => `~${value.replaceAll(" ", "_")}`)
    const mustNot = query.must_not.map((value) => `-${value.replaceAll(" ", "_")}`)

    return [...must, ...should, ...mustNot].join(" ").trim()
}

export function escapeTagForSearch(tag: string): string {
    return tag.trim().replaceAll(" ", "_")
}
