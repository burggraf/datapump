<script lang="ts">
    import { Root as Tabs, List as TabsList, Trigger as TabsTrigger, Content as TabsContent } from '$lib/components/ui/tabs';
    import FileInputCard from '$lib/components/FileInputCard.svelte';
    import DatabaseInputCard from '$lib/components/DatabaseInputCard.svelte';
    import type { FileType } from './types';

    let selectedImportType = $state<'flat' | 'spreadsheet' | 'sqlite' | 'database' | null>(null);
    let selectedFile: File | null = null;
    let flatFileContents = $state<string | null>(null);
    let spreadsheetContents = $state<string | null>(null);
    let sqliteContents = $state<string | null>(null);
    let selectedFlatFileType = $state<'csv' | 'tsv' | null>(null);
    let selectedSpreadsheetType = $state<'excel' | 'google-sheets' | null>(null);
    let selectedSqliteType = $state<'db' | 'zip' | null>(null);
    let selectedDatabaseType = $state<'postgres' | 'mysql' | 'sqlserver' | null>(null);
    let databaseConnectionString = $state<string | null>(null);

    function handleFileChange(event: Event) {
        const input = event.target as HTMLInputElement;
        if (input.files && input.files.length > 0) {
            selectedFile = input.files[0];
            console.log('Selected file:', selectedFile);
        }
    }

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
    <Tabs value={selectedImportType || 'flat'}>
        <TabsList>
            <TabsTrigger value="flat" onclick={() => selectedImportType = 'flat'}>Flat</TabsTrigger>
            <TabsTrigger value="spreadsheet" onclick={() => selectedImportType = 'spreadsheet'}>Spreadsheet</TabsTrigger>
            <TabsTrigger value="sqlite" onclick={() => selectedImportType = 'sqlite'}>SQLite</TabsTrigger>
            <TabsTrigger value="database" onclick={() => selectedImportType = 'database'}>Database</TabsTrigger>
        </TabsList>
        <TabsContent value="flat">
            <FileInputCard
                fileType="flat"
                selectedFileType={selectedFlatFileType}
                onFileTypeChange={(type) => selectedFlatFileType = type}
                onFileChange={handleFileChange}
                onImport={importFlatFile}
                accept=".csv,.tsv"
            />
        </TabsContent>
        <TabsContent value="spreadsheet">
            <FileInputCard
                fileType="spreadsheet"
                selectedFileType={selectedSpreadsheetType}
                onFileTypeChange={(type) => selectedSpreadsheetType = type}
                onFileChange={handleFileChange}
                onImport={importSpreadsheet}
                accept=".xls,.xlsx,.xlsm,.xlsb,.ods"
            />
        </TabsContent>
        <TabsContent value="sqlite">
            <FileInputCard
                fileType="sqlite"
                selectedFileType={selectedSqliteType}
                onFileTypeChange={(type) => selectedSqliteType = type}
                onFileChange={handleFileChange}
                onImport={importSqlite}
                accept=".sqlite,.db,.sqlite3"
                openAsOptions={[{value: 'db', label: 'Database File'}, {value: 'zip', label: 'Zip Archive'}]}
            />
        </TabsContent>
        <TabsContent value="database">
            <DatabaseInputCard
                selectedDatabaseType={selectedDatabaseType}
                onDatabaseTypeChange={(type) => selectedDatabaseType = type}
                databaseConnectionString={databaseConnectionString}
                onConnectionStringChange={(value) => databaseConnectionString = value}
                onImport={importDatabase}
            />
        </TabsContent>
    </Tabs>
</div>