import type { ErrorResponse } from "$lib/domain/value-objects/error";

const BASE_URL = "/api";

class ApiError extends Error {
    code?: number;

    constructor(message: string, code?: number) {
        super(message);
        this.name = "ApiError";
        this.code = code;
    }
}

function isErrorResponse(value: unknown): value is ErrorResponse {
    if (!value || typeof value !== "object") return false;
    const candidate = value as ErrorResponse;
    return typeof candidate.message === "string" && typeof candidate.code === "number";
}

async function readBody(response: Response): Promise<unknown> {
    const contentType = response.headers.get("content-type") ?? "";
    if (contentType.includes("application/json")) {
        return response.json();
    }
    return response.text();
}

export async function request<T>(
    endpoint: string,
    options: RequestInit = {}
): Promise<T> {
    const url = `${BASE_URL}${endpoint}`

    const headers = {
        "Content-Type": "application/json",
        ...options.headers,
    }

    const response = await fetch(url, { credentials: "include", ...options, headers })

    if (!response.ok) {
        const errorBody = await readBody(response)
        if (isErrorResponse(errorBody)) {
            throw new ApiError(errorBody.message, errorBody.code)
        }
        const errorText = typeof errorBody === "string" && errorBody.trim() ? errorBody : `Error ${response.status}`
        throw new ApiError(errorText)
    }

    if (response.status === 204) return {} as T

    return await readBody(response) as T
}

export const api = {
    get: <T>(url: string) => request<T>(url, { method: "GET" }),
    post: <T>(url: string, body?: unknown) =>
        request<T>(url, {
            method: "POST",
            body: body === undefined ? undefined : JSON.stringify(body),
        }),
    delete: <T>(url: string) => request<T>(url, { method: "DELETE" }),
    upload: async <T>(url: string, formData: FormData) => {
        const response = await fetch(`${BASE_URL}${url}`, {
            method: "POST",
            body: formData,
            credentials: "include",
        })
        if (!response.ok) {
            const errorBody = await readBody(response)
            if (isErrorResponse(errorBody)) {
                throw new ApiError(errorBody.message, errorBody.code)
            }
            const errorText = typeof errorBody === "string" && errorBody.trim() ? errorBody : `Error ${response.status}`
            throw new ApiError(errorText)
        }
        return await readBody(response) as T
    }
}
