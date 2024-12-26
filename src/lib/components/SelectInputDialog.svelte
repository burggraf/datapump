<script lang="ts">
	import * as Dialog from "$lib/components/ui/dialog";
	import * as Table from "$lib/components/ui/table";
	import { Button } from "$lib/components/ui/button";
	import { Plus, X } from "lucide-svelte";
	let { open = $bindable(false) } = $props();
	let showNewInputDiv = $state(false);
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
	const handleClick = (source: any) => {
		console.log("source:", source);
		open = false;
	};
</script>

<Dialog.Root {open} onOpenChange={(e) => (open = e)}>
	<Dialog.Portal>
		<Dialog.Content class="content mt-0 max-h-[50vh] gap-0 overflow-y-scroll pl-0 pr-0 pt-0">
			<Dialog.Header class="sticky top-0 z-10 bg-background pl-2 pr-2">
				<table class="w-full">
					<tbody>
						<tr>
							<td class="w-1/6">
								<Dialog.Close
									><Button variant="ghost" class="p-2">
										<X />
									</Button></Dialog.Close
								>
							</td>
							<td class="w-4/6 text-center">Select Input</td>
							<td class="w-1/6 text-right">
								<Button
									variant="ghost"
									class="p-2"
									onclick={() => {
										showNewInputDiv = true;
									}}
								>
									<Plus />
								</Button>
							</td>
						</tr>
					</tbody>
				</table>
			</Dialog.Header>
			{#if showNewInputDiv}
				<div
					class="fixed left-0 top-10 z-10 flex h-full w-full items-center justify-center bg-background"
				>
					<div class="text-xl">create new input source</div>
				</div>
			{/if}
			<Table.Root class="w-full">
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
