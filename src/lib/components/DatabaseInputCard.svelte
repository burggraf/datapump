<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { Select, SelectTrigger, SelectValue, SelectContent, SelectItem } from '$lib/components/ui/select';
  import type { DatabaseType } from './types';

  let {
      selectedDatabaseType,
      onDatabaseTypeChange,
      databaseConnectionString,
      onConnectionStringChange,
      onImport
  } = $props<{
      selectedDatabaseType: DatabaseType | null;
      onDatabaseTypeChange: (type: DatabaseType | null) => void;
      databaseConnectionString: string | null;
      onConnectionStringChange: (value: string | null) => void;
      onImport: () => void;
  }>();

  async function handleConnect(connectionString: string | null) {
    if (typeof window !== 'undefined' && window.__TAURI__) {
      console.log('Running inside Tauri environment');
      try {
        await window.__TAURI__.tauri.invoke('connect', { url: connectionString });
        alert('Connected to database!');
      } catch (e) {
        alert('Failed to connect to database: ' + e);
      }
    } else {
      console.log('Attempting database connection in development environment:', connectionString);
      alert('Simulating database connection in development.');
    }
  }

  const parseConnectionString = (connectionString: string | null) => {
      if (!connectionString) return null;
      const url = new URL(connectionString);
      const user = url.username;
      const password = url.password;
      const host = url.hostname;
      const port = url.port;
      const database = url.pathname.substring(1);
      return { user, password, host, port, database };
  };

  $effect.pre(() => {
      const savedSettings = localStorage.getItem('databaseSettings');
      if (savedSettings) {
          try {
              const settings = JSON.parse(savedSettings);
              if (settings?.output?.postgres) {
                  const { user, password, host, port, database } = settings.output.postgres;
                  const connectionString = `postgres://${user}:${password}@${host}:${port}/${database}`;
                  onConnectionStringChange(connectionString);
              }
          } catch (e) {
              console.error("Error parsing saved settings", e);
          }
      }
  });

  $effect(() => {
      if (databaseConnectionString) {
          const parsed = parseConnectionString(databaseConnectionString);
          if (parsed) {
              localStorage.setItem('databaseSettings', JSON.stringify({
                  output: {
                      postgres: parsed
                  }
              }));
          }
      }
  });
</script>

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
                    <SelectItem value="postgres" onclick={() => onDatabaseTypeChange('postgres')}>PostgreSQL</SelectItem>
                    <SelectItem value="mysql" onclick={() => onDatabaseTypeChange('mysql')}>MySQL</SelectItem>
                    <SelectItem value="sqlserver" onclick={() => onDatabaseTypeChange('sqlserver')}>SQL Server</SelectItem>
                </SelectContent>
            </Select>
        </div>
        <div class="mb-4">
          <Label for="remote-db-connection">Connection String</Label>
          <Input
            id="remote-db-connection"
            placeholder="Enter connection string"
            value={databaseConnectionString ?? ''}
            onchange={(e) => onConnectionStringChange((e.target as HTMLInputElement).value)}
          />
        </div>
    </CardContent>
</Card>