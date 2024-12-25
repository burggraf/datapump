<script lang="ts">
    import * as Input from "$lib/components/ui/input";
    import { parse } from 'papaparse';
    import { Table, TableBody, TableCaption, TableCell, TableHead, TableHeader, TableRow } from '$lib/components/ui/table';
    import { cn } from '$lib/utils';
    import * as Button from '$lib/components/ui/button';
    import * as Select from '$lib/components/ui/select';
    
    let file = $state<File | null>(null);
    let schema = $state<string[]>([]);
    let parsedData = $state<any[][]>([]);
    let selectedFileType = $state<'csv' | 'tsv'>('csv');

    const handleFileChange = (event: Event) => {
        const target = event.target as HTMLInputElement;
        if (target && target.files) {
            file = target.files[0];
        }
    };

    $effect(() => {
        if (file) {
            const reader = new FileReader();
            reader.onload = (event) => {
                if (event.target && event.target.result) {
                    const csvData = event.target.result as string;
                    const result = parse(csvData, { header: true, dynamicTyping: true, delimiter: selectedFileType === 'tsv' ? '\t' : ',' });
                    if (result.data && result.meta && result.meta.fields) {
                        schema = result.meta.fields;
                        parsedData = result.data as any[][];
                    }
                }
            };
            reader.readAsText(file);
        }
    });

    function triggerFileInput() {
        const input = document.getElementById('file-input') as HTMLInputElement;
        input.click();
    }
</script>

<div class="flex flex-col gap-4">
    <div class="flex gap-2 items-center">
        <Button.Root onclick={triggerFileInput}>
            Choose File
        </Button.Root>
        <Select.Root >
            <Select.Trigger>
                <Select.Value>
                    {selectedFileType === 'csv' ? 'CSV' : 'TSV'}
                </Select.Value>
            </Select.Trigger>
            <Select.Content>
                <Select.Item value="csv" onclick={() => selectedFileType = 'csv'}>CSV</Select.Item>
                <Select.Item value="tsv" onclick={() => selectedFileType = 'tsv'}>TSV</Select.Item>
            </Select.Content>
        </Select.Root>
    </div>
    <input type="file" id="file-input" accept=".csv, .txt, .tsv" onchange={handleFileChange} class="hidden" />

    {#if schema.length > 0}
        <Table>
            <TableCaption>Schema of the uploaded file</TableCaption>
            <TableHeader>
                <TableRow>
                    {#each schema as column}
                        <TableHead>{column}</TableHead>
                    {/each}
                </TableRow>
            </TableHeader>
            <TableBody>
                {#each parsedData as row}
                    <TableRow>
                        {#each schema as column}
                            <TableCell>{row[column as keyof typeof row]}</TableCell>
                        {/each}
                    </TableRow>
                {/each}
            </TableBody>
        </Table>
    {/if}
</div>