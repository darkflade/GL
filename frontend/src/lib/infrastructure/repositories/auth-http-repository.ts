import type {AuthRepository} from "$lib/application/ports/auth-repository";
import {api} from "$lib/infrastructure/http/client";

export const authHttpRepository: AuthRepository = {
    login: (userInfo) => {
        return api.post<string>("/auth/login", userInfo)
    },
    logout: () => {
        return api.post<string>("/auth/logout")
    },

    me: () => {
       return api.get<string>("/auth/me")
    },

    register: (userInfo) => {
        return api.post<string>("/auth/register", userInfo)
    }
}
