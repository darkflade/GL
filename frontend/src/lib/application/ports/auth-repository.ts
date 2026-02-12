import type {UserInfo} from "$lib/domain/value-objects/auth";

export interface AuthRepository {
    login(userInfo: UserInfo): Promise<string>;
    logout(): Promise<string>;
    me(): Promise<string>;
    register(userInfo: UserInfo): Promise<string>;

}