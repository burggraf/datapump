<script lang="ts">
    import { Root as Card, Header as CardHeader, Title as CardTitle, Content as CardContent } from '$lib/components/ui/card';
    import { Root as Select, Trigger as SelectTrigger, Value as SelectValue, Content as SelectContent, Item as SelectItem } from '$lib/components/ui/select';
    import { Root as Button } from '$lib/components/ui/button';
    import Label from '$lib/components/ui/label/label.svelte';

    let selectedFlatFileType = $state<'csv' | 'tsv' | null>(null);
    let flatFileContents = $state<string | null>(null);

    const exportFlatFile = () => {
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
</script>

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