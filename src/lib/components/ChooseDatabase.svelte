<script lang="ts">
	import * as Dialog from "$lib/components/ui/dialog/index.js";
	import { Button } from "$lib/components/ui/button/index.js";
	import { Input } from "$lib/components/ui/input/index.js";
	import { Label } from "$lib/components/ui/label/index.js";

	let {
		user = $bindable(""),
		password = $bindable(""),
		host = $bindable(""),
		port = $bindable(""),
		databaseName = $bindable(""),
		sourceType = $bindable(""),
		destinationType = $bindable(""),
		sourcePath = $bindable(""),
		destinationPath = $bindable("")
	} = $props();

	let showDialog = $state(false);

	function formatConnectionString(
		type: string,
		user: string,
		password: string,
		host: string,
		port: string,
		databaseName: string
	): string {
		if (type === "postgres") {
			return `postgres://${user}:${password}@${host}:${port}/${databaseName}`;
		}
		return "";
	}
</script>

<Dialog.Root bind:open={showDialog}>
	<Dialog.Trigger>
		<Button class="ml-2">Choose DB</Button>
	</Dialog.Trigger>
	<Dialog.Content class="sm:max-w-[425px]">
		<Dialog.Header>
			<Dialog.Title>Choose Database</Dialog.Title>
			<Dialog.Description>Select the database you want to migrate from.</Dialog.Description>
		</Dialog.Header>
		<div class="grid gap-4 py-4">
			<div class="grid gap-2">
				<Label for="user">User</Label>
				<Input id="user" type="text" bind:value={user} />
			</div>
			<div class="grid gap-2">
				<Label for="password">Password</Label>
				<Input id="password" type="password" bind:value={password} />
			</div>
			<div class="grid gap-2">
				<Label for="host">Host</Label>
				<Input id="host" type="text" bind:value={host} />
			</div>
			<div class="grid gap-2">
				<Label for="port">Port</Label>
				<Input id="port" type="number" bind:value={port} />
			</div>
			<div class="grid gap-2">
				<Label for="databaseName">Database Name</Label>
				<Input id="databaseName" type="text" bind:value={databaseName} />
			</div>
		</div>
		<Dialog.Footer>
			<Button
				type="submit"
				onclick={() => {
					if (sourceType) {
						sourcePath = formatConnectionString(
							sourceType,
							user,
							password,
							host,
							port,
							databaseName
						);
					}
					if (destinationType) {
						destinationPath = formatConnectionString(
							destinationType,
							user,
							password,
							host,
							port,
							databaseName
						);
					}
					showDialog = false;
				}}>Save</Button
			>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
