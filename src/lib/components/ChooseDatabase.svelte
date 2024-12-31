<script lang="ts">
	import * as Dialog from "$lib/components/ui/dialog/index.js";
	import { Button } from "$lib/components/ui/button/index.js";
	import { Input } from "$lib/components/ui/input/index.js";
	import { Label } from "$lib/components/ui/label/index.js";

	let db = $state({
		user: "",
		password: "",
		host: "",
		port: "",
		databaseName: ""
	});
</script>

<Dialog.Root>
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
				<Input id="user" type="text" bind:value={db.user} />
			</div>
			<div class="grid gap-2">
				<Label for="password">Password</Label>
				<Input id="password" type="password" bind:value={db.password} />
			</div>
			<div class="grid gap-2">
				<Label for="host">Host</Label>
				<Input id="host" type="text" bind:value={db.host} />
			</div>
			<div class="grid gap-2">
				<Label for="port">Port</Label>
				<Input id="port" type="number" bind:value={db.port} />
			</div>
			<div class="grid gap-2">
				<Label for="databaseName">Database Name</Label>
				<Input id="databaseName" type="text" bind:value={db.databaseName} />
			</div>
		</div>
		<Dialog.Footer>
			<Button type="submit">Save</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
