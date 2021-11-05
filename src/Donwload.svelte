<script lang="ts">
    import { tauri } from '@tauri-apps/api';
    import { fly, fade } from 'svelte/transition';
    export let hidePanleFunction : () => void;

    interface AutoCompleteItem {
        name : string,
        location : string
    }

    let name = "";
    let location = "";
    let version = "";
    let showOtherInputs = false;
    let errorMessage = "";
    let autoCompleteItems : Array<AutoCompleteItem> = [];
    let isNameInputActive = false;

    async function addMod() {
        try {
            await tauri.invoke("add_mod", {name : name, version: version, location : location});
        } catch(e) {
            errorMessage = e;
            return;
        }
        hidePanleFunction();
    }

    async function onNameKeyDown(event : KeyboardEvent) {
        if (event.key == "Enter") {
            showOtherInputs = true;
            return;
        }
        autoCompleteItems = await tauri.invoke("get_possible_mods", {name : name});
    }

    function selectAutoCompleteItem(index : number) {
        name = autoCompleteItems[index].name;
        location = autoCompleteItems[index].location;
    }
</script>

<main transition:fly={{ y: 50, duration: 500 }} class="panel">
    <div class="named-input">
        <p>Name:</p>
        <span id="auto-complete-input">
            <input
                bind:value="{name}"
                on:focus="{() => isNameInputActive = true}"
                on:blur="{() => {
                    showOtherInputs = true;
                    isNameInputActive = false
                }}"
                on:keyup="{(e) => onNameKeyDown(e)}"
            >
            {#if autoCompleteItems.length > 0 && isNameInputActive}
                <div id="auto-complete-list">
                    {#each autoCompleteItems as autoCompleteItem, index}
                        <div on:mousedown="{() => selectAutoCompleteItem(index)}" class="auto-complete-item">{autoCompleteItem.name}</div>
                    {/each}
                </div>
            {/if}
        </span>
    </div>
    
    {#if showOtherInputs}
        <div transition:fade={{ duration: 200 }}>
            <div class="named-input">
                <p>Location:</p>
                <input bind:value="{location}" type="url" placeholder="filepath or github link">
            </div>
            <div class="named-input">
                <p>Version:</p>
                <input bind:value="{version}" placeholder="optional">
            </div>
            <button on:click="{() => addMod()}">Add</button>
            <p class="error">{errorMessage}</p>
        </div>
    {/if}
    
</main>

<style>

    main {
        width: 80%;
    }

    main:focus-within {
        min-height: 20vh;
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

    /* Auto Complete Input */

    .named-input input, .named-input #auto-complete-input {
        width: 70%;
    }

    #auto-complete-input {
        position: relative;
    }

    #auto-complete-input input {
        width: 100%;
    }

    #auto-complete-input input:focus {
        border-radius: 8px 8px 0px 0px;
    }

    #auto-complete-list {
        position: absolute;
        display: block;
        margin: auto 0;
        width: 100%;
        border-radius: 0px 0px 8px 8px;
        padding: 0.5em;
        margin: 0 0 0.6em 0;
        box-sizing: border-box;
        border-top-width: 0px;
        border: 1px solid var(--primary);
        background-color: var(--panel-color);
        color: var(--text-color);
        word-wrap: break-word;
        overflow: auto;
        max-height: 15vh;
    }

    .auto-complete-item {
        text-align: left;
    }

    .auto-complete-item:hover {
        cursor: pointer;
        color: var(--primary);
    }

</style>