<script lang="ts">
	import { Button } from "$lib/components/ui/button";
	import { Input } from "$lib/components/ui/input";
	import ChooseDatabase from "./ChooseDatabase.svelte";
	import * as Card from "$lib/components/ui/card";
	import SourceTypeSelect from "./SourceTypeSelect.svelte";
	import MigrationCard from "./MigrationCardCode.svelte";

	const migrationCard = new MigrationCard();
</script>

<Card.Root class="h-full">
	<Card.Header>
		<Card.Title>Migration</Card.Title>
		<Card.Description>Migration status</Card.Description>
	</Card.Header>
	<Card.Content class="gap-2">
		<div class="block w-full">
			<div class="mt-1 flex items-center">
				<label for="filePath" class="mr-2 w-32 text-sm font-medium text-gray-700">Source:</label>
				<SourceTypeSelect bind:selectedValue={migrationCard.sourceType} class="ml-2 mr-2" />
				<Input
					type="text"
					id="filePath"
					value={migrationCard.sourcePath}
					autocomplete="off"
					autocapitalize="off"
					spellcheck="false"
					autocorrect="off"
					class="block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
				/>
				{#if migrationCard.sourceType === "csv_tsv" || migrationCard.sourceType === "sqlite"}
					<Button
						id="chooseSourceFile"
						class="ml-2"
						onclick={async () => {
							try {
								const path = await migrationCard.invoke<string>("open_file_dialog", {});
								migrationCard.sourcePath = path;
							} catch (error) {
								console.error("Error selecting file:", error);
							}
						}}
					>
						Choose File
					</Button>
				{/if}
				{#if migrationCard.sourceType !== "csv_tsv" && migrationCard.sourceType !== "sqlite"}
					<ChooseDatabase
						bind:user={migrationCard.sourceUser}
						bind:password={migrationCard.sourcePassword}
						bind:host={migrationCard.sourceHost}
						bind:port={migrationCard.sourcePort}
						bind:databaseName={migrationCard.sourceDatabaseName}
					/>
				{/if}
			</div>
			<div class="mt-4 flex items-center">
				<label for="destinationPath" class="mr-2 w-32 text-sm font-medium text-gray-700"
					>Output:</label
				>
				<SourceTypeSelect bind:selectedValue={migrationCard.destinationType} class="ml-2 mr-2" />
				<Input
					type="text"
					id="destinationPath"
					bind:value={migrationCard.destinationPath}
					placeholder="Enter output path"
					class="block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
					autocomplete="off"
					autocapitalize="off"
					spellcheck="false"
					autocorrect="off"
				/>
				{#if migrationCard.destinationType === "csv_tsv" || migrationCard.destinationType === "sqlite"}
					<Button
						id="chooseDestinationFile"
						class="ml-2"
						onclick={async () => {
							try {
								const path = await migrationCard.invoke<string>("open_file_dialog", {});
								migrationCard.destinationPath = path;
							} catch (error) {
								console.error("Error selecting file:", error);
							}
						}}
					>
						Choose File
					</Button>
				{/if}
				{#if migrationCard.destinationType !== "csv_tsv" && migrationCard.destinationType !== "sqlite"}
					<ChooseDatabase
						bind:user={migrationCard.destinationUser}
						bind:password={migrationCard.destinationPassword}
						bind:host={migrationCard.destinationHost}
						bind:port={migrationCard.destinationPort}
						bind:databaseName={migrationCard.destinationDatabaseName}
					/>
				{/if}
			</div>
			<div class="mt-4 flex items-center">
				<label for="tableName" class="mr-2 w-32 text-sm font-medium text-gray-700">Table:</label>
				<Input
					type="text"
					id="tableName"
					bind:value={migrationCard.tableName}
					placeholder="Enter table name"
					class="block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
					autocomplete="off"
					autocapitalize="off"
					spellcheck="false"
					autocorrect="off"
				/>
			</div>
		</div>
		<br />
		{#if !migrationCard.migrationInProgress}
			<Button onclick={() => migrationCard.startMigration()}>Start Migration</Button>
		{/if}
		{#if migrationCard.migrationInProgress}
			<Button
				onclick={() => migrationCard.cancelMigration()}
				disabled={migrationCard.status !== "processing"}
				class="bg-red-500 hover:bg-red-600"
			>
				Cancel Migration
			</Button>
		{/if}
		<div class="mt-4 rounded border p-4">
			<h3 class="mb-2 text-lg font-semibold">Status</h3>
			<div class="grid grid-cols-[25%_75%] gap-2">
				<div>Status:</div>
				<div>{migrationCard.status}</div>
				<div>Total Rows:</div>
				<div>{migrationCard.totalRows}</div>
				<div>Processed:</div>
				<div>{migrationCard.processedRows}</div>
				<div>Pct. Completed:</div>
				<div>
					{migrationCard.totalRows > 0
						? Math.round((migrationCard.processedRows / migrationCard.totalRows) * 100)
						: 0}%
				</div>
				<div>Rows per Second:</div>
				<div>{migrationCard.rows_per_second}</div>
				<div>Est. Time Remaining:</div>
				<div>{migrationCard.timeRemainingDisplay}</div>
			</div>
		</div>
	</Card.Content>
</Card.Root>
