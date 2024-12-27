<script lang="ts">
	import Header from "$lib/components/Header.svelte";
	import InputSourceCard from "$lib/components/InputSourceCard.svelte";
	import OutputSourceCard from "$lib/components/OutputSourceCard.svelte";

	let sourcePath = $state("");
	let sourceConnection = $state("");
	let schema = $state<{ name: string; type: string }[]>([]);
	let fileError = $state("");

	interface Source {
		title: string;
		description: string;
		type: "File" | "Remote Database";
		path: string;
		connection: any;
		selected: boolean;
	}

	let sources = $state<Source[]>([]);
	let selectedSource = $state<Source | null>(null);

	$effect.pre(() => {
		const storedSourcePath = localStorage.getItem("sourcePath");
		if (storedSourcePath) {
			sourcePath = storedSourcePath;
		}
		const storedSourceConnection = localStorage.getItem("sourceConnection");
		if (storedSourceConnection) {
			sourceConnection = JSON.parse(storedSourceConnection);
		}
		const storedSources = localStorage.getItem("inputSources");
		if (storedSources) {
			sources = JSON.parse(storedSources);
		} else {
			sources = [];
		}
	});

	$effect(() => {
		localStorage.setItem("sourcePath", sourcePath);
		localStorage.setItem("sourceConnection", JSON.stringify(sourceConnection));
		localStorage.setItem("inputSources", JSON.stringify(sources));
	});
</script>

<Header />

<div class="flex gap-4 p-4">
	<InputSourceCard bind:sourcePath bind:schema bind:fileError bind:selectedSource />

	<OutputSourceCard bind:sourceConnection />
</div>
