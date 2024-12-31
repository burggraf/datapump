<script lang="ts">
	import Header from "$lib/components/Header.svelte";
	import MigrationCard from "$lib/components/MigrationCard.svelte";

	let outputConnectionString = $state("");
</script>

<Header />
<!--<div class="flex gap-4 p-4">-->

<div class="p-4">
	<!--
	<div style="width: 50%;">
		<InputSourceCard bind:sourcePath bind:schema bind:fileError bind:selectedSource />
		<OutputSourceCard
			bind:outputConnectionString
			ocsType={ocsType()}
			ocsUser={ocsUser()}
			ocsPassword={ocsPassword()}
			ocsHost={ocsHost()}
			ocsPort={ocsPort()}
			ocsDatabase={ocsDatabase()}
	</div>
	<div style="width: 50%;">
			/>-->

	<MigrationCard />
	<!--</div>-->
</div>
