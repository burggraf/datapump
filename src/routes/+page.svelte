<script lang="ts">
	import { toast } from "svelte-sonner";
	import { Button } from "$lib/components/ui/button";
	import { Input } from "$lib/components/ui/input";
	import { Settings } from "lucide-svelte";
	import * as Card from "$lib/components/ui/card";
	import { executePostgresQuery } from "$lib/services/postgres.svelte";
	import { executeSqliteQuery } from "$lib/services/sqlite.svelte";

	let dialogOpen = $state(false);
	function toggleDialog() {
		console.log("toggleDialog called, dialogOpen", dialogOpen);
		dialogOpen = !dialogOpen;
	}
	let editingSourceIndex = $state(-1);
	let sourceType = $state<"File" | "Remote Database">("File");
	let sourcePath = $state("");
	let sourceConnection = $state({});
	let sourceTitle = $state("");
	let sourceDescription = $state("");
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
		const storedSources = localStorage.getItem("inputSources");
		if (storedSources) {
			sources = JSON.parse(storedSources);
		} else {
			sources = [];
		}
	});

	$effect(() => {
		localStorage.setItem("inputSources", JSON.stringify(sources));
	});

	const testPostgres = async () => {
		const { data, error } = await executePostgresQuery(
			"postgres://postgres:postgres@localhost:5432/postgres",
			"SELECT * FROM pg_catalog.pg_tables"
		);
		if (error) {
			toast.error(error.message);
			return;
		} else {
			toast.success("Postgres query successful");
			const outputElement = document.getElementById("output");
			if (outputElement) {
				outputElement.innerText = JSON.stringify(data, null, 2);
			}
		}
	};

	function handleSourceTypeChange(event: any) {
		sourceType = event;
	}

	function handleFileChange(newFilePath: string) {
		sourcePath = newFilePath;
	}

	function handleCredentialsChange(newCredentials: any) {
		sourceConnection = { ...newCredentials };
	}

	function addSource() {
		if (editingSourceIndex > -1) {
			sources = sources.map((source, index) => {
				if (index === editingSourceIndex) {
					return {
						title: sourceTitle,
						description: sourceDescription,
						type: sourceType,
						path: sourcePath,
						connection: sourceConnection,
						selected: source.selected
					};
				}
				return source;
			});
			editingSourceIndex = -1;
		} else {
			let newSource: Source = {
				title: sourceTitle,
				description: sourceDescription,
				type: sourceType,
				path: sourcePath,
				connection: sourceConnection,
				selected: sources.length === 0 ? true : false
			};
			sources = [...sources, newSource];
			if (sources.length === 1) {
				selectedSource = newSource;
			}
		}
		console.log("addSource called");
		dialogOpen = false;
		sourceTitle = "";
		sourceDescription = "";
		sourcePath = "";
		sourceConnection = {};
	}

	function editSource(source: Source) {
		console.log("editSource called", source);
		editingSourceIndex = sources.indexOf(source);
		sourceType = source.type;
		sourcePath = source.path;
		sourceConnection = source.connection;
		sourceTitle = source.title;
		sourceDescription = source.description;
		dialogOpen = true;
	}

	function selectSource(source: Source) {
		sources = sources.map((s) => ({ ...s, selected: s === source ? true : false }));
		selectedSource = source;
	}
</script>

<header class="flex items-center justify-between bg-gray-100 p-4">
	<h1 class="flex-grow text-center text-2xl font-bold">Data Pump</h1>
	<Button variant="ghost" size="icon">
		<Settings class="h-6 w-6" />
	</Button>
</header>

<div class="flex gap-4 p-4">
	<Card.Root class="w-1/2">
		<Card.Header>
			<Card.Title>Input</Card.Title>
			<Card.Description>Select or create an input source</Card.Description>
		</Card.Header>
		<Card.Content>
			<div class="mt-4">
				<input
					type="file"
					id="fileInput"
					style="display: none"
					onchange={(event) => {
						const file = (event.target as HTMLInputElement).files?.[0];
						if (file) {
							handleFileChange(file.name);
						}
					}}
				/>
				<Button variant="outline" onclick={() => document.getElementById("fileInput")?.click()}
					>Choose File</Button
				>
				<div class="mt-2">
					<label for="filePath" class="block text-sm font-medium text-gray-700">File Path</label>
					<Input
						type="text"
						id="filePath"
						value={sourcePath}
						oninput={(event) => handleFileChange((event.target as HTMLInputElement).value)}
						class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
					/>
				</div>
			</div>
			{#if selectedSource}
				<div class="mt-4">
					<p class="font-bold">{selectedSource.title}</p>
					<p class="text-sm text-gray-500">{selectedSource.description}</p>
				</div>
			{/if}
		</Card.Content>
		<Card.Footer>
			<p>Card Footer</p>
		</Card.Footer>
	</Card.Root>

	<Card.Root class="w-1/2">
		<Card.Header>
			<Card.Title>Output</Card.Title>
			<Card.Description>Card Description</Card.Description>
		</Card.Header>
		<Card.Content>
			<p>Card Content</p>
			<Button onclick={testPostgres}>test postgres query</Button>
			<Button
				onclick={async () => {
					const { data, error } = await executeSqliteQuery("test.db", "select 1");
					if (error) {
						toast.error(error.message);
						return;
					} else {
						toast.success("Sqlite query successful");
						const outputElement = document.getElementById("output");
						if (outputElement) {
							outputElement.innerText = JSON.stringify(data, null, 2);
						}
					}
				}}>test sqlite query</Button
			>
			<pre id="output"></pre>
		</Card.Content>
		<Card.Footer>
			<p>Card Footer</p>
		</Card.Footer>
	</Card.Root>
</div>
