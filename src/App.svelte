<script lang="ts">
	import { tauri, event } from '@tauri-apps/api';
	import { listen } from '@tauri-apps/api/event';
	import Manage from './Manage.svelte';

	let playButtonText = "Play";
	let message = "";
	const messages = ["", "baka", "let the sus begin", "Amogus", "Sussy", "When the Imposter is SUS", "DING DING DING DING DING DING", "DING DING DING", "🤨"];
	let currentMessageIndex = 0;
	let manage = false;
	let errorMessage = "";
	let doingPlay = false;
	let loading = true;
	let loadingMessage = "Loading ...";

	async function checkIfLoaded() {
		const unlisten = await listen('load', event => {
			loadingMessage = event.payload;
			if (event.payload == "done") {
				loading = false;
				unlisten();
			}
		});
	}

	async function play() {
		if (playButtonText == "Sussed") {return;}
		doingPlay = true;
		manage = false;
		const unlisten = await listen('progress', event => {
			playButtonText = event.payload as string;
		});
		try {
			await tauri.invoke("play");
		} catch(e) {
			playButtonText = "We made a little sussy wussy";
			errorMessage = e;
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
		if (doingPlay || errorMessage != "") return;
		if (await tauri.invoke("is_among_us_running")) {
			playButtonText = "Sussed"
		} else {
			playButtonText = "Play"
		}
	}, 1000);

	checkIfLoaded();
</script>

<main style="grid-template-columns: 1fr {manage ? "1.3fr" : "auto"};">
	<h1 style="grid-area: ti;">Sussy Launcher</h1>
	{#if !loading}
		<div id="buttons" style="grid-area: bu;">
			<button class="wide-button" on:click="{() => manage = !manage && !doingPlay}">Manage</button>
			<button class="wide-button" disabled="{playButtonText == "Sussed"}" on:click="{() => play()}">{playButtonText}</button>
			<p class="error">{errorMessage}</p>
		</div>
		{#if manage}
			<Manage></Manage>
		{/if}
		<footer style="grid-area: fo;">
			<p id="message">{message}</p>
		</footer>
	{:else}
		<h2 class="panel" id="loading-text">{loadingMessage}</h2>
	{/if}
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

	.wide-button:hover {
		font-size: 2.2em;
	}

	#message {
		text-align: center;
		margin: 0;
		transition: all 0.3s ease-in-out;
		color: var(--primary);
		font-weight: 200;
	}
	
	#loading-text {
		grid-area: bu;
		text-align: center;
		color: var(--primary);
		text-shadow: 0px 0px 10px rgba(70, 218, 255, 0.26);
		font-weight: 400;
		width: 80%;
	}

	/* Global vars */

	:root {
		--background: radial-gradient(circle, rgba(52,73,94,1) 0%, rgba(43,58,74,1) 53%, rgba(27,33,39,1) 100%);
		--border-color: #3498db;
		--text-color: #ecf0f1;
		--red: #e74c3c;
		--green: #2ecc71;
		--primary:#1abc9c;
		--panel-color: #273c75;
	}
</style>