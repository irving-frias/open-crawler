import { invoke } from '@tauri-apps/api/core';

export interface ExportPackageInfo {
  path: string;
  file_name: string;
  size_bytes: number;
  project_count: number;
  lightweight: boolean;
  include_credentials: boolean;
}

export interface ImportEntry {
  id: string;
  name: string;
  page_count: number;
}

export interface ImportSummary {
  imported: ImportEntry[];
  skipped: ImportEntry[];
  warnings: string[];
}

export interface TransferInfo {
  urls: string[];
  port: number;
  token: string;
  expires_in_secs: number;
  file_name: string;
  file_size_bytes: number;
}

export interface TransferProgress {
  stage: string;
  processed: number;
  total: number;
  percent: number;
}

export type ImportConflictMode = 'skip' | 'copy' | 'overwrite';

export function exportPackage(
  projectIds: string[],
  filePath: string,
  lightweight?: boolean,
  includeCredentials?: boolean,
  shareAfter?: boolean,
  silent?: boolean
): Promise<ExportPackageInfo> {
  return invoke<ExportPackageInfo>('export_package', {
    projectIds,
    filePath,
    lightweight,
    includeCredentials,
    shareAfter,
    silent,
  });
}

export function importPackage(filePath: string, mode: ImportConflictMode): Promise<ImportSummary> {
  return invoke<ImportSummary>('import_package', { filePath, mode });
}

export function startTransferServer(filePath: string, minutes?: number): Promise<TransferInfo> {
  return invoke<TransferInfo>('start_transfer_server', { filePath, minutes });
}

export function stopTransferServer(): Promise<void> {
  return invoke<void>('stop_transfer_server');
}

export function getActiveTransfer(): Promise<TransferInfo | null> {
  return invoke<TransferInfo | null>('get_active_transfer');
}

export function downloadTransfer(url: string, dest: string): Promise<void> {
  return invoke<void>('download_transfer', { url, dest });
}

export function importSharedIntent(mode: ImportConflictMode): Promise<ImportSummary | null> {
  return invoke<ImportSummary | null>('import_shared_intent', { mode });
}

export function openShareSheet(filePath: string): Promise<void> {
  return invoke<void>('open_share_sheet', { filePath });
}

export function btSend(addr: string, filePath: string): Promise<void> {
  return invoke<void>('bt_send', { addr, filePath });
}
