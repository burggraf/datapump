<script lang="ts">
	let { dispatch } = $props();

	let host = $state("");
	let port = $state("");
	let user = $state("");
	let password = $state("");
	let database = $state("");
	let databaseType = $state<"PostgreSQL" | "MySQL" | "SQL Server">("PostgreSQL");

	function handleInputChange() {
		dispatch({
			host,
			port,
			user,
			password,
			database,
			databaseType
		});
	}

	function handleDatabaseTypeChange(event: Event) {
		const target = event.target as HTMLSelectElement;
		databaseType = target.value as "PostgreSQL" | "MySQL" | "SQL Server";
		handleInputChange();
	}
</script>

<div>
	<label>
		Database Type:
		<select value={databaseType} onchange={handleDatabaseTypeChange}>
			<option value="PostgreSQL">PostgreSQL</option>
			<option value="MySQL">MySQL</option>
			<option value="SQL Server">SQL Server</option>
		</select>
	</label>
	<label>
		Host:
		<input type="text" bind:value={host} oninput={handleInputChange} />
	</label>
	<label>
		Port:
		<input type="text" bind:value={port} oninput={handleInputChange} />
	</label>
	<label>
		User:
		<input type="text" bind:value={user} oninput={handleInputChange} />
	</label>
	<label>
		Password:
		<input type="password" bind:value={password} oninput={handleInputChange} />
	</label>
	<label>
		Database:
		<input type="text" bind:value={database} oninput={handleInputChange} />
	</label>
</div>
