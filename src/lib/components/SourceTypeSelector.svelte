<script lang="ts">
	let { dispatch } = $props();

	let sourceType = $state<"File" | "Remote Database">("File");

	function handleSourceTypeChange(event: Event) {
		const target = event.target as HTMLInputElement;
		sourceType = target.value as "File" | "Remote Database";
		dispatch(sourceType);
	}
</script>

<div>
	<label>
		<input
			type="radio"
			value="File"
			checked={sourceType === "File"}
			onclick={handleSourceTypeChange}
		/>
		File
	</label>
	<label>
		<input
			type="radio"
			value="Remote Database"
			checked={sourceType === "Remote Database"}
			onclick={handleSourceTypeChange}
		/>
		Remote Database
	</label>
</div>
