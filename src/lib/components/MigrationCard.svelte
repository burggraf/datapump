<script lang="ts">
	import { Button } from "$lib/components/ui/button";
	import { Input } from "$lib/components/ui/input";
	import * as Card from "$lib/components/ui/card";
	import { migrate } from "$lib/services/migrateFileSqlite.svelte";
	import { invoke } from "@tauri-apps/api/core";
	import { listen } from "@tauri-apps/api/event";

	let { selectedSource, outputConnectionString } = $props<{
		selectedSource: File | null;
		outputConnectionString: string;
	}>();

	let totalRows = $state(0);
	let batchSize = $state(0);
	let status = $state("idle");

	interface ProgressEvent {
		total_rows: number;
		batch_size: number;
		status: string;
	}
	let sourcePath = $state("/Users/markb/dev/boxball/retrosheet_event.tsv");
	const startMigration = async () => {
		console.log("selectedSource", selectedSource);
		if (selectedSource) {
			await migrate(selectedSource, outputConnectionString);
		}
	};
	const appendToFile = async () => {
		await invoke("append_to_file", { filePath: "test_append.txt", text: "hello world\n" });
	};
	const test = async () => {
		const ts = +new Date();
		// Setup event listener
		const unlisten = await listen<ProgressEvent>("migration_progress", (event) => {
			console.log("Progress update:", event.payload);
			totalRows = event.payload.total_rows;
			batchSize = event.payload.batch_size;
			status = event.payload.status;
			console.log("time elapsed", (+new Date() - ts) / 1000, "seconds");
			console.log("records per second", event.payload.total_rows / ((+new Date() - ts) / 1000));
		});

		try {
			const schema = await invoke("get_csv_schema", { filePath: sourcePath });
			console.log("schema", schema);
			console.log("schema", typeof schema);
			if (typeof schema === "string") {
				console.log("schema", schema.split(","));
			}
			const result = await invoke("csv_to_sqlite", {
				filePath: sourcePath,
				batchSize: 100000,
				schema: schema,
				dbPath: "/Users/markb/Downloads/retrosheet_event_02.db"
			});
			console.log("result", result);
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
			<Input
				type="text"
				id="filePath"
				value={sourcePath}
				class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
			/>
		</div>
		<br />
		<Button onclick={test}>test</Button>
		<Button onclick={startMigration}>Start</Button>
		<Button onclick={appendToFile}>Append to File</Button>
	</Card.Content>
</Card.Root>
