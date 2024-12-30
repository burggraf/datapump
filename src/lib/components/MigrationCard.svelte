<script lang="ts">
	import { Button } from "$lib/components/ui/button";
	import { Input } from "$lib/components/ui/input";
	import * as Card from "$lib/components/ui/card";
	import { migrate } from "$lib/services/migrateFileSqlite.svelte";
	import { invoke } from "@tauri-apps/api/core";
	import { listen } from "@tauri-apps/api/event";
	import { getCurrentWindow } from "@tauri-apps/api/window";

	let { selectedSource, outputConnectionString } = $props<{
		selectedSource: File | null;
		outputConnectionString: string;
	}>();

	let totalRows = $state(0);
	let processedRows = $state(0);
	let batchSize = $state(0);
	let message = $state("");
	let status = $state("idle");
	let timeRemainingDisplay = $state("");
	let tableName = $state<string>("");
	let cancellationRequested = $state(false);

	$effect(() => {
		console.log("tableName:", tableName);
	});

	interface ProgressEvent {
		processed_rows: number;
		row_count: number;
		total_rows: number;
		batch_size: number;
		status: string;
		message?: string;
	}

	let sourcePath = $state("/Users/markb/dev/boxball/retrosheet_event.tsv");

	const startMigration = async () => {
		cancellationRequested = false;
		console.log("selectedSource", selectedSource);
		if (selectedSource) {
			await migrate(selectedSource, outputConnectionString);
		}
	};

	const cancelMigration = async () => {
		cancellationRequested = true;
		status = "cancelling";
		try {
			await invoke("cancel_migration");
			status = "cancelled";
			message = "Migration cancelled by user";
		} catch (error) {
			console.error("Error cancelling migration:", error);
			status = "error";
			message = "Failed to cancel migration";
		}
	};

	const appendToFile = async () => {
		await invoke("append_to_file", { filePath: "test_append.txt", text: "hello world\n" });
	};

	const test = async () => {
		let ts = +new Date();
		// Setup event listener
		console.log("listening for migration_progress");
		const unlisten = await listen<ProgressEvent>("migration_progress", (event) => {
			if (cancellationRequested) return;

			console.log("Progress update:", event.payload);
			processedRows = event.payload.processed_rows;
			totalRows = event.payload.total_rows;
			batchSize = event.payload.batch_size;
			status = event.payload.status;
			message = event.payload.message || "";
			const elapsed = (+new Date() - ts) / 1000;
			const rps = processedRows / elapsed;
			// calculate estimated time remaining
			let timeRemaining = (totalRows - processedRows) / rps;
			if (timeRemaining > 0 && isFinite(timeRemaining)) {
				const minutes = Math.floor(timeRemaining / 60);
				const seconds = Math.floor(timeRemaining % 60);
				timeRemainingDisplay = `${minutes}:${seconds < 10 ? "0" : ""}${seconds}`;
			}
			// display time remaining in minutes:seconds format
			switch (status) {
				case "parsing_schema_start":
					console.log("parsing schema start");
					if (message) console.log("message:", message);
					break;
				case "parsing_schema_complete":
					console.log("***************************");
					console.log("parsing schema complete");
					console.log(elapsed, "seconds elapsed");
					if (message) console.log("message:", message);
					console.log("***************************");
					ts = +new Date();
					break;
				case "counting_rows":
					console.log("counting rows");
					if (message) console.log("message:", message);
					break;
				case "counted_rows":
					console.log("***************************");
					console.log("counted rows");
					console.log(elapsed, "seconds elapsed");
					if (message) console.log("message:", message);
					console.log("***************************");
					ts = +new Date();
					break;
				case "processing":
					console.log(
						"time remaining",
						Math.floor(timeRemaining / 60),
						"minutes",
						Math.floor(timeRemaining % 60),
						"seconds",
						"(" + Math.round(rps) + " records per second)"
					);
					if (message) console.log("message:", message);
					break;
				case "completed":
					console.log("***************************");
					console.log("completed");
					console.log(
						Math.floor(timeRemaining / 60),
						"minutes",
						Math.floor(timeRemaining % 60),
						"seconds"
					);
					if (message) console.log("message:", message);
					console.log("***************************");
					break;
				default:
					console.log("unknown status:", status);
					if (message) console.log("message:", message);
			}
		});

		try {
			console.log("invoking get_csv_schema");
			const schema = await invoke("get_csv_schema", { filePath: sourcePath, tableName });
			console.log("schema", schema);
			console.log("schema", typeof schema);

			if (typeof schema !== "string") {
				throw new Error("Invalid schema format: expected string");
			}

			const schemaParts = schema.split(",");
			console.log("schema parts:", schemaParts);

			// Validate schema format
			if (!schemaParts.every((part) => part.includes(":"))) {
				throw new Error("Invalid schema format: each part should be in 'name:type' format");
			}

			console.log("invoking csv_to_sqlite");
			const window = getCurrentWindow();
			const result = await invoke("csv_to_sqlite", {
				window,
				filePath: sourcePath,
				batchSize: 50000,
				schema: schema,
				tableName: tableName,
				dbPath: "/Users/markb/Downloads/retrosheet_event_02.db"
			});
			console.log("result", result);
		} catch (error) {
			console.error("Error during CSV to SQLite migration:", error);
			throw error;
		} finally {
			// Clean up event listener
			unlisten();
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
				<label for="filePath" class="mr-2 w-32 text-sm font-medium text-gray-700"
					>Source Path:</label
				>
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
			</div>
			<div class="mt-4 flex items-center">
				<label for="tableName" class="mr-2 w-32 text-sm font-medium text-gray-700"
					>Table Name:</label
				>
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
		<Button onclick={test}>test</Button>
		<Button onclick={startMigration} disabled={status === "processing"}>Start</Button>
		<Button onclick={appendToFile}>Append to File</Button>
		<Button
			onclick={cancelMigration}
			disabled={status !== "processing"}
			class="bg-red-500 hover:bg-red-600"
		>
			Cancel
		</Button>
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
				<div>Est. Time Remaining:</div>
				<div>{timeRemainingDisplay}</div>
			</div>
		</div>
	</Card.Content>
</Card.Root>
