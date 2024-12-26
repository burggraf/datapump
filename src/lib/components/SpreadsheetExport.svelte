<script lang="ts">
    import { Root as Card, Header as CardHeader, Title as CardTitle, Content as CardContent } from '$lib/components/ui/card';
    import { Root as Select, Trigger as SelectTrigger, Value as SelectValue, Content as SelectContent, Item as SelectItem } from '$lib/components/ui/select';
    import { Root as Button } from '$lib/components/ui/button';
    import Label from '$lib/components/ui/label/label.svelte';

    let selectedSpreadsheetType = $state<'excel' | 'google-sheets' | null>(null);
    let spreadsheetContents = $state<string | null>(null);

    const exportSpreadsheet = () => {
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
</script>

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