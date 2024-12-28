<script lang="ts">
	import { Button } from "$lib/components/ui/button";
	import * as Card from "$lib/components/ui/card";
	import { migrate } from "$lib/services/migrateFileSqlite.svelte";
	import { invoke } from "@tauri-apps/api/core";
	let { sourcePath, outputConnectionString } = $props<{
		sourcePath: string;
		outputConnectionString: string;
	}>();
	const startMigration = async () => {
		await migrate(sourcePath, outputConnectionString);
	};
	const appendToFile = async () => {
		await invoke("append_to_file", { filePath: "test_append.txt", text: "hello world\n" });
	};
</script>

<Card.Root class="h-full">
	<Card.Header>
		<Card.Title>Migration</Card.Title>
		<Card.Description>Migration status</Card.Description>
	</Card.Header>
	<Card.Content>
		<Button onclick={startMigration}>Start</Button>
		<Button onclick={appendToFile}>Append to File</Button>
	</Card.Content>
</Card.Root>
