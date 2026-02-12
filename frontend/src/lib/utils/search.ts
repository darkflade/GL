import type {SearchPostsQuery} from "$lib/domain";

export function serializeQuery(query: SearchPostsQuery): string {
    const params = new URLSearchParams();

    Object.entries(query).forEach(([key, value]) => {
        if (value && value.length > 0) {
            params.set(key, value.join(','))
        }
    })

    return params.toString();
}

export function deserializeQuery(params: URLSearchParams): SearchPostsQuery {
    return {
        must: params.get('must')?.split(',').filter(Boolean) ?? [],
        should: params.get('should')?.split(',').filter(Boolean) ?? [],
        must_not: params.get('must_not')?.split(',').filter(Boolean) ?? [],
    }
}