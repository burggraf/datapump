<script lang="ts">
	import { toast } from "svelte-sonner";
	import { Button } from "$lib/components/ui/button";
	import { Input } from "$lib/components/ui/input";
	import { Settings } from "lucide-svelte";
	import * as Card from "$lib/components/ui/card";
	import { executePostgresQuery } from "$lib/services/postgres.svelte";
	import { executeSqliteQuery } from "$lib/services/sqlite.svelte";
	import { invoke } from "@tauri-apps/api/core";
	import { analyzeSchema } from "$lib/services/flat.svelte";

	let dialogOpen = $state(false);
	let sourceType = $state<"File" | "Remote Database">("File");
	let sourcePath = $state("");
	let sourceConnection = $state("");
	let isConnectionStringEmpty = $derived(sourceConnection === "");
	$effect.pre(() => {
		const storedSourcePath = localStorage.getItem("sourcePath");
		if (storedSourcePath) {
			sourcePath = storedSourcePath;
		}
		const storedSourceConnection = localStorage.getItem("sourceConnection");
		if (storedSourceConnection) {
			sourceConnection = JSON.parse(storedSourceConnection);
		}
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
		localStorage.setItem("sourcePath", sourcePath);
		localStorage.setItem("sourceConnection", JSON.stringify(sourceConnection));
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

	async function handleFileChange(newFilePath: string) {
		const realPath = await invoke("get_real_path", { filePath: newFilePath });
		sourcePath = realPath as string;
	}

	function handleCredentialsChange(newCredentials: any) {
		sourceConnection = newCredentials;
	}
	async function testConnectionString(connectionString: string) {
		const { data, error } = await executePostgresQuery(connectionString, "SELECT 1");
		if (error) {
			toast.error(JSON.stringify(error));
			return;
		} else {
			toast.success("Connection successful");
		}
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
					onchange={async (event) => {
						const fileInput = event.target as HTMLInputElement;
						const file = fileInput.files?.[0];
						if (file) {
							// Validate file type
							if (!["text/csv", "text/tab-separated-values"].includes(file.type)) {
								fileError = "Invalid file type. Please upload a CSV or TSV file.";
								return;
							}

							// Analyze schema
							try {
								schema = await analyzeSchema(file);
								fileError = "";
								handleFileChange(file.name);
							} catch (error) {
								fileError = "Failed to parse file. Please check the format.";
								schema = [];
							}
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
		<Card.Footer class="flex flex-col gap-2">
			{#if fileError}
				<p class="text-sm text-red-500">{fileError}</p>
			{/if}
			{#if schema.length > 0}
				<div class="text-sm">
					<p class="mb-1 font-medium">Detected Schema:</p>
					<ul class="space-y-1">
						{#each schema as field}
							<li class="flex gap-2">
								<span class="font-medium">{field.name}</span>
								<span class="text-gray-500">({field.type})</span>
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
			<div class="mb-4">
				<label for="connectionString" class="block text-sm font-medium text-gray-700"
					>Connection String</label
				>
				<Input
					type="text"
					id="connectionString"
					class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
					value={sourceConnection}
					oninput={(event) => handleCredentialsChange((event.target as HTMLInputElement).value)}
				/>
			</div>
			<p>Card Content</p>
			<Button
				disabled={isConnectionStringEmpty}
				onclick={() => {
					testConnectionString(sourceConnection);
				}}>Test connection string</Button
			>
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
