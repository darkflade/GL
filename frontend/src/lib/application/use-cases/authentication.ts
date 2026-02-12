import type {AuthRepository} from "$lib/application/ports/auth-repository";
import type {UserInfo} from "$lib/domain/value-objects/auth";

export const login = (repo: AuthRepository, userInfo: UserInfo) => {
    return repo.login(userInfo)
}

export const logout = (repo: AuthRepository) => {
    return repo.logout()
}

export const me = (repo: AuthRepository) => {
    return repo.me()
}

export const register = (repo: AuthRepository, userInfo: UserInfo) => {
    return repo.register(userInfo)
}