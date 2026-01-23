import {api} from "$lib/api/client";
import type {Post, SearchPostsQuery, Tag} from "$lib/domain/models";

export const searchAllPosts = async () => {
    return api.post<Post[]>('/posts/search', {
        must: [],
        should: [],
        must_not: [],
    })
}

export const searchPosts = async (query: SearchPostsQuery) => {
    console.log(`Triggered searchPosts`)
    return api.post<Post[]>('/posts/search', {
        must: query.must,
        should: query.should,
        must_not: query.must_not,
    })
}

export const searchTags = async (query: string = '') => {
    return api.get<Tag[]>(`/tags/search?query=${query}`)
}