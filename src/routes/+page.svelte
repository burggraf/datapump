<script lang="ts">
	import { toast } from "svelte-sonner";
	import { Button } from "$lib/components/ui/button";
	import { Settings } from "lucide-svelte";
	import * as Card from "$lib/components/ui/card";
	import { executePostgresQuery } from "$lib/services/postgres.svelte";
	import { executeSqliteQuery } from "$lib/services/sqlite.svelte";
	import SourceTypeSelector from "$lib/components/SourceTypeSelector.svelte";
	import FileSelector from "$lib/components/FileSelector.svelte";
	import DatabaseCredentials from "$lib/components/DatabaseCredentials.svelte";
	import { writable } from "svelte/store";

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
			<Button class="w-full" variant="outline" onclick={toggleDialog}>Select Input</Button>
			{#if selectedSource}
				<div class="mt-4">
					<p class="font-bold">{selectedSource.title}</p>
					<p class="text-sm text-gray-500">{selectedSource.description}</p>
				</div>
			{/if}
		</Card.Content>
		<Card.Footer>
			<!--<p>Card Footer</p>-->
			{#if sources.length > 0}
				<div class="mt-4 w-full p-4">
					<h2 class="mb-2 text-xl font-bold">Input Sources</h2>
					<ul>
						{#each sources as source}
							<li class="flex w-full items-center border-b p-2">
								<div
									role="button"
									tabindex="0"
									onclick={() => selectSource(source)}
									onkeydown={(event) => {
										if (event.key === "Enter") {
											selectSource(source);
										}
									}}
									class="w-full flex-grow cursor-pointer"
								>
									<p class="font-bold">{source.title}</p>
									<p class="text-sm text-gray-500">{source.description}</p>
								</div>
								{#if source.selected}
									<span>✓</span>
								{/if}
								<Button variant="ghost" size="icon" onclick={() => editSource(source)}>
									<svg
										xmlns="http://www.w3.org/2000/svg"
										width="24"
										height="24"
										viewBox="0 0 24 24"
										fill="none"
										stroke="currentColor"
										stroke-width="2"
										stroke-linecap="round"
										stroke-linejoin="round"
										class="lucide lucide-edit"
										><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" /><path
											d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4z"
										/></svg
									>
								</Button>
							</li>
						{/each}
					</ul>
				</div>
			{/if}
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

{#if dialogOpen}
	<div class="fixed inset-0 z-50 h-full w-full overflow-y-auto bg-gray-600 bg-opacity-50">
		<div class="relative top-20 mx-auto w-96 rounded-md border bg-white p-5 shadow-lg">
			<div class="mt-3 text-center">
				<h3 class="text-lg font-medium leading-6 text-gray-900">Create Input Source</h3>
				<div class="mt-2 px-7 py-3">
					<SourceTypeSelector dispatch={handleSourceTypeChange} />
					{#if sourceType === "File"}
						<FileSelector dispatch={handleFileChange} />
					{/if}
					{#if sourceType === "Remote Database"}
						<DatabaseCredentials dispatch={handleCredentialsChange} />
					{/if}
					<label class="mt-4 block">
						Title:
						<input
							type="text"
							class="w-full border p-2"
							style="color: black !important; background-color: white !important;"
							bind:value={sourceTitle}
						/>
					</label>
					<label class="mt-4 block">
						Description:
						<input
							type="text"
							class="w-full border p-2"
							style="color: black !important; background-color: white !important;"
							bind:value={sourceDescription}
						/>
					</label>
					<input type="hidden" bind:value={editingSourceIndex} />
				</div>
				<div class="items-center px-4 py-3">
					<Button
						class="w-full rounded-md bg-green-500 px-4 py-2 text-base font-medium text-white shadow-sm hover:bg-green-600 focus:outline-none focus:ring-2 focus:ring-green-300"
						onclick={addSource}
					>
						Add Source
					</Button>
					<Button
						class="mt-2 w-full rounded-md bg-gray-200 px-4 py-2 text-base font-medium text-gray-800 shadow-sm hover:bg-gray-300 focus:outline-none focus:ring-2 focus:ring-gray-300"
						onclick={() => {
							console.log("cancel button clicked");
							dialogOpen = false;
						}}
					>
						Cancel
					</Button>
				</div>
			</div>
		</div>
	</div>
{/if}
