<script lang="ts">
	import * as Dialog from "$lib/components/ui/dialog/index.js";
	import { Button } from "$lib/components/ui/button/index.js";
</script>

<Dialog.Root>
	<Dialog.Trigger>
		<Button>Choose DB</Button>
	</Dialog.Trigger>
	<Dialog.Content class="sm:max-w-[425px]">
		<Dialog.Header>
			<Dialog.Title>Choose Database</Dialog.Title>
			<Dialog.Description>Select the database you want to migrate from.</Dialog.Description>
		</Dialog.Header>
		<div class="grid gap-4 py-4"></div>
		<Dialog.Footer>
			<Button type="submit">Save</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
