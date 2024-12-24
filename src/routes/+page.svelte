<script lang="ts">
    import { Root as Tabs, List as TabsList, Trigger as TabsTrigger, Content as TabsContent } from '$lib/components/ui/tabs';
    import { Root as Button } from '$lib/components/ui/button';
    import { Root as Card, Header as CardHeader, Title as CardTitle, Content as CardContent } from '$lib/components/ui/card';
    import { Root as Input } from '$lib/components/ui/input';
    import { Root as Select, Trigger as SelectTrigger, Value as SelectValue, Content as SelectContent, Item as SelectItem } from '$lib/components/ui/select';
    import { Root as DropdownMenu, Item as DropdownMenuItem, Content as DropdownMenuContent, Trigger as DropdownMenuTrigger } from '$lib/components/ui/dropdown-menu';
    import Label from '$lib/components/ui/label/label.svelte';

    let selectedFile: File | null = null;

    function handleFileChange(event: Event) {
        const input = event.target as HTMLInputElement;
        if (input.files && input.files.length > 0) {
            selectedFile = input.files[0];
            console.log('Selected file:', selectedFile);
        }
    }
</script>

<div class="container mx-auto p-4">
    <Tabs>
        <TabsList>
            <TabsTrigger value="import">Import</TabsTrigger>
            <TabsTrigger value="export">Export</TabsTrigger>
        </TabsList>
        <TabsContent value="import">
            <h2 class="text-2xl font-bold mb-4">Import Data</h2>
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
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
                        <Button onclick={() => alert('Import Flat File')}>Import</Button>
                    </CardContent>
                </Card>
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
                                    <SelectItem value="excel">Excel</SelectItem>
                                    <SelectItem value="google-sheets">Google Sheets</SelectItem>
                                </SelectContent>
                            </Select>
                        </div>
                        <div class="mb-4">
                            <Label for="spreadsheet-upload">Upload File</Label>
                             <Input type="file" id="spreadsheet-upload" accept=".xls,.xlsx,.xlsm,.xlsb,.ods" onchange={handleFileChange} />
                        </div>
                        <Button onclick={() => alert('Import Spreadsheet')}>Import</Button>
                    </CardContent>
                </Card>
                 <Card>
                    <CardHeader>
                        <CardTitle>SQLite</CardTitle>
                    </CardHeader>
                    <CardContent>
                        <div class="mb-4">
                            <Label for="sqlite-file-upload">Upload File</Label>
                            <Input type="file" id="sqlite-file-upload" accept=".sqlite,.db,.sqlite3" onchange={handleFileChange} />
                        </div>
                        <Button onclick={() => alert('Import SQLite')}>Import</Button>
                    </CardContent>
                </Card>
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
                                    <SelectItem value="postgres">PostgreSQL</SelectItem>
                                    <SelectItem value="mysql">MySQL</SelectItem>
                                    <SelectItem value="sqlserver">SQL Server</SelectItem>
                                </SelectContent>
                            </Select>
                        </div>
                        <div class="mb-4">
                            <Label for="remote-db-connection">Connection String</Label>
                            <Input type="text" id="remote-db-connection" placeholder="Enter connection string" />
                        </div>
                        <Button onclick={() => alert('Import Remote DB')}>Import</Button>
                    </CardContent>
                </Card>
            </div>
        </TabsContent>
        <TabsContent value="export">
            <h2 class="text-2xl font-bold mb-4">Export Data</h2>
             <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
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
                                    <SelectItem value="csv">CSV</SelectItem>
                                    <SelectItem value="tsv">TSV</SelectItem>
                                </SelectContent>
                            </Select>
                        </div>
                        <Button onclick={() => alert('Export Flat File')}>Export</Button>
                    </CardContent>
                </Card>
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
                                    <SelectItem value="excel">Excel</SelectItem>
                                    <SelectItem value="google-sheets">Google Sheets</SelectItem>
                                </SelectContent>
                            </Select>
                        </div>
                        <Button onclick={() => alert('Export Spreadsheet')}>Export</Button>
                    </CardContent>
                </Card>
                 <Card>
                    <CardHeader>
                        <CardTitle>SQLite</CardTitle>
                    </CardHeader>
                    <CardContent>
                        <Button onclick={() => alert('Export SQLite')}>Export</Button>
                    </CardContent>
                </Card>
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
                        <Button onclick={() => alert('Export Remote DB')}>Export</Button>
                    </CardContent>
                </Card>
            </div>
        </TabsContent>
    </Tabs>
</div>