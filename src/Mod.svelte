<script lang="ts">
    import { tauri } from "@tauri-apps/api";
    import { bind } from "svelte/internal";
    import type { Mod } from "./structures";
    export let index : number;
    export let mod : Mod;

    async function update_mod_config() {
        let response = await tauri.invoke("update_mod_config", {"index" : index, "newMod" : mod});
    }
</script>

<span class="mod">
    <span class="left">
        <input type="checkbox" bind:checked={mod.enabled} on:change="{() => update_mod_config()}">
        {mod.name}
        {mod.version}
    </span>
    {#if mod.version != mod.newest_version && !mod.do_update}
        <span class="newer-version" on:click="{() => {mod.do_update = true; update_mod_config()}}">🔼{mod.newest_version}</span>
    {/if}
</span>

<style>
    .mod {
        display: flex;
        flex-direction: row;
        width: 100%;
        text-align: left;
        justify-content: space-between;
    }

    .newer-version {
        color: var(--primary);
        font-weight: 600;
        display: inline-block;
        text-align: right;
        transition: font-size 0.6s ease-in-out;
    }

    .newer-version:hover {
        color: var(--green);
        font-size: 1.1em;
        cursor: pointer;
    }
</style>