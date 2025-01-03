import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import Papa from 'papaparse';

export default class MigrationCard {
    constructor() {
        this.initializeLocalStorage();
        this.setupLocalStorageEffects();
    }

    invoke = invoke;
    listen = listen;
    getCurrentWindow = getCurrentWindow;
    totalRows = $state(0);
    processedRows = $state(0);
    batchSize = $state(0);
    message = $state("");
    status = $state("idle");
    rowsPerSecond = $state(0);
    timeRemainingDisplay = $state("");

    cancellationRequested = $state(false);
    migrationInProgress = $state(false);

    sourceType = $state("csv_tsv");
    sourcePath = $state("");
    sourceUser = $state("");
    sourcePassword = $state("");
    sourceHost = $state("");
    sourcePort = $state("");
    sourceDatabaseName = $state("");
    destinationType = $state("sqlite");
    destinationPath = $state("");
    destinationUser = $state("");
    destinationPassword = $state("");
    destinationHost = $state("");
    destinationPort = $state("");
    destinationDatabaseName = $state("");

    tableName = $state("");
    selectedFile = $state<File | null>(null);

    async analyzeSchema() {
        console.log('Analyzing schema for file:', this.sourcePath);

        try {
            console.log('Starting file analysis');
            const CHUNK_SIZE = 10000; // Number of lines per chunk
            const MAX_LINES = 100000; // Maximum number of lines to analyze
            
            // Get first batch to determine schema
            let offset = 0;
            let hasMoreChunks = true;
            let firstBatch = true;
            const columnTypes: { [key: string]: string } = {};
            let totalRows = 0;
            let fileDelimiter = ',';
            let linebreak = '\n';

            while (hasMoreChunks) {
                console.log(`Reading batch starting at offset ${offset}...`);
                const [chunks, isLastBatch]: [string[], boolean] = await this.invoke<[string[], boolean]>('read_file_chunks', { 
                    filePath: this.sourcePath,
                    chunkSize: CHUNK_SIZE,
                    offset
                });
                
                if (!chunks || chunks.length === 0) {
                    if (firstBatch) {
                        throw new Error('No data in file');
                    }
                    break;
                }

                console.log(`Received ${chunks.length} chunks in batch. Processing...`);

                // Initialize schema from first chunk if this is the first batch
                if (firstBatch) {
                    const firstChunkResults: { data: any[]; meta: Papa.ParseMeta } = await new Promise((resolve, reject) => {
                        console.log('Parsing first chunk to determine schema...');
                        
                        // Get first line and analyze it
                        const firstLine = chunks[0].split('\n')[0];
                        console.log('Analyzing first line:', firstLine);
                        
                        // Count fields with different delimiters
                        const testDelimiters = [',', '\t', ';'];
                        const fieldCounts = testDelimiters.map(d => firstLine.split(d).length);
                        console.log('Field counts by delimiter:', fieldCounts);
                        
                        // Choose delimiter that gives the most fields
                        const maxFields = Math.max(...fieldCounts);
                        const delimiter = testDelimiters[fieldCounts.indexOf(maxFields)];
                        console.log(`Selected delimiter: "${delimiter}" (${maxFields} fields)`);
                        
                        // Set up timeout protection
                        const timeout = setTimeout(() => {
                            console.error('Papa.parse timeout after 30 seconds');
                            reject(new Error('Parser timeout'));
                        }, 30000);
                        
                        Papa.parse(chunks[0], {
                            header: true,
                            delimiter: delimiter,
                            skipEmptyLines: true,
                            complete: (results) => {
                                clearTimeout(timeout);
                                console.log('First chunk parsed successfully:', {
                                    rows: results.data.length,
                                    fields: results.meta.fields?.length,
                                    delimiter: results.meta.delimiter
                                });
                                fileDelimiter = delimiter;
                                linebreak = results.meta.linebreak;
                                resolve(results);
                            },
                            error: (error) => {
                                clearTimeout(timeout);
                                console.error('Error parsing first chunk:', error);
                                reject(error);
                            }
                        });
                    });

                    console.log('Analyzing column types from first chunk...');
                    if (firstChunkResults.data.length > 0) {
                        const firstRow = firstChunkResults.data[0];
                        for (const column in firstRow) {
                            columnTypes[column] = this.detectType(firstRow[column]);
                        }
                    }
                    console.log('Initial column types:', columnTypes);
                    firstBatch = false;
                }

                // Process all chunks in this batch
                let processedChunks = 0;
                const startTime = Date.now();
                
                for (const chunk of chunks) {
                    processedChunks++;
                    console.log(`Analyzing chunk ${processedChunks}/${chunks.length} in current batch`);
                    
                    const chunkResults: { data: any[] } = await new Promise((resolve, reject) => {
                        Papa.parse(chunk, {
                            header: true,
                            delimiter: fileDelimiter,
                            skipEmptyLines: true,
                            complete: (results) => {
                                console.log(`Chunk parsed: ${results.data.length} rows`);
                                resolve(results);
                            },
                            error: (error) => {
                                console.error(`Error parsing chunk:`, error);
                                reject(error);
                            }
                        });
                    });

                    totalRows += chunkResults.data.length;
                    const elapsedSeconds = (Date.now() - startTime) / 1000;
                    console.log(`Processed ${totalRows} total rows in ${elapsedSeconds.toFixed(1)}s`);

                    // Stop if we've reached the maximum number of lines
                    if (totalRows >= MAX_LINES) {
                        console.log(`Reached maximum line limit of ${MAX_LINES}. Stopping analysis.`);
                        hasMoreChunks = false;
                        break;
                    }

                    // Update column types based on new data
                    if (chunkResults.data.length > 0) {
                        for (const column in columnTypes) {
                            // Only check if we haven't already determined it's text
                            if (columnTypes[column] !== 'text') {
                                const values = chunkResults.data
                                    .map(row => row[column])
                                    .filter(val => val !== null && val !== undefined && val !== '');

                                if (columnTypes[column] === 'integer' || columnTypes[column] === 'number') {
                                    const allNumbers = values.every(val => !isNaN(Number(val)));
                                    if (!allNumbers) {
                                        columnTypes[column] = 'text';
                                    } else if (columnTypes[column] === 'integer') {
                                        const allIntegers = values.every(val => Number.isInteger(Number(val)));
                                        if (!allIntegers) {
                                            columnTypes[column] = 'number';
                                        }
                                    }
                                } else if (columnTypes[column] === 'date') {
                                    const allDates = values.every(val => !isNaN(Date.parse(val)));
                                    if (!allDates) {
                                        columnTypes[column] = 'text';
                                    }
                                }
                            }
                        }
                    }
                }

                // Break out of main loop if we hit the line limit
                if (!hasMoreChunks) {
                    break;
                }

                // Update offset and check if we need to continue
                offset += chunks.length * CHUNK_SIZE;
                hasMoreChunks = !isLastBatch;
                console.log(`Batch complete. Total rows so far: ${totalRows}. Continue: ${hasMoreChunks}`);
            }

            console.log('Final column types:', columnTypes);
            return {
                delimiter: fileDelimiter,
                fields: columnTypes,
                linebreak
            };
        } catch (error) {
            console.error('Error analyzing schema:', error);
            throw error;
        }
    }

    private detectType(value: any): string {
        // Handle null, undefined, or empty string
        if (value === null || value === undefined || value === '') {
            return 'text';
        }

        // Try parsing as number
        const num = Number(value);
        if (!isNaN(num)) {
            return Number.isInteger(num) ? 'integer' : 'number';
        }

        // Try parsing as date
        const date = new Date(value);
        if (!isNaN(date.getTime())) {
            return 'date';
        }

        // Check for boolean values
        const lowerValue = String(value).toLowerCase();
        if (lowerValue === 'true' || lowerValue === 'false') {
            return 'boolean';
        }

        // Default to text
        return 'text';
    }

    private initializeLocalStorage() {
        const storedSourcePath = localStorage.getItem("sourcePath");
        if (storedSourcePath) this.sourcePath = storedSourcePath;
        const storedDestinationPath = localStorage.getItem("destinationPath");
        if (storedDestinationPath) this.destinationPath = storedDestinationPath;
        const storedTableName = localStorage.getItem("tableName");
        if (storedTableName) this.tableName = storedTableName;
        const storedSourceType = localStorage.getItem("sourceType");
        if (storedSourceType) this.sourceType = storedSourceType;
        const storedSourceUser = localStorage.getItem("sourceUser");
        if (storedSourceUser) this.sourceUser = storedSourceUser;
        const storedSourcePassword = localStorage.getItem("sourcePassword");
        if (storedSourcePassword) this.sourcePassword = storedSourcePassword;
        const storedSourceHost = localStorage.getItem("sourceHost");
        if (storedSourceHost) this.sourceHost = storedSourceHost;
        const storedSourcePort = localStorage.getItem("sourcePort");
        if (storedSourcePort) this.sourcePort = storedSourcePort;
        const storedSourceDatabaseName = localStorage.getItem("sourceDatabaseName");
        if (storedSourceDatabaseName) this.sourceDatabaseName = storedSourceDatabaseName;
        const storedDestinationType = localStorage.getItem("destinationType");
        if (storedDestinationType) this.destinationType = storedDestinationType;
        const storedDestinationUser = localStorage.getItem("destinationUser");
        if (storedDestinationUser) this.destinationUser = storedDestinationUser;
        const storedDestinationPassword = localStorage.getItem("destinationPassword");
        if (storedDestinationPassword) this.destinationPassword = storedDestinationPassword;
        const storedDestinationHost = localStorage.getItem("destinationHost");
        if (storedDestinationHost) this.destinationHost = storedDestinationHost;
        const storedDestinationPort = localStorage.getItem("destinationPort");
        if (storedDestinationPort) this.destinationPort = storedDestinationPort;
        const storedDestinationDatabaseName = localStorage.getItem("destinationDatabaseName");
        if (storedDestinationDatabaseName) this.destinationDatabaseName = storedDestinationDatabaseName;
    }

    private setupLocalStorageEffects() {
        $effect(() => {
            localStorage.setItem("sourcePath", this.sourcePath);
            localStorage.setItem("destinationPath", this.destinationPath);
            localStorage.setItem("tableName", this.tableName);
            localStorage.setItem("sourceType", this.sourceType);
            localStorage.setItem("sourceUser", this.sourceUser);
            localStorage.setItem("sourcePassword", this.sourcePassword);
            localStorage.setItem("sourceHost", this.sourceHost);
            localStorage.setItem("sourcePort", this.sourcePort);
            localStorage.setItem("sourceDatabaseName", this.sourceDatabaseName);
            localStorage.setItem("destinationType", this.destinationType);
            localStorage.setItem("destinationUser", this.destinationUser);
            localStorage.setItem("destinationPassword", this.destinationPassword);
            localStorage.setItem("destinationHost", this.destinationHost);
            localStorage.setItem("destinationPort", this.destinationPort);
            localStorage.setItem("destinationDatabaseName", this.destinationDatabaseName);
            // this.tableName = this.tableNameFromPath();
        });
    }

    /*
    tableNameFromPath = $derived(() => {
        if (!this.sourcePath) return "";
        const filename = this.sourcePath.split("/").pop() || "";
        return filename.replace(/\.[^/.]+$/, "");
    });
    */

    async cancelMigration() {
        this.cancellationRequested = true;
        this.status = "cancelling";
        try {
            await invoke("cancel_migration");
            this.status = "cancelled";
            this.message = "Migration cancelled by user";
            this.migrationInProgress = false;
        } catch (error) {
            console.error("Error cancelling migration:", error);
            this.status = "error";
            this.message = "Failed to cancel migration";
            this.migrationInProgress = false;
        }
    }

    async startMigration() {
        // Reset state variables
        this.totalRows = 0;
        this.processedRows = 0;
        this.batchSize = 0;
        this.message = "";
        this.status = "idle";
        this.timeRemainingDisplay = "";
        this.cancellationRequested = false;
        this.migrationInProgress = true;

        // Reset cancellation flag in Rust
        await invoke("reset_cancellation");

        let ts = +new Date();
        // Setup event listener
        const unlisten = await listen<ProgressEvent>("migration_progress", (event) => {
            if (this.cancellationRequested) return;

            this.processedRows = event.payload.processed_rows;
            this.totalRows = event.payload.total_rows;
            this.batchSize = event.payload.batch_size;
            this.status = event.payload.status;
            this.message = event.payload.message || "";
            const elapsed = (+new Date() - ts) / 1000;
            const rps = elapsed > 0 ? this.processedRows / elapsed : 0;
            this.rowsPerSecond = Math.round(rps);
            
            // Calculate time remaining only if we have a valid rate and total rows
            if (rps > 0 && this.totalRows > 0 && this.processedRows < this.totalRows) {
                let timeRemaining = (this.totalRows - this.processedRows) / rps;
                const hours = Math.floor(timeRemaining / 3600);
                const minutes = Math.floor((timeRemaining % 3600) / 60);
                const seconds = Math.floor(timeRemaining % 60);
                this.timeRemainingDisplay = `${hours}h ${minutes}m ${seconds}s`;
            }
            
            if (this.status === "parsing_schema_complete" || this.status === "counted_rows") {
                ts = +new Date();
            }
        });

        try {
            const schema = await invoke("get_csv_schema", { filePath: this.sourcePath, tableName: this.tableName });

            if (typeof schema !== "string") {
                throw new Error("Invalid schema format: expected string");
            }

            const schemaParts = schema.split(",");

            // Validate schema format
            if (!schemaParts.every((part) => part.includes(":"))) {
                throw new Error("Invalid schema format: each part should be in 'name:type' format");
            }

            const window = getCurrentWindow();
            const result = await invoke(`csv_to_${this.destinationType}`, {
                window,
                filePath: this.sourcePath,
                batchSize: 10000,
                schema: schema,
                tableName: this.tableName,
                dbPath: this.destinationPath
            });
        } catch (error) {
            console.error("Error during CSV to SQLite migration:", error);
            this.status = "Error: " + (error as string) || "ERROR";
            this.migrationInProgress = false;
            throw error;
        } finally {
            // Clean up event listener
            unlisten();
            this.migrationInProgress = false;
        }
    }

    async startMigration2() {
        // Reset state variables
        this.totalRows = 0;
        this.processedRows = 0;
        this.batchSize = 0;
        this.message = "";
        this.status = "idle";
        this.timeRemainingDisplay = "";
        this.cancellationRequested = false;
        this.migrationInProgress = true;

        try {
            console.log("Starting migration with source:", this.sourcePath);
            
            // Check if table exists first for PostgreSQL
            if (this.destinationType === "postgres") {
                console.log("Checking if table exists:", this.tableName);
                const exists = await invoke("check_postgres_table_exists", {
                    connectionString: this.getConnectionString(),
                    tableName: this.tableName
                });
                
                if (exists) {
                    throw `Table '${this.tableName}' already exists. Aborting import.`;
                }
            }

            // Analyze schema
            console.log("Analyzing schema...");
            const schemaInfo = await this.analyzeSchema();
            console.log("Schema Analysis Results:");
            console.log("Field Delimiter:", schemaInfo.delimiter);
            console.log("Line Break:", schemaInfo.linebreak);
            console.log("Fields:", schemaInfo.fields);

            // Reset cancellation flag in Rust
            console.log("Resetting cancellation flag...");
            await invoke("reset_cancellation");

            let ts = +new Date();
            // Setup event listener
            console.log("Setting up progress listener...");
            const unlisten = await listen<ProgressEvent>("migration_progress", (event) => {
                if (this.cancellationRequested) return;

                this.processedRows = event.payload.processed_rows;
                this.totalRows = event.payload.total_rows;
                this.batchSize = event.payload.batch_size;
                this.status = event.payload.status;
                this.message = event.payload.message || "";
                const elapsed = (+new Date() - ts) / 1000;
                const rps = elapsed > 0 ? this.processedRows / elapsed : 0;
                this.rowsPerSecond = Math.round(rps);
                
                // Calculate time remaining only if we have a valid rate and total rows
                if (rps > 0 && this.totalRows > 0 && this.processedRows < this.totalRows) {
                    let timeRemaining = (this.totalRows - this.processedRows) / rps;
                    const hours = Math.floor(timeRemaining / 3600);
                    const minutes = Math.floor((timeRemaining % 3600) / 60);
                    const seconds = Math.floor(timeRemaining % 60);
                    this.timeRemainingDisplay = `${hours}h ${minutes}m ${seconds}s`;
                }
                
                if (this.status === "parsing_schema_complete" || this.status === "counted_rows") {
                    ts = +new Date();
                }
            });

            try {
                console.log("Starting import to PostgreSQL...");
                const connectionString = this.getConnectionString();
                console.log("Using connection string (password hidden):", 
                    connectionString.replace(/:[^:@]+@/, ':***@'));
                
                const fields = Object.entries(schemaInfo.fields).map(([name, type]) => ({ name, type }));
                console.log("Prepared fields for import:", fields);

                const result = await invoke("import_csv_to_postgres", {
                    connectionString,
                    pathToFile: this.sourcePath,
                    tableName: this.tableName,
                    delimiter: schemaInfo.delimiter,
                    linebreak: schemaInfo.linebreak,
                    fields
                });
                
                console.log("Import completed successfully");
            } catch (error) {
                // Don't treat cancellation as an error
                if (error === "Migration cancelled by user") {
                    console.log("Migration cancelled by user");
                    return;
                }
                
                console.error("Error during CSV to PostgreSQL migration:", error);
                this.status = "Error: " + (error as string) || "ERROR";
                throw error;
            } finally {
                console.log("Cleaning up...");
                unlisten();
                this.migrationInProgress = false;
            }
        } catch (error) {
            console.error("Error during migration process:", error);
            this.status = "Error: " + (error as string) || "ERROR";
            this.migrationInProgress = false;
            throw error;
        }
    }

    getConnectionString() {
        // URL encode the password to handle special characters
        const encodedPassword = encodeURIComponent(this.destinationPassword);
        const encodedUser = encodeURIComponent(this.destinationUser);
        return `postgresql://${encodedUser}:${encodedPassword}@${this.destinationHost}:${this.destinationPort}/${this.destinationDatabaseName}`;
    }
}

interface ProgressEvent {
    total_rows: number;
    processed_rows: number;
    row_count: number;
    batch_size: number;
    status: string;
    message?: string;
}