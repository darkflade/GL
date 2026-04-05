import type {UUID} from "$lib/domain";

export function postIDFromURL(params: URLSearchParams): UUID | null {
    return params.get("id");
}

