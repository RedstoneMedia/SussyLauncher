<script lang="ts">
    import { tauri } from '@tauri-apps/api';
    import { fly, fade } from 'svelte/transition';
    export let hidePanleFunction : () => void;

    let name = "";
    let location = "";
    let version = "";
    let showOtherInputs = false;

    async function addMod() {
        let response = await tauri.invoke("add_mod", {name : name, version: version, location : location});
        if (typeof(response) == "string") {
			console.error(response);
            return;
		}
        hidePanleFunction();
    }

    function nameKeyDown(event : KeyboardEvent) {
        if (event.key == "Enter") {
            showOtherInputs = true;
        }
    }
</script>

<main transition:fly={{ y: 50, duration: 500 }} class="panel">
    <div class="named-input">
        <p>Name:</p>
        <input bind:value="{name}" on:blur="{() => showOtherInputs = true}" on:keyup="{nameKeyDown}">
    </div>
    
    {#if showOtherInputs}
        <div transition:fade={{ duration: 200 }}>
            <div class="named-input">
                <p>Location:</p>
                <input bind:value="{location}" type="url">
            </div>
            <div class="named-input">
                <p>Version:</p>
                <input bind:value="{version}">
            </div>
            <button on:click="{() => addMod()}">Add</button>
        </div>
    {/if}
    
</main>

<style>

    main {
        width: 80%;
    }

    input {
        background-color: var(--background-color);
    }

    .named-input {
        margin: 10px 0px 10px 0px;
        display: flex;
        flex-direction: row;
        justify-content: space-between;
        place-items: center;
        width: 100%;
    }

    .named-input input, .named-input p {
        margin: auto 0;
    }

    .named-input input {
        width: 70%;
    }

</style>