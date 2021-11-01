<script lang="ts">
	import { tauri } from '@tauri-apps/api';
	import Manage from './Manage.svelte';

	let playButtonText = "Play";
	let message = "";
	const messages = ["", "baka", "let the sus begin", "Amogus", "Sussy", "When the Imposter is SUS", "DING DING DING DING DING DING", "DING DING DING", "this ain't funny"];
	let currentMessageIndex = 0;
	let manage = false;

	async function play() {
		playButtonText = "Sussing ...";
		let respose = await tauri.invoke("play");
		if (respose !== null) {
			playButtonText = "We made a little Sussy Wussy";
			console.error(respose);
		}
	}

	setInterval(() => {
		if (currentMessageIndex >= messages.length) {
			currentMessageIndex = 1;
		}
		message = messages[currentMessageIndex];
		currentMessageIndex++;
	}, 5000);
</script>

<main>
	<h1 style="grid-area: ti;">Sussy Launcher</h1>
	<div id="buttons" style="grid-area: bu;">
		<button class="wide-button" on:click="{() => manage = !manage}">Manage</button>
		<button class="wide-button" on:click="{() => play()}">{playButtonText}</button>
	</div>
	{#if manage}
		<Manage></Manage>
	{/if}
	<footer style="grid-area: fo;">
		<p id="message">{message}</p>
	</footer>
</main>

<style>

	main {
		text-align: center;
		height: max-content;
		margin: 0 auto;
		color: var(--text-color);
		display: grid;
		grid-template-columns: 1fr auto;
		grid-template-rows: 0.2fr 1fr auto;
		grid-template-areas: "ti ti"
							 "bu ml"
							 "fo fo";
		align-items: center;
		align-content: center;
		justify-content: center;
		place-items: center;
		height: 100%;
	}

	h1 {
		color: #ff3e00;
		text-transform: uppercase;
		font-size: 4em;
		font-weight: 100;
	}

	#buttons {
		display: flex;
		flex-direction: column;
		width: 100%;
		place-items: center;
	}

	.wide-button {
		font-size: 2em;
		width: 40%;
	}

	#message {
		text-align: center;
		margin: 0;
		transition: all 0.3s ease-in-out;
		color: var(--primary);
		font-weight: 200;
	}

	/* Global vars */

	:root {
		--background-color: #34495e;
		--border-color: #3498db;
		--text-color: #ecf0f1;
		--red: #e74c3c;
		--green: #2ecc71;
		--primary:#1abc9c;
		--panel-color: #273c75;
	}
</style>