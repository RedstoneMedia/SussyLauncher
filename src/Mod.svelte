<script lang="ts">

    interface Mod {
        name : String,
        version : String,
        enabled : boolean
    }
    export let mod : Mod;
</script>

<span>
    <input type="checkbox" checked={mod.enabled}>
    {mod.name}
    {mod.version}
</span>