<script lang="ts">
	import * as Dialog from "$lib/components/ui/dialog";
	let { open = $bindable() } = $props();
</script>

<Dialog.Root {open}>
	<Dialog.Trigger>
		<button>Open Dialog</button>
	</Dialog.Trigger>
	<Dialog.Portal>
		<Dialog.Content>
			<Dialog.Header>
				<Dialog.Title>Select Input</Dialog.Title>
			</Dialog.Header>
		</Dialog.Content>
	</Dialog.Portal>
</Dialog.Root>
