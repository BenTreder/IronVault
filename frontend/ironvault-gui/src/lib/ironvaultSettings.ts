import { defaultRepoPath } from './ironvaultBridge'

export const repoPathStorageKey = 'ironvault-repo-path'

export function loadRepoPath(): string {
  return localStorage.getItem(repoPathStorageKey) || defaultRepoPath
}

export function saveRepoPath(repoPath: string): string {
  const cleanPath = repoPath.trim() || defaultRepoPath

  localStorage.setItem(repoPathStorageKey, cleanPath)

  return cleanPath
}

export function resetRepoPath(): string {
  localStorage.removeItem(repoPathStorageKey)

  return defaultRepoPath
}
