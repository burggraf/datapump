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
		sourceConnection = newCredentials;
	}

	function addSource() {
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
		console.log("addSource called");
		dialogOpen = false;
		sourceTitle = "";
		sourceDescription = "";
		sourcePath = "";
		sourceConnection = {};
	}

	function selectSource(source: Source) {
		sources = sources.map((s) => ({ ...s, selected: s === source }));
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
			<Button class="w-full" variant="outline" onclick={() => (dialogOpen = true)}
				>Select Input</Button
			>
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
									onkeydown={() => selectSource(source)}
									class="w-full flex-grow cursor-pointer"
								>
									<p class="font-bold">{source.title}</p>
									<p class="text-sm text-gray-500">{source.description}</p>
								</div>
								{#if source.selected}
									<span>✓</span>
								{/if}
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
