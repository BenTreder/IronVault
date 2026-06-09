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

export interface SnapshotInfo {
  name: string
  created_at?: string
  file_count?: number
  files?: number
  directories?: number
  symlinks?: number
  total_size?: number
}

export const defaultRepoPath = '/mnt/backups/ironvault'

export async function getRepoInfo(repoPath: string): Promise<RepoInfo> {
  return await invoke<RepoInfo>('get_info', { repoPath })
}

export async function verifyRepository(repoPath: string): Promise<VerifyResult> {
  return await invoke<VerifyResult>('verify_repository', { repoPath })
}

export async function listSnapshots(repoPath: string): Promise<SnapshotInfo[]> {
  return await invoke<SnapshotInfo[]>('list_snapshots', { repoPath })
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
