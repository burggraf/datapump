<script lang="ts">
    import { Root as Button } from '$lib/components/ui/button';
    import Label from '$lib/components/ui/label/label.svelte';
    import { Root as Input } from '$lib/components/ui/input';
    import { invoke } from '@tauri-apps/api/core';

    let { selectedDatabaseType, databaseConnectionString } = $props<{
        selectedDatabaseType: 'postgres' | 'mysql' | 'sqlserver' | null,
        databaseConnectionString: { user?: string, password?: string, host?: string, port?: number, dbname?: string } | null
    }>();

    let sqlQuery = $state<string>('');
    let queryResult = $state<any>(null);
    let queryError = $state<string | null>(null);

    async function handleQuery() {
        console.log('Executing query:', sqlQuery);
        if (!selectedDatabaseType || !databaseConnectionString?.user || !databaseConnectionString?.password || !databaseConnectionString?.host || !databaseConnectionString?.port || !databaseConnectionString?.dbname) {
            queryError = 'Invalid connection string: missing connection details';
            return;
        }
        try {
            const connectionString = `${selectedDatabaseType}://${databaseConnectionString?.user}:${databaseConnectionString?.password}@${databaseConnectionString?.host}:${databaseConnectionString?.port}/${databaseConnectionString?.dbname}`;
            const result = await invoke("execute_query", { connectionString, query: sqlQuery });
            queryResult = result;
            queryError = null;
        } catch (e: any) {
            queryResult = null;
            queryError = e.toString();
        }
    }
</script>

<div class="mt-4">
    <Label for="sql-query">SQL Query</Label>
    <Input id="sql-query" type="textarea" bind:value={sqlQuery} placeholder="Enter SQL query here" />
    <Button class="mt-2" onclick={handleQuery}>Execute Query</Button>
    {#if queryError}
        <p class="text-red-500 mt-2">Error: {queryError}</p>
    {:else}
    {/if}
    {#if queryResult}
        <pre class="mt-2">{JSON.stringify(queryResult, null, 2)}</pre>
    {/if}
</div>