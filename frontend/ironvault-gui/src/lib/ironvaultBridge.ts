import { invoke } from '@tauri-apps/api/core'

export interface RepoInfo {
  path: string
  snapshot_count: number
  total_size: number
  total_chunks?: number
  free_space: number
}

export interface VerifyResult {
  valid: boolean
  message: string
}

export interface BackupResult {
  success: boolean
  message: string
}

export interface SetupTestVaultResult {
  repo_path: string
  config_path: string
  source_path: string
  initialized_repo: boolean
  message: string
}

export interface SetupCustomVaultResult {
  repo_path: string
  config_path: string
  source_path: string
  initialized_repo: boolean
  message: string
}

export interface SnapshotInfo {
  name: string
  created_at?: string
  file_count?: number
  files?: number
  directories?: number
  symlinks?: number
  total_size?: number
}

export interface RestoreConflict {
  source_path: string
  target_path: string
  kind: string
}

export interface RestorePlanInfo {
  snapshot: string
  target: string
  files: number
  directories: number
  symlinks: number
  total_size: number
  conflict_count: number
  safe_to_restore: boolean
  conflicts: RestoreConflict[]
}

export interface RestoreResult {
  success: boolean
  message: string
}

export const defaultRepoPath = '/mnt/backups/ironvault'

export async function getRepoInfo(repoPath: string): Promise<RepoInfo> {
  return await invoke<RepoInfo>('get_info', { repoPath })
}

export async function verifyRepository(repoPath: string): Promise<VerifyResult> {
  return await invoke<VerifyResult>('verify_repository', { repoPath })
}

export async function createBackup(configPath: string): Promise<BackupResult> {
  return await invoke<BackupResult>('create_backup', { configPath })
}

export async function setupTestVault(): Promise<SetupTestVaultResult> {
  return await invoke<SetupTestVaultResult>('setup_test_vault')
}

export async function setupCustomVault(
  sourcePath: string,
  repoPath: string,
  configPath: string
): Promise<SetupCustomVaultResult> {
  return await invoke<SetupCustomVaultResult>('setup_custom_vault', {
    sourcePath,
    repoPath,
    configPath
  })
}

export async function listSnapshots(repoPath: string): Promise<SnapshotInfo[]> {
  return await invoke<SnapshotInfo[]>('list_snapshots', { repoPath })
}

export async function getRestorePlan(
  repoPath: string,
  snapshot: string,
  targetPath: string
): Promise<RestorePlanInfo> {
  return await invoke<RestorePlanInfo>('restore_plan', {
    repoPath,
    snapshot,
    targetPath
  })
}

export async function restoreSnapshot(
  repoPath: string,
  snapshot: string,
  targetPath: string
): Promise<RestoreResult> {
  return await invoke<RestoreResult>('restore_snapshot', {
    repoPath,
    snapshot,
    targetPath
  })
}

export function snapshotFileCount(snapshot: SnapshotInfo): number {
  return snapshot.file_count ?? snapshot.files ?? 0
}

export function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes <= 0) {
    return '0 B'
  }

  const units = ['B', 'KiB', 'MiB', 'GiB', 'TiB']
  let size = bytes
  let unitIndex = 0

  while (size >= 1024 && unitIndex < units.length - 1) {
    size = size / 1024
    unitIndex++
  }

  const rounded = size >= 10 || unitIndex === 0 ? size.toFixed(0) : size.toFixed(1)
  return `${rounded} ${units[unitIndex]}`
}
