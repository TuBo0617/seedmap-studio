<script setup lang="ts">
import type { HistoryEntry } from '../lib/history'
import { relativeTime } from '../lib/history'

defineProps<{
  entries: HistoryEntry[]
  current: string
}>()

const emit = defineEmits<{
  (event: 'pick', entry: HistoryEntry): void
  (event: 'remove', entry: HistoryEntry): void
  (event: 'clear'): void
  (event: 'close'): void
}>()

const DIM_LABEL: Record<string, string> = {
  overworld: '主世界',
  nether: '下界',
  end: '末地',
}
</script>

<template>
  <section class="panel">
    <header>
      <div>
        <b>最近种子</b>
        <span class="sub">{{ entries.length }} 条</span>
      </div>
      <button class="close" aria-label="关闭" @click="emit('close')">×</button>
    </header>

    <div class="list">
      <p v-if="!entries.length" class="empty">还没有记录，生成一次地图就会出现在这里。</p>
      <div
        v-for="entry in entries"
        :key="`${entry.seed}-${entry.version}`"
        class="row"
        :class="{ current: entry.seed === current }"
      >
        <button class="main" @click="emit('pick', entry)">
          <span class="seed">{{ entry.seed }}</span>
          <span class="meta">
            {{ entry.version }} · {{ DIM_LABEL[entry.dimension] ?? entry.dimension }} ·
            {{ relativeTime(entry.at) }}
          </span>
        </button>
        <button class="remove" aria-label="删除" @click="emit('remove', entry)">×</button>
      </div>
    </div>

    <footer>
      <button class="btn" :disabled="!entries.length" @click="emit('clear')">清空历史</button>
      <button class="btn ghost" @click="emit('close')">关闭</button>
    </footer>
  </section>
</template>

<style scoped>
.panel {
  display: flex;
  flex-direction: column;
  width: 280px;
  max-height: min(70vh, 560px);
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: 12px;
  box-shadow: 0 18px 40px rgba(0, 0, 0, 0.5);
  overflow: hidden;
}

header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  border-bottom: 1px solid var(--border);
}

header b {
  font-size: 12.5px;
}

.sub {
  margin-left: 8px;
  font-size: 10.5px;
  color: var(--muted);
}

.close {
  font-size: 16px;
  line-height: 1;
  color: var(--muted);
}

.list {
  overflow-y: auto;
  padding: 4px;
}

.empty {
  margin: 14px 12px;
  font-size: 11.5px;
  line-height: 1.6;
  color: var(--muted);
}

.row {
  display: flex;
  align-items: center;
  gap: 2px;
  border-radius: 8px;
}

.row:hover {
  background: var(--panel-2);
}

.row.current {
  background: var(--accent-soft);
}

.main {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 7px 9px;
  text-align: left;
}

.seed {
  font-size: 12px;
  font-variant-numeric: tabular-nums;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.meta {
  font-size: 10px;
  color: var(--muted);
}

.remove {
  width: 26px;
  height: 26px;
  border-radius: 6px;
  color: #5f5f6a;
  font-size: 14px;
}

.remove:hover {
  color: #ff8a80;
}

footer {
  display: flex;
  gap: 6px;
  padding: 10px;
  border-top: 1px solid var(--border);
}

.btn {
  flex: 1;
  height: 30px;
  border-radius: 7px;
  background: var(--panel-2);
  border: 1px solid var(--border);
  font-size: 11.5px;
  color: var(--text);
}

.btn.ghost {
  flex: 0 0 auto;
  padding: 0 12px;
  color: var(--muted);
}

.btn:disabled {
  opacity: 0.45;
}
</style>
