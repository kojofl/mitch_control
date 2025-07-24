<script lang="ts">
	import { Modal } from "@skeletonlabs/skeleton-svelte";
	import { invoke } from "@tauri-apps/api/core";
	import { listen } from "@tauri-apps/api/event";
	import { onMount } from "svelte";

	type MitchDiscovered = {
		name: string;
	};

	interface Mitch {
		name: string;
		connected: boolean;
	}

	let mitches: Mitch[] = $state([]);
	let details: any = $state();
	let loading = $state(false);

	onMount(async () => {
		mitches = await invoke("get_mitches");
	});

	listen<MitchDiscovered>("mitch-discovered", (event) => {
		mitches.push({
			name: event.payload.name,
			connected: false,
		});
	});
	async function start_recording(i: number) {
		try {
			await invoke("start_recording", { id: i });
			details = await invoke("get_mitch_details", { id: i });
		} catch (e) {
			console.log(e);
		}
	}
	async function stop_recording(i: number) {
		try {
			await invoke("stop_recording", { id: i });
			details = await invoke("get_mitch_details", { id: i });
		} catch (e) {
			console.log(e);
		}
	}
	async function get_details(i: number) {
		try {
			details = await invoke("get_mitch_details", { id: i });
		} catch (e) {
			console.log(e);
		}
	}

	async function connect(i: number) {
		try {
			loading = true;
			await invoke("connect", { id: i });
			mitches[i].connected = true;
		} catch (e) {
			console.log(e);
		}
		loading = false;
	}

	async function disconnect(i: number) {
		try {
			await invoke("disconnect", { id: i });
			mitches[i].connected = false;
		} catch (e) {
			console.log(e);
		}
	}

	function modalClose() {
		details = undefined;
	}
</script>

{#if loading}
	<div
		id="loading-overlay"
		class="fixed inset-0 bg-black/50 z-50 flex items-center justify-center"
	>
		<div
			class="w-12 h-12 rounded-full animate-spin border-4 border-solid border-white/30 border-t-white"
		></div>
	</div>
{/if}

<main class="container flex m-auto items-center flex-col">
	<div class="w-full grid grid-cols-2 gap-4 mt-4">
		{#each mitches as mitch, i}
			<!-- svelte-ignore a11y_invalid_attribute -->
			<a
				href="#"
				class="card preset-filled-surface-100-900 border-[1px] border-surface-200-800 card-hover divide-surface-200-800 block max-w-md divide-y overflow-hidden"
			>
				<header class="space-y-4 p-4">
					<p class="h3">{mitch.name}</p>
				</header>
				<article class="space-y-4 p-4">
					{#if mitch.connected}
						<p>Connected</p>
					{:else}
						<p>Not Connected</p>
					{/if}
				</article>
				<footer class="flex items-center justify-between gap-4 p-4">
					{#if mitch.connected}
						<button
							type="button"
							class="btn preset-filled-error-500"
							onclick={async () => await disconnect(i)}
							>Disconnect</button
						>
						<button
							type="button"
							class="btn preset-filled-primary-500"
							onclick={async () => await get_details(i)}
							>Open Controls</button
						>
						<Modal
							open={details}
							onOpenChange={(e) => {
								if (!e.open) {
									details = undefined;
								}
							}}
							triggerBase="btn preset-tonal"
							contentBase="card bg-surface-100-900 p-4 space-y-4 shadow-xl max-w-screen-sm"
							backdropClasses="backdrop-blur-sm"
						>
							{#snippet content()}
								{#if details}
									<header class="flex justify-between">
										<h2 class="h2">{details.name}</h2>
									</header>
									<article>
										<p class="opacity-60">
											{details.state}
										</p>
									</article>
									<footer class="flex justify-end gap-4">
										<button
											type="button"
											class="btn preset-tonal"
											onclick={modalClose}>Cancel</button
										>
										{#if details.state !== "SysTx"}
											<button
												type="button"
												class="btn preset-filled"
												onclick={async () =>
													await start_recording(i)}
												>Start Recording</button
											>
										{:else}
											<button
												type="button"
												class="btn preset-filled"
												onclick={async () =>
													await stop_recording(i)}
												>Stop Recording</button
											>
										{/if}
									</footer>
								{/if}
							{/snippet}
						</Modal>
					{:else}
						<button
							type="button"
							class="btn preset-filled-primary-500"
							onclick={async () => await connect(i)}
							>Connect</button
						>
					{/if}
				</footer>
			</a>
		{/each}
	</div>
</main>
