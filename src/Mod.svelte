<script lang="ts">
    import { tauri } from "@tauri-apps/api";
    import { bind } from "svelte/internal";
    import type { Mod } from "./structures";
    export let index : number;
    export let mod : Mod;
    export let reloadModListFunction : () => void;

    async function update_mod_config() {
        await tauri.invoke("update_mod_config", {"index" : index, "newMod" : mod});
    }

    async function remove_mod() {
        await tauri.invoke("remove_mod", {"index" : index});
        reloadModListFunction();
    }
    
    async function update_mod(_ : Event) {
        mod.do_update = true;
        update_mod_config()
    }
</script>

<span class="mod">
    <span>
        <input type="checkbox" bind:checked={mod.enabled} on:change="{() => update_mod_config()}">
        {mod.name}
        {mod.version}
    </span>
    <span>
        <span class="remove" on:click="{() => remove_mod()}" on:keydown="{() => remove_mod()}">🗑️</span>
        {#if mod.version != mod.newest_version}
            <span class="update">
                {#if !mod.do_update}
                <span class="newer-version" on:click="{update_mod}" on:keydown="{update_mod}">🔼{mod.newest_version}</span>
                {:else}
                    Will Update
                {/if}
            </span>
        {/if}
    </span>
    
</span>

<style>
    .mod {
        display: flex;
        flex-direction: row;
        width: 100%;
        text-align: left;
        justify-content: space-between;
    }

    .update {
        color: var(--primary);
        font-weight: 500;
        display: inline-block;
        text-align: right;
    }

    .newer-version {
        font-weight: 600;
        transition: font-size 0.6s ease-in-out;
    }

    .newer-version:hover {
        color: var(--green);
        font-size: 1.1em;
        cursor: pointer;
    }

    .remove {
        transition: all 0.4s ease-in-out;
        border: solid 1px rgba(0, 0, 0, 0);
        border-radius: 5px;
    }

    .remove:hover {
        cursor: pointer;
        font-size: 1.2em;
        border: solid 1px var(--red);
        border-radius: 5px;
    }
</style>