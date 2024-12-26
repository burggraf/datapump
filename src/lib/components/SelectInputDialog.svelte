<script lang="ts">
	import * as Dialog from "$lib/components/ui/dialog";
	import * as Table from "$lib/components/ui/table";
	import { Button } from "$lib/components/ui/button";
	import { Plus, X } from "lucide-svelte";
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
		}
	];
	const handleClick = (source: any) => {
		console.log("source:", source);
		open = false;
	};
</script>

<Dialog.Root {open} onOpenChange={(e) => (open = e)}>
	<Dialog.Portal>
		<Dialog.Content class="max-h-[50vh] overflow-y-auto">
			<Dialog.Header>
				<table class="w-full">
					<tbody>
						<tr>
							<td class="w-1/6">
								<Button onclick={() => (open = false)} variant="ghost" class="p-2">
									<X />
								</Button>
							</td>
							<td class="w-4/6 text-center"> Select Input </td>
							<td class="w-1/6 text-right">
								<Button variant="ghost" class="p-2">
									<Plus />
								</Button>
							</td>
						</tr>
					</tbody>
				</table>
			</Dialog.Header>
			<Table.Root class="w-full p-4">
				<Table.Body>
					{#each inputSources as source, i (i)}
						<Table.Row
							onclick={() => {
								handleClick(source);
							}}
						>
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
