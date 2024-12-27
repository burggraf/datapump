<script lang="ts">
	import { Button } from "$lib/components/ui/button";
	import { Input } from "$lib/components/ui/input";
	import * as Card from "$lib/components/ui/card";
	import { toast } from "svelte-sonner";
	import { executePostgresQuery } from "$lib/services/postgres.svelte";
	import { executeSqliteQuery } from "$lib/services/sqlite.svelte";

	let {
		outputConnectionString = $bindable(""),
		ocsType = $bindable(""),
		ocsUser = $bindable(""),
		ocsPassword = $bindable(""),
		ocsHost = $bindable(""),
		ocsPort = $bindable(""),
		ocsDatabase = $bindable("")
	} = $props();
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
		switch (ocsType) {
			case "postgres":
				const { data: pgData, error: pgError } = await executePostgresQuery(
					connectionString,
					"SELECT version()"
				);
				if (pgError) {
					toast.error(JSON.stringify(pgError));
					return;
				} else {
					if (pgData && pgData.rows && pgData.rows[0] && pgData.rows[0][0]) {
						toast.success("Connection successful:" + JSON.stringify(pgData.rows[0][0]));
					} else {
						toast.error("Connection successful:" + JSON.stringify(pgData));
					}
				}
				break;
			case "sqlite":
				const { data: sqliteData, error: sqliteError } = await executeSqliteQuery(
					ocsDatabase,
					"SELECT sqlite_version()"
				);
				if (sqliteError) {
					toast.error(JSON.stringify(sqliteError));
					return;
				} else {
					if (sqliteData && sqliteData.rows && sqliteData.rows[0] && sqliteData.rows[0][0]) {
						toast.success(
							"Connection successful:" + " version " + JSON.stringify(sqliteData.rows[0][0])
						);
					} else {
						toast.success("Connection successful:" + JSON.stringify(sqliteData));
					}
				}
				break;
			default:
				toast.error("Unsupported connection type");
		}
	}
</script>

<Card.Root>
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
		<Button
			disabled={isConnectionStringEmpty}
			onclick={() => {
				testConnectionString(outputConnectionString);
			}}>Test connection string</Button
		>
		<pre id="output"></pre>
	</Card.Content>
	<Card.Footer>
		<p>
			Type: {ocsType}<br /> User: {ocsUser}<br /> Password: {ocsPassword ? "*********" : ""} <br />
			Host: {ocsHost}<br /> Port: {ocsPort}<br />
			Database: {ocsDatabase}
		</p>
	</Card.Footer>
</Card.Root>
