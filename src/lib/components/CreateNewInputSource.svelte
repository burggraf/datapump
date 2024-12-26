<script lang="ts">
	import * as Dialog from "$lib/components/ui/dialog";
	import { Button } from "$lib/components/ui/button";
	import { X } from "lucide-svelte";
	import { createEventDispatcher } from "svelte";
	let { open = $bindable(false) } = $props();
	const dispatch = createEventDispatcher();
</script>

<Dialog.Root
	{open}
	onOpenChange={(e) => {
		open = e;
		if (!open) {
			dispatch("close");
		}
	}}
>
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
							<td class="w-4/6 text-center">Create New Input Source</td>
							<td class="w-1/6"></td>
						</tr>
					</tbody>
				</table>
			</Dialog.Header>
			<div class="p-4">create new input source</div>
		</Dialog.Content>
	</Dialog.Portal>
</Dialog.Root>
