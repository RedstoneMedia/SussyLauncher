<script lang="ts">
	import { tauri, event } from '@tauri-apps/api';
	import { listen } from '@tauri-apps/api/event';
	import Manage from './Manage.svelte';

	let playButtonText = "Play";
	let message = "";
	const messages = ["", "baka", "let the sus begin", "Amogus", "Sussy", "When the Imposter is SUS", "DING DING DING DING DING DING", "DING DING DING", "this ain't funny"];
	let currentMessageIndex = 0;
	let manage = false;
	let error_message = "";
	let doingPlay = false;

	async function play() {
		doingPlay = true;
		const unlisten = await listen('progress', event => {
			playButtonText = event.payload as string;
		});
		try {
			await tauri.invoke("play");
		} catch(e) {
			playButtonText = "We made a little sussy wussy";
			error_message = e;
			console.error(e);
		}
		unlisten();
		doingPlay = false;
	}

	setInterval(() => {
		if (currentMessageIndex >= messages.length) {
			currentMessageIndex = 1;
		}
		message = messages[currentMessageIndex];
		currentMessageIndex++;
	}, 5000);

	setInterval(async () => {
		if (doingPlay || error_message != "") return;
		if (await tauri.invoke("is_among_us_running")) {
			playButtonText = "Sussed"
		} else {
			playButtonText = "Play"
		}
	}, 1000);
</script>

<main style="grid-template-columns: 1fr {manage ? "1.3fr" : "auto"};">
	<h1 style="grid-area: ti;">Sussy Launcher</h1>
	<div id="buttons" style="grid-area: bu;">
		<button class="wide-button" on:click="{() => manage = !manage}">Manage</button>
		<button class="wide-button" on:click="{() => play()}">{playButtonText}</button>
		<p id="error">{error_message}</p>
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
		overflow: hidden;
		grid-template-columns: 1fr 0.8fr;
		grid-template-rows: 0.2fr 1fr 0fr;
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
		color: var(--red);
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
		width: 60%;
		word-wrap: break-word;
	}

	#message {
		text-align: center;
		margin: 0;
		transition: all 0.3s ease-in-out;
		color: var(--primary);
		font-weight: 200;
	}

	#error {
		text-align: center;
		margin: 0;
		color: var(--red);
		font-weight: bold;
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