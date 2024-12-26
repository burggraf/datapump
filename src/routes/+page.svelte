<script lang="ts">
	import { toast } from "svelte-sonner";
	import { Button } from "$lib/components/ui/button";
	import { Settings } from "lucide-svelte";
	import * as Card from "$lib/components/ui/card";
	import { executePostgresQuery } from "$lib/services/postgres.svelte";
	import * as Dialog from "$lib/components/ui/dialog";
	import SelectInputDialog from "$lib/components/SelectInputDialog.svelte";
	let dialogOpen = $state(false);
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
			<Card.Description>Card Description</Card.Description>
		</Card.Header>
		<Card.Content>
			<Button class="w-full" variant="outline" onclick={() => (dialogOpen = true)}
				>Select Input</Button
			>
		</Card.Content>
		<Card.Footer>
			<p>Card Footer</p>
		</Card.Footer>
	</Card.Root>
	<SelectInputDialog bind:open={dialogOpen} />

	<Card.Root class="w-1/2">
		<Card.Header>
			<Card.Title>Output</Card.Title>
			<Card.Description>Card Description</Card.Description>
		</Card.Header>
		<Card.Content>
			<p>Card Content</p>
			<Button onclick={testPostgres}>test postgres query</Button>
			<div id="output"></div>
		</Card.Content>
		<Card.Footer>
			<p>Card Footer</p>
		</Card.Footer>
	</Card.Root>
</div>
