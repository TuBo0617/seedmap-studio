<script setup lang="ts">
import { computed, ref } from 'vue'
import { BIOME_NAMES } from '../lib/biomes'

const props = defineProps<{
  biomes: number[]
  selected: number[]
}>()

const emit = defineEmits<{
  (event: 'update:selected', value: number[]): void
  (event: 'close'): void
}>()

const query = ref('')

const label = (id: number) => BIOME_NAMES[id] ?? `群系 ${id}`

const items = computed(() => {
  const keyword = query.value.trim().toLowerCase()
  return props.biomes
    .map((id) => ({ id, label: label(id) }))
    .filter((item) => !keyword || item.label.toLowerCase().includes(keyword))
})

const selectedSet = computed(() => new Set(props.selected))

function toggle(id: number) {
  const next = new Set(props.selected)
  if (next.has(id)) next.delete(id)
  else next.add(id)
  emit('update:selected', [...next])
}

function setAll(on: boolean) {
  emit('update:selected', on ? props.biomes.slice(0, 24) : [])
}
</script>

<template>
  <section class="panel">
    <header>
      <div>
        <b>生物群系高亮</b>
        <span class="sub">{{ selected.length }} 项已选</span>
      </div>
      <button class="close" aria-label="关闭" @click="emit('close')">×</button>
    </header>

    <div class="tools">
      <input v-model="query" placeholder="搜索群系" spellcheck="false" />
      <button @click="setAll(false)">清空</button>
      <button @click="setAll(true)">常见</button>
    </div>

    <div class="list">
      <button
        v-for="item in items"
        :key="item.id"
        class="row"
        :class="{ on: selectedSet.has(item.id) }"
        @click="toggle(item.id)"
      >
        <i :style="{ background: selectedSet.has(item.id) ? '#ff77b9' : '#3a3a43' }" />
        <span>{{ item.label }}</span>
        <em>{{ item.id }}</em>
      </button>
      <p v-if="!items.length" class="empty">没有匹配的群系</p>
    </div>
  </section>
</template>

<style scoped>
.panel {
  display: flex;
  flex-direction: column;
  width: 260px;
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

.tools {
  display: flex;
  gap: 6px;
  padding: 8px 10px;
  border-bottom: 1px solid var(--border);
}

.tools input {
  flex: 1;
  min-width: 0;
  height: 26px;
  padding: 0 8px;
  background: var(--panel-2);
  border: 1px solid var(--border);
  border-radius: 6px;
  outline: none;
  font-size: 11.5px;
}

.tools button {
  height: 26px;
  padding: 0 8px;
  background: var(--panel-2);
  border: 1px solid var(--border);
  border-radius: 6px;
  font-size: 11px;
  color: var(--muted);
}

.list {
  overflow-y: auto;
  padding: 4px;
}

.row {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 6px 8px;
  border-radius: 7px;
  font-size: 12px;
  text-align: left;
  color: var(--muted);
}

.row:hover {
  background: var(--panel-2);
}

.row.on {
  color: var(--text);
  background: var(--accent-soft);
}

.row i {
  width: 9px;
  height: 9px;
  border-radius: 3px;
  flex: 0 0 auto;
}

.row span {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.row em {
  font-style: normal;
  font-size: 10px;
  color: #55555f;
}

.empty {
  margin: 12px;
  font-size: 11.5px;
  color: var(--muted);
  text-align: center;
}
</style>
