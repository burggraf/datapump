<script lang="ts">
    import * as Input from "$lib/components/ui/input";
    import { Table, TableCaption, TableHead, TableHeader, TableRow, TableCell } from '$lib/components/ui/table';
    import * as Button from '$lib/components/ui/button';
    import * as Select from '$lib/components/ui/select';
    import { analyzeSchema } from '$lib/services/flat.svelte';

    let file = $state<File | null>(null);
    let schema = $state<{ name: string; type: string }[]>([]);
    let selectedFileType = $state<'csv' | 'tsv'>('csv');

    const handleFileChange = async (event: Event) => {
        const target = event.target as HTMLInputElement;
        if (target && target.files) {
            file = target.files[0];
            if (file) {
                schema = await analyzeSchema(file);
            }
        }
    };

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
                    <TableHead>Name</TableHead>
                    <TableHead>Type</TableHead>
                </TableRow>
            </TableHeader>
            
                {#each schema as column}
                    <TableRow>
                        <TableCell>{column.name}</TableCell>
                        <TableCell>{column.type}</TableCell>
                    </TableRow>
                {/each}
        </Table>
    {/if}
</div>