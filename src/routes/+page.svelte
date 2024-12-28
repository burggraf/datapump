<script lang="ts">
	import Header from "$lib/components/Header.svelte";
	import MigrationCard from "$lib/components/MigrationCard.svelte";
	import InputSourceCard from "$lib/components/InputSourceCard.svelte";
	import OutputSourceCard from "$lib/components/OutputSourceCard.svelte";

	let sourcePath = $state("");
	let outputConnectionString = $state("");
	let ocsType = $derived(() => {
		const match = outputConnectionString.match(/^([^:]+):\/\//);
		return match ? match[1] : "";
	});
	let ocsUser = $derived(() => {
		if (ocsType() === "sqlite") {
			return "";
		}
		const match = outputConnectionString.match(/:\/\/([^:]+):([^@]+)@/);
		return match ? match[1] : "";
	});
	let ocsPassword = $derived(() => {
		if (ocsType() === "sqlite") {
			return "";
		}
		const match = outputConnectionString.match(/:\/\/([^:]+):([^@]+)@/);
		return match ? match[2] : "";
	});
	let ocsHost = $derived(() => {
		if (ocsType() === "sqlite") {
			return "";
		}
		const match = outputConnectionString.match(/@([^:]+):/);
		return match ? match[1] : "";
	});
	let ocsPort = $derived(() => {
		if (ocsType() === "sqlite") {
			return "";
		}
		const match = outputConnectionString.match(/@.+:(\d+)\//);
		return match ? match[1] : "";
	});
	let ocsDatabase = $derived(() => {
		if (ocsType() === "sqlite") {
			const match = outputConnectionString.match(/^sqlite:\/\/(.+)$/);
			return match ? match[1] : "";
		}
		const match = outputConnectionString.match(/\/([^/]+)$/);
		return match ? match[1] : "";
	});

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
	let selectedSource = $state<File | null>(null);

	$effect.pre(() => {
		const storedSourcePath = localStorage.getItem("sourcePath");
		if (storedSourcePath) {
			sourcePath = storedSourcePath;
		}
		const storedOutputConnectionString = localStorage.getItem("outputConnectionString");
		if (storedOutputConnectionString) {
			outputConnectionString = storedOutputConnectionString;
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
		localStorage.setItem("outputConnectionString", outputConnectionString);
		localStorage.setItem("inputSources", JSON.stringify(sources));
	});
</script>

<Header />

<div class="flex gap-4 p-4">
	<div style="width: 50%;">
		<InputSourceCard bind:sourcePath bind:schema bind:fileError bind:selectedSource />
		<OutputSourceCard
			bind:outputConnectionString
			ocsType={ocsType()}
			ocsUser={ocsUser()}
			ocsPassword={ocsPassword()}
			ocsHost={ocsHost()}
			ocsPort={ocsPort()}
			ocsDatabase={ocsDatabase()}
		/>
	</div>
	<div style="width: 50%;">
		<MigrationCard {selectedSource} {outputConnectionString} />
	</div>
</div>
