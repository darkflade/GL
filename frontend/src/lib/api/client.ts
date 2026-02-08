const BASE_URL = '/api';

export async function request<T>(
    endpoint: string,
    options: RequestInit = {}
): Promise<T> {
    const url = `${BASE_URL}${endpoint}`

    const headers = {
        'Content-Type': 'application/json',
        ...options.headers,
    }

    const response = await fetch(url, { ...options, headers })

    if (!response.ok) {
        const errorText = await response.text()
        throw new Error(errorText || `Error ${response.status}`)
    }

    if (response.status === 204) return {} as T

    return response.json()
}

export const api = {
    get: <T>(url: string) => request<T>(url, { method: 'GET'}),
    post: <T>(url: string, body: any) => request<T>(url, { method: 'POST', body: JSON.stringify(body) }),
    delete: <T>(url: string) => request<T>(url, { method: 'DELETE' }),
    upload: async <T>(url: string, formData: FormData) => {
        const response = await fetch(`${BASE_URL}${url}`, {
            method: 'POST',
            body: formData,
        })
        if (!response.ok) throw new Error(`Error ${response.text}`)
        return await response.json() as Promise<T>
    }
}