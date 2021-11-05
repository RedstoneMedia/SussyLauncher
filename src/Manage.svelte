<script lang="ts">
    import { fly, fade } from 'svelte/transition';
    import { tauri } from '@tauri-apps/api';
    import Mod from './Mod.svelte';
    import Donwload from './Donwload.svelte';

    let mods : Array<Object> = [];
    let download = false;

    async function getMods() {
		let response = await tauri.invoke("get_mods");
		if (typeof(response) == "string") {
			console.error(response);
		} else {
			mods = response as Array<Mod>;
		}
	}
    getMods();
</script>

<main in:fly={{ y: -50, duration: 1000 }} out:fade={{ duration: 500 }}>
    <div class="panel" id="mod-list-panel">
        <div id="mods">
            {#each mods as mod, index}
                <Mod mod={mod} index={index} reloadModListFunction={getMods}></Mod>
            {/each}
        </div>
        <button style="min-width: 50%;" on:click="{() => {download = !download}}">Download</button>
    </div>
    {#if download}
        <Donwload hidePanleFunction={() => {
            download = false;
            getMods();
        }}></Donwload>
    {/if}
</main>

<style>
    main {
        grid-area: ml;
        display: flex;
        align-items: center;
        place-items: center;
        flex-direction: column;
        width: 100%;
        flex-grow: 0;
    }

    #mod-list-panel {
        padding: 10px 20px 0px 20px;
        width: 80%;
    }

    #mods {
        display: flex;
        flex-direction: column;
        place-items: start;
        margin-bottom: 10px;
    }
</style>