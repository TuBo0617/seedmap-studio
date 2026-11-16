import type { Dimension } from './api'

export interface HistoryEntry {
  seed: string
  version: string
  dimension: Dimension
  at: number
}

const KEY = 'seedmap.history.v1'
const LIMIT = 40

export function loadHistory(): HistoryEntry[] {
  try {
    const raw = localStorage.getItem(KEY)
    if (!raw) return []
    const parsed = JSON.parse(raw)
    if (!Array.isArray(parsed)) return []
    return parsed
      .filter(
        (item): item is HistoryEntry =>
          typeof item?.seed === 'string' &&
          typeof item?.version === 'string' &&
          typeof item?.at === 'number',
      )
      .slice(0, LIMIT)
  } catch {
    return []
  }
}

export function recordHistory(entry: HistoryEntry): HistoryEntry[] {
  const rest = loadHistory().filter(
    (item) => !(item.seed === entry.seed && item.version === entry.version),
  )
  const next = [entry, ...rest].slice(0, LIMIT)
  save(next)
  return next
}

export function removeHistory(seed: string, version: string): HistoryEntry[] {
  const next = loadHistory().filter(
    (item) => !(item.seed === seed && item.version === version),
  )
  save(next)
  return next
}

export function clearHistory(): HistoryEntry[] {
  save([])
  return []
}

function save(entries: HistoryEntry[]) {
  try {
    localStorage.setItem(KEY, JSON.stringify(entries))
  } catch {
    // Private mode or a full quota: history is a convenience, not state.
  }
}

export function relativeTime(at: number): string {
  const diff = Date.now() - at
  const minute = 60_000
  const hour = 60 * minute
  const day = 24 * hour
  if (diff < minute) return '刚刚'
  if (diff < hour) return `${Math.floor(diff / minute)} 分钟前`
  if (diff < day) return `${Math.floor(diff / hour)} 小时前`
  if (diff < 7 * day) return `${Math.floor(diff / day)} 天前`
  return new Date(at).toLocaleDateString('zh-CN')
}
