
<script lang="ts">
    import type {UserInfo} from "$lib/domain/value-objects/auth";
    import { repositories } from "$lib/composition/repositories";
    import { register } from "$lib/application/use-cases/authentication";

    let userInfo: UserInfo = $state({
        username: "",
        password: ""
    })

    async function handleRegister(event: SubmitEvent) {
        event.preventDefault()

        const response = await register(repositories.authentication, userInfo)

        console.log(`[INFO] Register state: ${response}`)

    }

</script>


<div>
    <h1>Register</h1>

    <form onsubmit={handleRegister}>
        <input type="text" placeholder="Username" bind:value={userInfo.username} required />
        <input type="password" placeholder="Password" bind:value={userInfo.password} required />
        <input type="submit"/>
    </form>
</div>
