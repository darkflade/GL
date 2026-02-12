<script lang="ts">
    import type {UserInfo} from "$lib/domain/value-objects/auth";
    import { login } from "$lib/application/use-cases/authentication";
    import {repositories} from "$lib/composition/repositories";

    let userInfo: UserInfo = $state({
        username: "",
        password: ""
    })

    async function handleLogin(event: SubmitEvent) {
        event.preventDefault()

        const response = await login(repositories.authentication, userInfo)
        console.log(`[INFO] Login state: ${response}`)
    }

</script>

<div>
    <h1>Login Page</h1>
    <div>
        <form onsubmit={handleLogin}>
            <input type="text" placeholder="Nickname" bind:value={userInfo.username} required />
            <input type="password" placeholder="Password" bind:value={userInfo.password} required />
            <input type="submit" value="submit"/>
        </form>

    </div>
</div>
