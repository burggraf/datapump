<script lang="ts">
    import { Root as Tabs, List as TabsList, Trigger as TabsTrigger, Content as TabsContent } from '$lib/components/ui/tabs';
    import { Root as Card, Header as CardHeader, Title as CardTitle, Content as CardContent } from '$lib/components/ui/card';
    import { Root as Select, Trigger as SelectTrigger, Value as SelectValue, Content as SelectContent, Item as SelectItem } from '$lib/components/ui/select';
    import { Root as Button } from '$lib/components/ui/button';
    import Label from '$lib/components/ui/label/label.svelte';
    import { Root as Input } from '$lib/components/ui/input';
    import type { FileType, DatabaseType } from './types';

    let selectedExportType = $state<'flat' | 'spreadsheet' | 'sqlite' | 'database' | null>(null);
    let flatFileContents = $state<string | null>(null);
    let spreadsheetContents = $state<string | null>(null);
    let sqliteContents = $state<string | null>(null);
    let selectedFlatFileType = $state<'csv' | 'tsv' | null>(null);
    let selectedSpreadsheetType = $state<'excel' | 'google-sheets' | null>(null);
    let selectedSqliteType = $state<'db' | 'zip' | null>(null);
    let selectedDatabaseType = $state<'postgres' | 'mysql' | 'sqlserver' | null>(null);
    let databaseConnectionString = $state<{ user?: string, password?: string, host?: string, port?: number, dbname?: string } | null>(null);

    $effect.pre(() => {
        const savedSettings = localStorage.getItem('databaseSettings');
        if (savedSettings) {
            try {
                const settings = JSON.parse(savedSettings);
                if (settings?.output?.postgres) {
                    databaseConnectionString = settings.output.postgres;
                }
            } catch (e) {
                console.error("Error parsing saved settings", e);
            }
        }
    });

    $effect(() => {
        if (databaseConnectionString) {
            localStorage.setItem('databaseSettings', JSON.stringify({
                output: {
                    postgres: databaseConnectionString
                }
            }));
        }
    });

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

<div class="w-1/2 p-4">
    <h2 class="text-2xl font-bold mb-4">Output</h2>
     <Tabs value={selectedExportType || 'database'}>
        <TabsList>
            <TabsTrigger value="flat" onclick={() => selectedExportType = 'flat'}>Flat</TabsTrigger>
            <TabsTrigger value="spreadsheet" onclick={() => selectedExportType = 'spreadsheet'}>Spreadsheet</TabsTrigger>
            <TabsTrigger value="sqlite" onclick={() => selectedExportType = 'sqlite'}>SQLite</TabsTrigger>
             <TabsTrigger value="database" onclick={() => selectedExportType = 'database'}>Database</TabsTrigger>
        </TabsList>
        <TabsContent value="flat">
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
        </TabsContent>
        <TabsContent value="spreadsheet">
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
        </TabsContent>
         <TabsContent value="sqlite">
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
        </TabsContent>
        <TabsContent value="database">
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
                                <SelectItem value="postgres" onclick={() => selectedDatabaseType = 'postgres'}>PostgreSQL</SelectItem>
                                <SelectItem value="mysql" onclick={() => selectedDatabaseType = 'mysql'}>MySQL</SelectItem>
                                <SelectItem value="sqlserver" onclick={() => selectedDatabaseType = 'sqlserver'}>SQL Server</SelectItem>
                            </SelectContent>
                        </Select>
                    </div>
                    {#if selectedDatabaseType === 'postgres'}
                        <div class="mb-4">
                            <Label for="postgres-user">User</Label>
                            <Input id="postgres-user" class="w-full" value={databaseConnectionString?.user} onchange={(e: Event) => { if (e.target instanceof HTMLInputElement) databaseConnectionString = {...databaseConnectionString, user: e.target.value} }} />
                        </div>
                        <div class="mb-4">
                            <Label for="postgres-password">Password</Label>
                            <Input type="password" id="postgres-password" class="w-full" value={databaseConnectionString?.password} onchange={(e: Event) => { if (e.target instanceof HTMLInputElement) databaseConnectionString = {...databaseConnectionString, password: e.target.value} }} />
                        </div>
                        <div class="mb-4">
                            <Label for="postgres-host">Host</Label>
                            <Input id="postgres-host" class="w-full" value={databaseConnectionString?.host} onchange={(e: Event) => { if (e.target instanceof HTMLInputElement) databaseConnectionString = {...databaseConnectionString, host: e.target.value} }} />
                        </div>
                        <div class="mb-4">
                            <Label for="postgres-port">Port</Label>
                            <Input type="number" id="postgres-port" class="w-full" value={databaseConnectionString?.port} onchange={(e: Event) => { if (e.target instanceof HTMLInputElement) databaseConnectionString = {...databaseConnectionString, port: Number(e.target.value)} }} />
                        </div>
                        <div class="mb-4">
                            <Label for="postgres-dbname">Database Name</Label>
                            <Input id="postgres-dbname" class="w-full" value={databaseConnectionString?.dbname} onchange={(e: Event) => { if (e.target instanceof HTMLInputElement) databaseConnectionString = {...databaseConnectionString, dbname: e.target.value} }} />
                            <Button class="w-full mt-2">Test</Button>
                        </div>
                    {/if}
                    <Button class="w-full" onclick={exportDatabase}>Export</Button>
                </CardContent>
            </Card>
        </TabsContent>
    </Tabs>
</div>