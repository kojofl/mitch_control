<script lang="ts">
	import { Modal } from "@skeletonlabs/skeleton-svelte";
	import { invoke } from "@tauri-apps/api/core";
	import { listen } from "@tauri-apps/api/event";
	import { onMount } from "svelte";
	import MitchModal from "./MitchModal.svelte";

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
	let selected: number | undefined = $state(undefined);

	onMount(async () => {
		mitches = await invoke("get_mitches");
	});

	listen<MitchDiscovered>("mitch-discovered", (event) => {
		mitches.push({
			name: event.payload.name,
			connected: false,
		});
	});
	async function start_recording() {
		try {
			await invoke("start_recording", { id: selected });
			details = await invoke("get_mitch_details", { id: selected });
		} catch (e) {
			console.log(e);
		}
	}
	async function stop_recording() {
		try {
			await invoke("stop_recording", { id: selected });
			details = await invoke("get_mitch_details", { id: selected });
		} catch (e) {
			console.log(e);
		}
	}
	async function get_details(i: number) {
		try {
			details = await invoke("get_mitch_details", { id: i });
			selected = i;
			console.log(selected);
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
								<MitchModal bind:details id={selected} />
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
