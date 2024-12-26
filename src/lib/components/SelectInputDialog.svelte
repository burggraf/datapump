<script lang="ts">
	import * as Dialog from "$lib/components/ui/dialog";
	let { open = $bindable(false) } = $props();
	$effect(() => {
		console.log("SelectInputDialog open:", open);
	});
</script>

<Dialog.Root {open} onOpenChange={(e) => (open = e.detail)}>
	<Dialog.Portal>
		<Dialog.Content>
			<Dialog.Header>
				<Dialog.Title>Select Input</Dialog.Title>
			</Dialog.Header>
		</Dialog.Content>
	</Dialog.Portal>
</Dialog.Root>
