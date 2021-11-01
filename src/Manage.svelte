<script lang="ts">
    import { fly } from 'svelte/transition';
    import { tauri } from '@tauri-apps/api';
    import Mod from './Mod.svelte';

    let mods : Array<Object> = [];
    async function getMods() {
		let response = await tauri.invoke("get_mods");
		if (typeof(response) == "string") {
			console.error(response);
		} else {
			mods = response as Array<Mod>;
			console.log(mods);
		}
	}
    getMods();
</script>

<main in:fly={{ y: -50, duration: 1000 }} out:fly={{ y: 50, duration: 300 }}>
    <div id="mods">
        {#each mods as mod}
            <Mod mod={mod}></Mod>
        {/each}
    </div>
    <button style="min-width: 50%;">Download</button>
</main>

<style>
    main {
        grid-area: ml;
        padding: 10px 20px 0px 20px;
        margin: 10px;
        background-color: var(--panel-color);
        border: solid var(--border-color);
        border-radius: 10px;
        max-height: 50vh;
        display: block;
        overflow: auto;
    
    }

    #mods {
        display: flex;
        flex-direction: column;
        place-items: start;
        margin-bottom: 10px;
    }
</style>