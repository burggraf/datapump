<script lang="ts">
	let { dispatch } = $props();

	function handleFileChange(event: Event) {
		const target = event.target as HTMLInputElement;
		if (target.files && target.files.length > 0) {
			const file = target.files[0];
			dispatch(file.name);
		}
	}
</script>

<div>
	<input type="file" onchange={handleFileChange} />
</div>
