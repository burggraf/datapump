<script lang="ts">
    import * as Select from '$lib/components/ui/select';
    import FileInputCard from '$lib/components/FileInputCard.svelte';
    import DatabaseInputCard from '$lib/components/DatabaseInputCard.svelte';
    import type { FileType } from './types';
    import LoadFile from './LoadFile.svelte';
    import { Button } from '$lib/components/ui/button';
    import { Dialog, DialogContent, DialogHeader, DialogTitle } from '$lib/components/ui/dialog';

        let inputPresets = $state<string[]>([
        ]);
        let selectedImportType = $state<'flat' | 'spreadsheet' | 'sqlite' | 'database' | null>(null);
    let flatFileContents = $state<string | null>(null);
    let spreadsheetContents = $state<string | null>(null);
    let sqliteContents = $state<string | null>(null);
    let selectedFlatFileType = $state<'csv' | 'tsv' | null>(null);
    let selectedSpreadsheetType = $state<'excel' | 'google-sheets' | null>(null);
    let selectedSqliteType = $state<'db' | 'zip' | null>(null);
    let selectedDatabaseType = $state<'postgres' | 'mysql' | 'sqlserver' | null>(null);
    let databaseConnectionString = $state<string | null>(null);
    let selectedFile = $state<File | null>(null);
    let showModal = $state(false);

    async function importFlatFile() {
        if (!selectedFile) {
            alert('Please select a file');
            return;
        }

        try {
            const fileContents = await selectedFile.text();
            flatFileContents = fileContents;
            console.log('File contents:', fileContents);
            alert('Flat file imported successfully. Check the console for the file contents.');
        } catch (error) {
            console.error('Error reading file:', error);
            alert('Error reading file');
        }
    }

    async function importSpreadsheet() {
        if (!selectedFile) {
            alert('Please select a file');
            return;
        }

        try {
            const fileContents = await selectedFile.text();
            spreadsheetContents = fileContents;
            console.log('File contents:', fileContents);
            alert('Spreadsheet imported successfully. Check the console for the file contents.');
        } catch (error) {
             console.error('Error reading file:', error);
            alert('Error reading file');
        }
    }

    async function importSqlite() {
        if (!selectedFile) {
            alert('Please select a file');
            return;
        }

        try {
            const fileContents = await selectedFile.text();
            sqliteContents = fileContents;
            console.log('File contents:', fileContents);
            alert('SQLite database imported successfully. Check the console for the file contents.');
        } catch (error) {
            console.error('Error reading file:', error);
            alert('Error reading file');
        }
    }

    function importDatabase() {
        if (!selectedDatabaseType) {
            alert('Please select a database type');
            return;
        }

        if (!databaseConnectionString) {
            alert('Please enter a connection string');
            return;
        }

        console.log('Importing database:', selectedDatabaseType, databaseConnectionString);
        alert('Database import initiated. Check the console for details.');
    }
</script>
<div class="w-1/2 p-4">
    <h2 class="text-2xl font-bold mb-4">Input</h2>
    <div class="flex items-center space-x-2">
        <Select.Root selected={selectedImportType} onSelectedChange={(value) => selectedImportType = value}>
            <Select.Trigger class="w-full">
                <Select.Value placeholder="Select Input Source" />
            </Select.Trigger>
            <Select.Content>
                {#each inputPresets as preset}
                    <Select.Item value={preset}>{preset}</Select.Item>
                {/each}
                <Select.Item value="add_new" onclick={() => showModal = true}>Add New Input Source</Select.Item>
            </Select.Content>
            <Select.Input name="importType" bind:value={$selectedImportType} />
        </Select.Root>
    </div>
    <Dialog open={showModal} onOpenChange={(open) => showModal = open}>
        <DialogContent>
            <DialogHeader>
                <DialogTitle>Input Source</DialogTitle>
            </DialogHeader>
        </DialogContent>
    </Dialog>
</div>