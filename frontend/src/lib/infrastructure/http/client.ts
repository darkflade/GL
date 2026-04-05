const BASE_URL = "/api";

export class ApiError extends Error {
    code?: number;

    constructor(message: string, code?: number) {
        super(message);
        this.name = "ApiError";
        this.code = code;
    }
}

interface BackendErrorBody {
    code?: number;
    message?: string;
    error?: string;
}

function isBackendErrorBody(value: unknown): value is BackendErrorBody {
    if (!value || typeof value !== "object") return false;
    const candidate = value as BackendErrorBody;
    return (
        typeof candidate.message === "string" ||
        typeof candidate.error === "string" ||
        typeof candidate.code === "number"
    );
}

async function readBody(response: Response): Promise<unknown> {
    const contentType = response.headers.get("content-type") ?? "";
    if (contentType.includes("application/json")) {
        return response.json();
    }
    return response.text();
}

function toApiError(errorBody: unknown, fallbackStatus: number): ApiError {
    if (isBackendErrorBody(errorBody)) {
        const message = errorBody.message ?? errorBody.error ?? `Error ${fallbackStatus}`;
        const code = errorBody.code ?? fallbackStatus;
        return new ApiError(message, code);
    }

    const message = typeof errorBody === "string" && errorBody.trim() ? errorBody : `Error ${fallbackStatus}`;
    return new ApiError(message, fallbackStatus);
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
        throw toApiError(errorBody, response.status)
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
    patch: <T>(url: string, body?: unknown) =>
        request<T>(url, {
            method: "PATCH",
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
            throw toApiError(errorBody, response.status)
        }
        return await readBody(response) as T
    }
}
