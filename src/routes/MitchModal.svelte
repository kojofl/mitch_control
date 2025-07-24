<script lang="ts">
	import { invoke } from "@tauri-apps/api/core";

	let { details = $bindable(), id } = $props();
	let recording = $state("Accelerometry");

	async function start_recording() {
		try {
			await invoke("start_recording", { id, recording });
			details = await invoke("get_mitch_details", { id });
		} catch (e) {
			console.log(e);
		}
	}
	async function stop_recording() {
		try {
			await invoke("stop_recording", { id });
			details = await invoke("get_mitch_details", { id });
		} catch (e) {
			console.log(e);
		}
	}

	function modalClose() {
		details = undefined;
	}
</script>

{#if details}
	<header class="flex justify-between">
		<h2 class="h2">{details.name}</h2>
	</header>
	<article>
		<p class="opacity-60">
			{details.state}
		</p>
		<select class="select w-auto" bind:value={recording}>
			<option value="Accelerometry">Accelerometry</option>
			<option value="Pressure">Pressure</option>
		</select>
	</article>
	<footer class="flex justify-end gap-4">
		<button type="button" class="btn preset-tonal" onclick={modalClose}
			>Cancel</button
		>
		{#if details.state !== "SysTx"}
			<button
				type="button"
				class="btn preset-filled"
				onclick={async () => await start_recording()}
				>Start Recording</button
			>
		{:else}
			<button
				type="button"
				class="btn preset-filled"
				onclick={async () => await stop_recording()}
				>Stop Recording</button
			>
		{/if}
	</footer>
{/if}
