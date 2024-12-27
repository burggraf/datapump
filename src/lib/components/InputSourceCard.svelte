<script lang="ts">
	import { Button } from "$lib/components/ui/button";
	import { Input } from "$lib/components/ui/input";
	import * as Card from "$lib/components/ui/card";
	import { analyzeSchema } from "$lib/services/flat.svelte";
	import { invoke } from "@tauri-apps/api/core";

	let {
		sourcePath = $bindable(""),
		schema = $bindable<{ name: string; type: string }[]>([]),
		fileError = $bindable(""),
		selectedSource = $bindable<any>(null)
	} = $props();

	async function handleFileChange(newFilePath: string) {
		const realPath = await invoke("get_real_path", { filePath: newFilePath });
		sourcePath = realPath as string;
	}

	$effect.pre(() => {
		const storedSourcePath = localStorage.getItem("sourcePath");
		if (storedSourcePath) {
			sourcePath = storedSourcePath;
		}
	});

	$effect(() => {
		localStorage.setItem("sourcePath", sourcePath);
	});
</script>

<Card.Root class="mb-4">
	<Card.Header>
		<Card.Title>Input</Card.Title>
		<Card.Description>Select or create an input source</Card.Description>
	</Card.Header>
	<Card.Content>
		<div class="mt-4">
			<input
				type="file"
				id="fileInput"
				style="display: none"
				onchange={async (event) => {
					const fileInput = event.target as HTMLInputElement;
					const file = fileInput.files?.[0];
					if (file) {
						// Validate file type
						if (!["text/csv", "text/tab-separated-values"].includes(file.type)) {
							fileError = "Invalid file type. Please upload a CSV or TSV file.";
							return;
						}

						// Analyze schema
						try {
							schema = await analyzeSchema(file);
							fileError = "";
							handleFileChange(file.name);
						} catch (error) {
							fileError = "Failed to parse file. Please check the format.";
							schema = [];
						}
					}
				}}
			/>
			<Button variant="outline" onclick={() => document.getElementById("fileInput")?.click()}
				>Choose File</Button
			>
			<div class="mt-2">
				<label for="filePath" class="block text-sm font-medium text-gray-700">File Path</label>
				<Input
					type="text"
					id="filePath"
					value={sourcePath}
					class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
				/>
			</div>
		</div>
		{#if selectedSource}
			<div class="mt-4">
				<p class="font-bold">{selectedSource.title}</p>
				<p class="text-sm text-gray-500">{selectedSource.description}</p>
			</div>
		{/if}
	</Card.Content>
	<Card.Footer class="flex flex-col gap-2">
		{#if fileError}
			<p class="text-sm text-red-500">{fileError}</p>
		{/if}
		{#if schema.length > 0}
			<div class="text-sm">
				<p class="mb-1 font-medium">Detected Schema:</p>
				<ul class="space-y-1">
					{#each schema as field}
						<li class="flex gap-2">
							<span class="font-medium">{field.name}</span>
							<span class="text-gray-500">({field.type})</span>
						</li>
					{/each}
				</ul>
			</div>
		{/if}
	</Card.Footer>
</Card.Root>
