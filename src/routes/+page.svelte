<script lang="ts">
    import { Root as Tabs, List as TabsList, Trigger as TabsTrigger, Content as TabsContent } from '$lib/components/ui/tabs';
    import { Root as Button } from '$lib/components/ui/button';
    import { Root as Card, Header as CardHeader, Title as CardTitle, Content as CardContent } from '$lib/components/ui/card';
    import { Root as Input } from '$lib/components/ui/input';
    import { Root as Select, Trigger as SelectTrigger, Value as SelectValue, Content as SelectContent, Item as SelectItem } from '$lib/components/ui/select';
    import { Root as DropdownMenu, Item as DropdownMenuItem, Content as DropdownMenuContent, Trigger as DropdownMenuTrigger } from '$lib/components/ui/dropdown-menu';
    import Label from '$lib/components/ui/label/label.svelte';

    let selectedFile: File | null = null;
    let selectedOperation = $state<'import' | 'export' | null>(null);
    let selectedImportType = $state<'flat' | 'spreadsheet' | 'sqlite' | 'database' | null>(null);
    let selectedExportType = $state<'flat' | 'spreadsheet' | 'sqlite' | 'database' | null>(null);
    let flatFileContents = $state<string | null>(null);
    let selectedFlatFileType = $state<'csv' | 'tsv' | null>(null);
    let spreadsheetContents = $state<string | null>(null);
    let selectedSpreadsheetType = $state<'excel' | 'google-sheets' | null>(null);
    let sqliteContents = $state<string | null>(null);
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

    function exportFlatFile() {
        if (!flatFileContents) {
            alert('No data to export');
            return;
        }

        if (!selectedFlatFileType) {
            alert('Please select a file type to export');
            return;
        }

        const blob = new Blob([flatFileContents], { type: `text/${selectedFlatFileType}` });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `data.${selectedFlatFileType}`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
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

    function exportSpreadsheet() {
         if (!spreadsheetContents) {
            alert('No data to export');
            return;
        }

        if (!selectedSpreadsheetType) {
            alert('Please select a file type to export');
            return;
        }

        const blob = new Blob([spreadsheetContents], { type: 'application/octet-stream' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `data.${selectedSpreadsheetType}`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
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

    function exportSqlite() {
        if (!sqliteContents) {
            alert('No data to export');
            return;
        }

        if (!selectedSqliteType) {
            alert('Please select a file type to export');
            return;
        }

        const blob = new Blob([sqliteContents], { type: 'application/octet-stream' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `data.${selectedSqliteType}`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
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

    function exportDatabase() {
        if (!selectedDatabaseType) {
            alert('Please select a database type');
            return;
        }

         if (!databaseConnectionString) {
            alert('Please enter a connection string');
            return;
        }

        console.log('Exporting database:', selectedDatabaseType, databaseConnectionString);
        alert('Database export initiated. Check the console for details.');
    }
</script>

<div class="container mx-auto p-4">
    <Tabs value={selectedOperation || 'import'}>
        <TabsList>
            <TabsTrigger value="import" onclick={() => selectedOperation = 'import'}>Import</TabsTrigger>
            <TabsTrigger value="export" onclick={() => selectedOperation = 'export'}>Export</TabsTrigger>
        </TabsList>
        {#if selectedOperation === 'import'}
        <TabsContent value="import">
            <h2 class="text-2xl font-bold mb-4">Import Data</h2>
            <DropdownMenu>
                <DropdownMenuTrigger>Select Import Type</DropdownMenuTrigger>
                <DropdownMenuContent>
                    <DropdownMenuItem onclick={() => selectedImportType = 'flat'}>Flat Files</DropdownMenuItem>
                    <DropdownMenuItem onclick={() => selectedImportType = 'spreadsheet'}>Spreadsheets</DropdownMenuItem>
                    <DropdownMenuItem onclick={() => selectedImportType = 'sqlite'}>SQLite</DropdownMenuItem>
                    <DropdownMenuItem onclick={() => selectedImportType = 'database'}>Databases</DropdownMenuItem>
                </DropdownMenuContent>
            </DropdownMenu>
            {#if selectedImportType === 'flat'}
                <Card>
                    <CardHeader>
                        <CardTitle>Flat Files</CardTitle>
                    </CardHeader>
                    <CardContent>
                        <div class="mb-4">
                            <Label for="flat-file-type">File Type</Label>
                            <Select>
                                <SelectTrigger id="flat-file-type" class="w-full">
                                    <SelectValue placeholder="Select file type" />
                                </SelectTrigger>
                                <SelectContent>
                                    <SelectItem value="csv">CSV</SelectItem>
                                    <SelectItem value="tsv">TSV</SelectItem>
                                </SelectContent>
                            </Select>
                        </div>
                        <div class="mb-4">
                            <Label for="flat-file-upload">Upload File</Label>
                            <Input type="file" id="flat-file-upload" accept=".csv,.tsv" onchange={handleFileChange} />
                        </div>
                        <Button class="w-full" onclick={importFlatFile}>Import</Button>
                    </CardContent>
                </Card>
            {/if}
            {#if selectedImportType === 'spreadsheet'}
                <Card>
                    <CardHeader>
                        <CardTitle>Spreadsheets</CardTitle>
                    </CardHeader>
                    <CardContent>
                        <div class="mb-4">
                            <Label for="spreadsheet-type">File Type</Label>
                            <Select>
                                <SelectTrigger id="spreadsheet-type" class="w-full">
                                    <SelectValue placeholder="Select file type" />
                                </SelectTrigger>
                                <SelectContent>
                                    <SelectItem value="excel" onclick={() => selectedSpreadsheetType = 'excel'}>Excel</SelectItem>
                                    <SelectItem value="google-sheets" onclick={() => selectedSpreadsheetType = 'google-sheets'}>Google Sheets</SelectItem>
                                </SelectContent>
                            </Select>
                        </div>
                        <div class="mb-4">
                            <Label for="spreadsheet-upload">Upload File</Label>
                             <Input type="file" id="spreadsheet-upload" accept=".xls,.xlsx,.xlsm,.xlsb,.ods" onchange={handleFileChange} />
                        </div>
                        <Button class="w-full" onclick={importSpreadsheet}>Import</Button>
                    </CardContent>
                </Card>
            {/if}
            {#if selectedImportType === 'sqlite'}
                 <Card>
                    <CardHeader>
                        <CardTitle>SQLite</CardTitle>
                    </CardHeader>
                    <CardContent>
                        <div class="mb-4">
                            <Label for="sqlite-file-upload">Upload File</Label>
                            <Input type="file" id="sqlite-file-upload" accept=".sqlite,.db,.sqlite3" onchange={handleFileChange} />
                        </div>
                        <div class="mb-4">
                            <Label for="sqlite-open-as">Open As</Label>
                            <Select>
                                <SelectTrigger id="sqlite-open-as" class="w-full">
                                    <SelectValue placeholder="Select open as" />
                                </SelectTrigger>
                                <SelectContent>
                                    <SelectItem value="db" onclick={() => selectedSqliteType = 'db'}>Database File</SelectItem>
                                    <SelectItem value="zip" onclick={() => selectedSqliteType = 'zip'}>Zip Archive</SelectItem>
                                </SelectContent>
                            </Select>
                        </div>
                        <Button class="w-full" onclick={importSqlite}>Import</Button>
                    </CardContent>
                </Card>
            {/if}
            {#if selectedImportType === 'database'}
                <Card>
                    <CardHeader>
                        <CardTitle>Databases</CardTitle>
                    </CardHeader>
                    <CardContent>
                        <div class="mb-4">
                            <Label for="remote-db-type">Database Type</Label>
                            <Select>
                                <SelectTrigger id="remote-db-type" class="w-full">
                                    <SelectValue placeholder="Select database type" />
                                </SelectTrigger>
                                <SelectContent>
                                    <SelectItem value="postgres" onclick={() => selectedDatabaseType = 'postgres'}>PostgreSQL</SelectItem>
                                    <SelectItem value="mysql" onclick={() => selectedDatabaseType = 'mysql'}>MySQL</SelectItem>
                                    <SelectItem value="sqlserver" onclick={() => selectedDatabaseType = 'sqlserver'}>SQL Server</SelectItem>
                                </SelectContent>
                            </Select>
                        </div>
                        <div class="mb-4">
                            <Label for="remote-db-connection">Connection String</Label>
                            <Input type="text" id="remote-db-connection" placeholder="Enter connection string" on:input={(e) => databaseConnectionString = e.target.value} />
                        </div>
                        <Button class="w-full" onclick={importDatabase}>Import</Button>
                    </CardContent>
                </Card>
            {/if}
        </TabsContent>
        {/if}
        {#if selectedOperation === 'export'}
        <TabsContent value="export">
            <h2 class="text-2xl font-bold mb-4">Export Data</h2>
            <DropdownMenu>
                <DropdownMenuTrigger>Select Export Type</DropdownMenuTrigger>
                <DropdownMenuContent>
                     <DropdownMenuItem onclick={() => selectedExportType = 'flat'}>Flat Files</DropdownMenuItem>
                    <DropdownMenuItem onclick={() => selectedExportType = 'spreadsheet'}>Spreadsheets</DropdownMenuItem>
                    <DropdownMenuItem onclick={() => selectedExportType = 'sqlite'}>SQLite</DropdownMenuItem>
                    <DropdownMenuItem onclick={() => selectedExportType = 'database'}>Databases</DropdownMenuItem>
                </DropdownMenuContent>
            </DropdownMenu>
             {#if selectedExportType === 'flat'}
                <Card>
                    <CardHeader>
                        <CardTitle>Flat Files</CardTitle>
                    </CardHeader>
                    <CardContent>
                        <div class="mb-4">
                            <Label for="flat-file-type-export">File Type</Label>
                            <Select>
                                <SelectTrigger id="flat-file-type-export" class="w-full">
                                    <SelectValue placeholder="Select file type" />
                                </SelectTrigger>
                                <SelectContent>
                                    <SelectItem value="csv" onclick={() => selectedFlatFileType = 'csv'}>CSV</SelectItem>
                                    <SelectItem value="tsv" onclick={() => selectedFlatFileType = 'tsv'}>TSV</SelectItem>
                                </SelectContent>
                            </Select>
                        </div>
                        <Button class="w-full" onclick={exportFlatFile}>Export</Button>
                    </CardContent>
                </Card>
            {/if}
            {#if selectedExportType === 'spreadsheet'}
                <Card>
                    <CardHeader>
                        <CardTitle>Spreadsheets</CardTitle>
                    </CardHeader>
                    <CardContent>
                        <div class="mb-4">
                            <Label for="spreadsheet-type-export">File Type</Label>
                            <Select>
                                <SelectTrigger id="spreadsheet-type-export" class="w-full">
                                    <SelectValue placeholder="Select file type" />
                                </SelectTrigger>
                                <SelectContent>
                                    <SelectItem value="excel" onclick={() => selectedSpreadsheetType = 'excel'}>Excel</SelectItem>
                                    <SelectItem value="google-sheets" onclick={() => selectedSpreadsheetType = 'google-sheets'}>Google Sheets</SelectItem>
                                </SelectContent>
                            </Select>
                        </div>
                        <Button class="w-full" onclick={exportSpreadsheet}>Export</Button>
                    </CardContent>
                </Card>
            {/if}
             {#if selectedExportType === 'sqlite'}
                 <Card>
                     <CardHeader>
                         <CardTitle>SQLite</CardTitle>
                     </CardHeader>
                     <CardContent>
                         <div class="mb-4">
                             <Label for="sqlite-export-as">Export As</Label>
                             <Select>
                                 <SelectTrigger id="sqlite-export-as" class="w-full">
                                     <SelectValue placeholder="Select export type" />
                                 </SelectTrigger>
                                 <SelectContent>
                                     <SelectItem value="db" onclick={() => selectedSqliteType = 'db'}>Database File</SelectItem>
                                     <SelectItem value="zip" onclick={() => selectedSqliteType = 'zip'}>Zip Archive</SelectItem>
                                 </SelectContent>
                             </Select>
                         </div>
                         <Button class="w-full" onclick={exportSqlite}>Export</Button>
                     </CardContent>
                 </Card>
            {/if}
            {#if selectedExportType === 'database'}
                <Card>
                    <CardHeader>
                        <CardTitle>Remote Databases</CardTitle>
                    </CardHeader>
                    <CardContent>
                        <div class="mb-4">
                            <Label for="remote-db-type-export">Database Type</Label>
                            <Select>
                                <SelectTrigger id="remote-db-type-export" class="w-full">
                                    <SelectValue placeholder="Select database type" />
                                </SelectTrigger>
                                <SelectContent>
                                    <SelectItem value="postgres">PostgreSQL</SelectItem>
                                    <SelectItem value="mysql">MySQL</SelectItem>
                                    <SelectItem value="sqlserver">SQL Server</SelectItem>
                                </SelectContent>
                            </Select>
                        </div>
                        <Button class="w-full" onclick={() => alert('Export Remote DB')}>Export</Button>
                    </CardContent>
                </Card>
            {/if}
        </TabsContent>
        {/if}
    </Tabs>
</div>