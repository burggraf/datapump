<script lang="ts">
	import { onMount } from "svelte";
	import { Button } from "$lib/components/ui/button";
	import { Input } from "$lib/components/ui/input";
	import * as Card from "$lib/components/ui/card";
	import { migrate } from "$lib/services/migrateFileSqlite.svelte";
	import { invoke } from "@tauri-apps/api/core";
	import { listen } from "@tauri-apps/api/event";
	import { getCurrentWindow } from "@tauri-apps/api/window";
	import SourceTypeSelect from "./SourceTypeSelect.svelte";

	let totalRows = $state(0);
	let processedRows = $state(0);
	let batchSize = $state(0);
	let message = $state("");
	let status = $state("idle");
	let rows_per_second = $state(0);
	let timeRemainingDisplay = $state("");
	let tableName = $state<string>("");
	let dbPath = $state("");
	let cancellationRequested = $state(false);
	let migrationInProgress = $state(false);
	let sourceType = $state("csv_tsv");
	let outputType = $state("sqlite");
	let sourcePath = $state("");

	$effect(() => {
		const storedSourcePath = localStorage.getItem("sourcePath");
		if (storedSourcePath) {
			sourcePath = storedSourcePath;
		}
		const storedDbPath = localStorage.getItem("dbPath");
		if (storedDbPath) {
			dbPath = storedDbPath;
		}
		const storedTableName = localStorage.getItem("tableName");
		if (storedTableName) {
			tableName = storedTableName;
		}
	});
	$effect(() => {
		localStorage.setItem("sourcePath", sourcePath);
	});
	$effect(() => {
		localStorage.setItem("dbPath", dbPath);
	});
	$effect(() => {
		localStorage.setItem("tableName", tableName);
	});
	$effect(() => {
		tableName = tableNameFromPath();
	});
	let tableNameFromPath = $derived(() => {
		if (!sourcePath) return "";
		const filename = sourcePath.split("/").pop() || "";
		return filename.replace(/\.[^/.]+$/, "");
	});

	interface ProgressEvent {
		processed_rows: number;
		row_count: number;
		total_rows: number;
		batch_size: number;
		status: string;
		message?: string;
	}

	const cancelMigration = async () => {
		cancellationRequested = true;
		status = "cancelling";
		try {
			await invoke("cancel_migration");
			status = "cancelled";
			message = "Migration cancelled by user";
			migrationInProgress = false;
		} catch (error) {
			console.error("Error cancelling migration:", error);
			status = "error";
			message = "Failed to cancel migration";
			migrationInProgress = false;
		}
	};

	const startMigration = async () => {
		// Reset state variables
		totalRows = 0;
		processedRows = 0;
		batchSize = 0;
		message = "";
		status = "idle";
		timeRemainingDisplay = "";
		cancellationRequested = false;
		migrationInProgress = true;

		let ts = +new Date();
		// Setup event listener
		const unlisten = await listen<ProgressEvent>("migration_progress", (event) => {
			if (cancellationRequested) return;

			processedRows = event.payload.processed_rows;
			totalRows = event.payload.total_rows;
			batchSize = event.payload.batch_size;
			status = event.payload.status;
			message = event.payload.message || "";
			const elapsed = (+new Date() - ts) / 1000;
			const rps = processedRows / elapsed;
			rows_per_second = Math.round(processedRows / elapsed);
			// calculate estimated time remaining
			let timeRemaining = (totalRows - processedRows) / rps;
			if (timeRemaining > 0 && isFinite(timeRemaining)) {
				const minutes = Math.floor(timeRemaining / 60);
				const seconds = Math.floor(timeRemaining % 60);
				timeRemainingDisplay = `${minutes}:${seconds < 10 ? "0" : ""}${seconds}`;
			}
			if (status === "parsing_schema_complete" || status === "counted_rows") {
				ts = +new Date();
			}
		});

		try {
			const schema = await invoke("get_csv_schema", { filePath: sourcePath, tableName });

			if (typeof schema !== "string") {
				throw new Error("Invalid schema format: expected string");
			}

			const schemaParts = schema.split(",");

			// Validate schema format
			if (!schemaParts.every((part) => part.includes(":"))) {
				throw new Error("Invalid schema format: each part should be in 'name:type' format");
			}

			const window = getCurrentWindow();
			const result = await invoke("csv_to_sqlite", {
				window,
				filePath: sourcePath,
				batchSize: 50000,
				schema: schema,
				tableName: tableName,
				dbPath
			});
		} catch (error) {
			console.error("Error during CSV to SQLite migration:", error);
			migrationInProgress = false;
			throw error;
		} finally {
			// Clean up event listener
			unlisten();
			migrationInProgress = false;
		}
	};
</script>

<Card.Root class="h-full">
	<Card.Header>
		<Card.Title>Migration</Card.Title>
		<Card.Description>Migration status</Card.Description>
	</Card.Header>
	<Card.Content class="gap-2">
		<div class="block w-full">
			<div class="mt-1 flex items-center">
				<label for="filePath" class="mr-2 w-32 text-sm font-medium text-gray-700">Source:</label>
				<SourceTypeSelect bind:selectedValue={sourceType} class="ml-2 mr-2" />
				<Input
					type="text"
					id="filePath"
					value={sourcePath}
					autocomplete="off"
					autocapitalize="off"
					spellcheck="false"
					autocorrect="off"
					class="block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
				/>
				<Button
					class="ml-2"
					onclick={async () => {
						try {
							const path = await invoke<string>("open_file_dialog", {});
							sourcePath = path;
						} catch (error) {
							console.error("Error selecting file:", error);
						}
					}}
				>
					Choose File
				</Button>
			</div>
			<div class="mt-4 flex items-center">
				<label for="dbPath" class="mr-2 w-32 text-sm font-medium text-gray-700">Output:</label>
				<SourceTypeSelect bind:selectedValue={outputType} class="ml-2 mr-2" />
				<Input
					type="text"
					id="dbPath"
					bind:value={dbPath}
					placeholder="Enter output path"
					class="block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
					autocomplete="off"
					autocapitalize="off"
					spellcheck="false"
					autocorrect="off"
				/>
				<Button
					class="ml-2"
					onclick={async () => {
						try {
							const path = await invoke<string>("open_file_dialog", {});
							dbPath = path;
						} catch (error) {
							console.error("Error selecting file:", error);
						}
					}}
				>
					Choose File
				</Button>
			</div>
			<div class="mt-4 flex items-center">
				<label for="tableName" class="mr-2 w-32 text-sm font-medium text-gray-700">Table:</label>
				<Input
					type="text"
					id="tableName"
					bind:value={tableName}
					placeholder="Enter table name"
					class="block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
					autocomplete="off"
					autocapitalize="off"
					spellcheck="false"
					autocorrect="off"
				/>
			</div>
		</div>
		<br />
		{#if !migrationInProgress}
			<Button onclick={startMigration}>Start Migration</Button>
		{/if}
		{#if migrationInProgress}
			<Button
				onclick={cancelMigration}
				disabled={status !== "processing"}
				class="bg-red-500 hover:bg-red-600"
			>
				Cancel Migration
			</Button>
		{/if}
		<div class="mt-4 rounded border p-4">
			<h3 class="mb-2 text-lg font-semibold">Status</h3>
			<div class="grid grid-cols-2 gap-2">
				<div>Status:</div>
				<div>{status}</div>
				<div>Total Rows:</div>
				<div>{totalRows}</div>
				<div>Processed:</div>
				<div>{processedRows}</div>
				<div>Pct. Completed:</div>
				<div>{totalRows > 0 ? Math.round((processedRows / totalRows) * 100) : 0}%</div>
				<div>Rows per Second:</div>
				<div>{rows_per_second}</div>
				<div>Est. Time Remaining:</div>
				<div>{timeRemainingDisplay}</div>
			</div>
		</div>
	</Card.Content>
</Card.Root>
