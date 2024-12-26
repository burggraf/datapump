<script lang="ts">
	import * as Dialog from "$lib/components/ui/dialog";
	import * as Table from "$lib/components/ui/table";
	let { open = $bindable(false) } = $props();
	$effect(() => {
		console.log("SelectInputDialog open:", open);
	});
	const inputSources = [
		{
			name: "Camera",
			description: "Use your device's camera as an input source."
		},
		{
			name: "Microphone",
			description: "Use your device's microphone as an input source."
		},
		{
			name: "Screen Share",
			description: "Share your screen as an input source."
		},
		{
			name: "Camera",
			description: "Use your device's camera as an input source."
		},
		{
			name: "Microphone",
			description: "Use your device's microphone as an input source."
		},
		{
			name: "Screen Share",
			description: "Share your screen as an input source."
		},
		{
			name: "Camera",
			description: "Use your device's camera as an input source."
		},
		{
			name: "Microphone",
			description: "Use your device's microphone as an input source."
		},
		{
			name: "Screen Share",
			description: "Share your screen as an input source."
		},
		{
			name: "Camera",
			description: "Use your device's camera as an input source."
		},
		{
			name: "Microphone",
			description: "Use your device's microphone as an input source."
		},
		{
			name: "Screen Share",
			description: "Share your screen as an input source."
		}
	];
</script>

<Dialog.Root {open} onOpenChange={(e) => (open = e)}>
	<Dialog.Portal>
		<Dialog.Content>
			<Dialog.Header>
				<Dialog.Title>Select Input</Dialog.Title>
			</Dialog.Header>
			<Table.Root class="w-full p-4">
				<Table.Body>
					{#each inputSources as source, i (i)}
						<Table.Row>
							<Table.Cell class="font-medium">
								{source.name}
								<div class="text-sm text-muted-foreground">
									{source.description}
								</div>
							</Table.Cell>
						</Table.Row>
					{/each}
				</Table.Body>
			</Table.Root>
		</Dialog.Content>
	</Dialog.Portal>
</Dialog.Root>
