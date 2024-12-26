<script lang="ts">
    import { Root as Card, Header as CardHeader, Title as CardTitle, Content as CardContent } from '$lib/components/ui/card';
    import { Root as Select, Trigger as SelectTrigger, Value as SelectValue, Content as SelectContent, Item as SelectItem } from '$lib/components/ui/select';
    import { Root as Button } from '$lib/components/ui/button';
    import Label from '$lib/components/ui/label/label.svelte';
    import { Root as Input } from '$lib/components/ui/input';
    import { createEventDispatcher } from 'svelte';

    const dispatch = createEventDispatcher();

    let { selectedDatabaseType, databaseConnectionString } = $props<{
        selectedDatabaseType: 'postgres' | 'mysql' | 'sqlserver' | null,
        databaseConnectionString: { user?: string, password?: string, host?: string, port?: number, dbname?: string } | null
    }>();
    let dbTestResult = $state<string>('Not tested');

    const testDatabaseConnection = async () => {
        if (selectedDatabaseType && databaseConnectionString) {
            if (window.__TAURI__) {
                const connectionString = `${selectedDatabaseType}://${databaseConnectionString?.user}:${databaseConnectionString?.password}@${databaseConnectionString?.host}:${databaseConnectionString?.port}/${databaseConnectionString?.dbname}`;
                    try {
                        const result = await window.__TAURI__.tauri.invoke('connect', { url: connectionString });
                        dbTestResult = 'Database connection successful!';
                        console.log(result);
                    } catch (e) {
                        dbTestResult = 'Failed to connect to database: ' + e;
                        console.error(e);
                    }
            }
        } else {
            alert('Please select a database type and enter connection details.');
        }
    }

    const exportDatabase = () => {
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

    const updateDatabaseConnectionString = (key: string, value: string | number) => {
        if (databaseConnectionString) {
            databaseConnectionString = { ...databaseConnectionString, [key]: value };
            dispatch('databaseConnectionStringChange', databaseConnectionString);
        }
    }

    const updateSelectedDatabaseType = (value: 'postgres' | 'mysql' | 'sqlserver' | null) => {
        selectedDatabaseType = value;
        dispatch('selectedDatabaseTypeChange', selectedDatabaseType);
    }
</script>

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
                    <SelectItem value="postgres" onclick={() => updateSelectedDatabaseType('postgres')}>PostgreSQL</SelectItem>
                    <SelectItem value="mysql" onclick={() => updateSelectedDatabaseType('mysql')}>MySQL</SelectItem>
                    <SelectItem value="sqlserver" onclick={() => updateSelectedDatabaseType('sqlserver')}>SQL Server</SelectItem>
                </SelectContent>
            </Select>
        </div>
        {#if selectedDatabaseType === 'postgres'}
            <div class="mb-4">
                <Label for="postgres-user">User</Label>
                <Input id="postgres-user" class="w-full" value={databaseConnectionString?.user} onchange={(e) => { if (e.target instanceof HTMLInputElement) updateDatabaseConnectionString('user', e.target.value) }} />
            </div>
            <div class="mb-4">
                <Label for="postgres-password">Password</Label>
                <Input type="password" id="postgres-password" class="w-full" value={databaseConnectionString?.password} onchange={(e) => { if (e.target instanceof HTMLInputElement) updateDatabaseConnectionString('password', e.target.value) }} />
            </div>
            <div class="mb-4">
                <Label for="postgres-host">Host</Label>
                <Input id="postgres-host" class="w-full" value={databaseConnectionString?.host} onchange={(e) => { if (e.target instanceof HTMLInputElement) updateDatabaseConnectionString('host', e.target.value) }} />
            </div>
            <div class="mb-4">
                <Label for="postgres-port">Port</Label>
                <Input type="number" id="postgres-port" class="w-full" value={databaseConnectionString?.port} onchange={(e) => { if (e.target instanceof HTMLInputElement) updateDatabaseConnectionString('port', Number(e.target.value)) }} />
            </div>
            <div class="mb-4">
                <Label for="postgres-dbname">Database Name</Label>
                <Input id="postgres-dbname" class="w-full" value={databaseConnectionString?.dbname} onchange={(e) => { if (e.target instanceof HTMLInputElement) updateDatabaseConnectionString('dbname', e.target.value) }} />
            </div>
        <Button class="w-full mb-4" onclick={testDatabaseConnection}>Test Database Connection</Button>
    {/if}
    <Button class="w-full" onclick={exportDatabase}>Export</Button>

    <p>{dbTestResult}</p>
    </CardContent>
</Card>