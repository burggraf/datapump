import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

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
    rows_per_second = $state(0);
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
            const rps = this.processedRows / elapsed;
            this.rows_per_second = Math.round(this.processedRows / elapsed);
            // calculate estimated time remaining
            let timeRemaining = (this.totalRows - this.processedRows) / rps;
            if (timeRemaining > 0 && isFinite(timeRemaining)) {
                const minutes = Math.floor(timeRemaining / 60);
                const seconds = Math.floor(timeRemaining % 60);
                this.timeRemainingDisplay = `${minutes}:${seconds < 10 ? "0" : ""}${seconds}`;
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
                batchSize: 50000,
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
            const rps = this.processedRows / elapsed;
            this.rows_per_second = Math.round(this.processedRows / elapsed);
            
            if (this.status === "parsing_schema_complete" || this.status === "counted_rows") {
                ts = +new Date();
            }
        });

        try {
            // Construct connection string for PostgreSQL
            const connectionString = `postgresql://${this.destinationUser}:${this.destinationPassword}@${this.destinationHost}:${this.destinationPort}/${this.destinationDatabaseName}`;

            const result = await invoke("import_csv_to_postgres", {
                connectionString,
                pathToFile: this.sourcePath,
                tableName: this.tableName
            });
        } catch (error) {
            console.error("Error during CSV to PostgreSQL migration:", error);
            this.status = "Error: " + (error as string) || "ERROR";
            this.migrationInProgress = false;
            throw error;
        } finally {
            // Clean up event listener
            unlisten();
            this.migrationInProgress = false;
        }
    }
}

interface ProgressEvent {
    processed_rows: number;
    row_count: number;
    total_rows: number;
    batch_size: number;
    status: string;
    message?: string;
}