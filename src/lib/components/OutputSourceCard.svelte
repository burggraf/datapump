<script lang="ts">
	import { Button } from "$lib/components/ui/button";
	import { Input } from "$lib/components/ui/input";
	import * as Card from "$lib/components/ui/card";
	import { toast } from "svelte-sonner";
	import { executePostgresQuery } from "$lib/services/postgres.svelte";
	import { executeSqliteQuery } from "$lib/services/sqlite.svelte";

	let { outputConnectionString = $bindable("") } = $props();
	let isConnectionStringEmpty = $derived(outputConnectionString === "");

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

	function handleCredentialsChange(newCredentials: any) {
		outputConnectionString = newCredentials;
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
				value={outputConnectionString}
				oninput={(event) => handleCredentialsChange((event.target as HTMLInputElement).value)}
			/>
		</div>
		<p>Card Content</p>
		<Button
			disabled={isConnectionStringEmpty}
			onclick={() => {
				testConnectionString(outputConnectionString);
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
