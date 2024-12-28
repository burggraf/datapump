<script lang="ts">
	import { Button } from "$lib/components/ui/button";
	import * as Card from "$lib/components/ui/card";
	import { migrate } from "$lib/services/migrateFileSqlite.svelte";
	let { sourcePath, outputConnectionString } = $props<{
		sourcePath: string;
		outputConnectionString: string;
	}>();
	const startMigration = async () => {
		await migrate(sourcePath, outputConnectionString);
	};
</script>

<Card.Root class="h-full">
	<Card.Header>
		<Card.Title>Migration</Card.Title>
		<Card.Description>Migration status</Card.Description>
	</Card.Header>
	<Card.Content>
		<Button onclick={startMigration}>Start</Button>
	</Card.Content>
</Card.Root>
