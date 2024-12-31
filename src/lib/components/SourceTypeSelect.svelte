<script lang="ts">
	import * as Select from "$lib/components/ui/select/index.js";

	const sourceTypes = [
		{ value: "csv_tsv", label: "CSV/TSV" },
		{ value: "sqlite", label: "SQLite" },
		{ value: "postgres", label: "Postgres" },
		{ value: "mysql", label: "MySQL" },
		{ value: "sql_server", label: "SQL Server" }
	];

	let { selectedValue = $bindable(""), name = "sourceType", class: className = "" } = $props();

	const triggerContent = $derived(
		sourceTypes.find((t) => t.value === selectedValue)?.label ?? "Select source type"
	);
</script>

<div class={className}>
	<Select.Root type="single" bind:value={selectedValue} {name}>
		<Select.Trigger class="w-[200px]">
			{triggerContent}
		</Select.Trigger>
		<Select.Content>
			<Select.Group>
				<Select.GroupHeading>Source Types</Select.GroupHeading>
				{#each sourceTypes as type}
					<Select.Item value={type.value} label={type.label}>
						{type.label}
					</Select.Item>
				{/each}
			</Select.Group>
		</Select.Content>
	</Select.Root>
</div>
