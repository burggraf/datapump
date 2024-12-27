<script lang="ts">
	import Header from "$lib/components/Header.svelte";
	import InputSourceCard from "$lib/components/InputSourceCard.svelte";
	import OutputSourceCard from "$lib/components/OutputSourceCard.svelte";

	let sourcePath = $state("");
	let outputConnectionString = $state("");
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
		const storedOutputConnectionString = localStorage.getItem("sourceConnection");
		if (storedOutputConnectionString) {
			outputConnectionString = JSON.parse(storedOutputConnectionString);
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
		localStorage.setItem("outputConnectionString", JSON.stringify(outputConnectionString));
		localStorage.setItem("inputSources", JSON.stringify(sources));
	});
</script>

<Header />

<div class="flex gap-4 p-4">
	<InputSourceCard bind:sourcePath bind:schema bind:fileError bind:selectedSource />

	<OutputSourceCard bind:outputConnectionString />
</div>
