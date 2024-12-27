declare module '@tauri-apps/api/tauri' {
  export function invoke(cmd: string, args?: any): Promise<any>;
}