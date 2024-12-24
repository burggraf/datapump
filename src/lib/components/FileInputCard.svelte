<script lang="ts">
    import { Root as Card, Header as CardHeader, Title as CardTitle, Content as CardContent } from '$lib/components/ui/card';
    import { Root as Input } from '$lib/components/ui/input';
    import { Root as Select, Trigger as SelectTrigger, Value as SelectValue, Content as SelectContent, Item as SelectItem } from '$lib/components/ui/select';
    import { Root as Button } from '$lib/components/ui/button';
    import Label from '$lib/components/ui/label/label.svelte';
    import type { FileType } from './types';
    import FileUpload from './FileUpload.svelte';

    let {
        fileType,
        selectedFileType,
        onFileTypeChange,
        onFileChange,
        onImport,
        accept,
        openAsOptions
    } = $props<{
        fileType: 'flat' | 'spreadsheet' | 'sqlite';
        selectedFileType: FileType | null;
        onFileTypeChange: (type: FileType | null) => void;
        onFileChange: (event: Event) => void;
        onImport: () => void;
        accept: string;
        openAsOptions?: { value: string, label: string }[];
    }>();
    let schema = $state<{ name: string; type: string }[]>([]);
</script>

<Card>
    <CardHeader>
        <CardTitle>{fileType === 'flat' ? 'Flat Files' : fileType === 'spreadsheet' ? 'Spreadsheets' : 'SQLite'}</CardTitle>
    </CardHeader>
    <CardContent>
        {#if fileType !== 'sqlite'}
            <div class="mb-4">
                <Label for="{fileType}-file-type">File Type</Label>
                <Select>
                    <SelectTrigger id="{fileType}-file-type" class="w-full">
                        <SelectValue placeholder="Select file type" />
                    </SelectTrigger>
                    <SelectContent>
                        {#if fileType === 'flat'}
                            <SelectItem value="csv" onclick={() => onFileTypeChange('csv')}>CSV</SelectItem>
                            <SelectItem value="tsv" onclick={() => onFileTypeChange('tsv')}>TSV</SelectItem>
                        {:else if fileType === 'spreadsheet'}
                            <SelectItem value="excel" onclick={() => onFileTypeChange('excel')}>Excel</SelectItem>
                            <SelectItem value="google-sheets" onclick={() => onFileTypeChange('google-sheets')}>Google Sheets</SelectItem>
                        {/if}
                    </SelectContent>
                </Select>
            </div>
        {/if}
        
        {#if fileType === 'sqlite' && openAsOptions}
            <div class="mb-4">
                <Label for="sqlite-open-as">Open As</Label>
                <Select>
                    <SelectTrigger id="sqlite-open-as" class="w-full">
                        <SelectValue placeholder="Select open as" />
                    </SelectTrigger>
                    <SelectContent>
                        {#each openAsOptions as option}
                            <SelectItem value={option.value} onclick={() => onFileTypeChange(option.value)}>{option.label}</SelectItem>
                        {/each}
                    </SelectContent>
                </Select>
            </div>
        {/if}
        <Button class="w-full" onclick={onImport}>Import</Button>
        <FileUpload bind:schema={schema} onFileChange={onFileChange} />
    </CardContent>
</Card>