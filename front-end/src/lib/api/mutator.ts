export const customFetch = async <T>(
    url: string,
    options: RequestInit = {}
): Promise<T> => {
    const baseUrl = import.meta.env.VITE_API_BASE_URL ?? 'http://localhost:3000';

    const response = await fetch(`${baseUrl}${url}`, {
        ...options,
        credentials: 'include',
        headers: {
            'Content-Type': 'application/json',
            ...options.headers,
        },
    });

    const data = response.status === 204 ? undefined : await response.json();

    return { data, status: response.status, headers: response.headers } as T;
};
