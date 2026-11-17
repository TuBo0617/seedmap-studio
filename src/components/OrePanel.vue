<script setup lang="ts">
import { computed } from 'vue'
import type { Dimension, OreKind } from '../lib/api'
import { oreItemIcon } from '../lib/icons'
import { oresForDimension } from '../lib/ores'

const props = defineProps<{
  dimension: Dimension
  supported: boolean
  active: OreKind[]
  scanning: boolean
  status: string
  hits: number
}>()

const emit = defineEmits<{
  (event: 'update:active', value: OreKind[]): void
  (event: 'scan'): void
  (event: 'clear'): void
  (event: 'close'): void
}>()

const list = computed(() => oresForDimension(props.dimension))
const activeSet = computed(() => new Set(props.active))

function toggle(kind: OreKind) {
  const next = new Set(props.active)
  if (next.has(kind)) next.delete(kind)
  else next.add(kind)
  emit('update:active', [...next])
}

function reset() {
  emit('update:active', list.value.filter((ore) => ore.primary).map((ore) => ore.kind))
}
</script>

<template>
  <section class="panel">
    <header>
      <div>
        <b>矿石分布</b>
        <span class="sub">{{ active.length }} 种 · {{ hits }} 处</span>
      </div>
      <button class="close" aria-label="关闭" @click="emit('close')">×</button>
    </header>

    <p v-if="!supported" class="note">当前版本或维度不支持矿石预测（1.13+ 主世界 / 下界）。</p>

    <div v-else class="grid">
      <button
        v-for="ore in list"
        :key="ore.kind"
        class="chip"
        :class="{ on: activeSet.has(ore.kind) }"
        :title="`Y ${ore.yMin} ~ ${ore.yMax}`"
        @click="toggle(ore.kind)"
      >
        <i class="cube" :class="{ on: activeSet.has(ore.kind) }">
          <img
            v-if="oreItemIcon(ore.kind)"
            :src="oreItemIcon(ore.kind)"
            :alt="ore.label"
            loading="lazy"
          />
          <b v-else :style="{ background: ore.color }" />
        </i>
        <span>{{ ore.label }}</span>
      </button>
    </div>

    <p class="note small">
      只预测“候选位置”，实际生成还受地形与空气暴露影响；结果按 16×16 区块返回。
    </p>

    <footer>
      <button class="btn primary" :disabled="!supported || scanning || !active.length" @click="emit('scan')">
        {{ scanning ? '扫描中…' : '扫描当前视野' }}
      </button>
      <button class="btn" @click="emit('clear')">清除</button>
      <button class="btn ghost" @click="reset">默认</button>
    </footer>
    <p v-if="status" class="status">{{ status }}</p>
  </section>
</template>

<style scoped>
.panel {
  display: flex;
  flex-direction: column;
  width: 300px;
  max-height: min(76vh, 640px);
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

.grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 5px;
  padding: 10px;
  overflow-y: auto;
}

.chip {
  display: flex;
  align-items: center;
  gap: 5px;
  height: 28px;
  padding: 0 7px;
  background: var(--panel-2);
  border: 1px solid var(--border);
  border-radius: 7px;
  font-size: 11px;
  color: var(--muted);
  overflow: hidden;
}

.chip.on {
  color: var(--text);
  border-color: rgba(255, 119, 185, 0.45);
  background: var(--accent-soft);
}

.chip i.cube {
  width: 20px;
  height: 20px;
  border-radius: 5px;
  flex: 0 0 auto;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.04);
}

.chip i.cube.on {
  background: rgba(255, 119, 185, 0.14);
  box-shadow: 0 0 0 1px rgba(255, 119, 185, 0.4);
}

.chip i.cube img {
  width: 18px;
  height: 18px;
  object-fit: contain;
  image-rendering: -webkit-optimize-contrast;
}

.chip i.cube b {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  opacity: 0.55;
}

.chip.on i.cube b {
  opacity: 1;
  box-shadow: 0 0 0 2px rgba(0, 0, 0, 0.25);
}

.chip i.cube:not(.on) img {
  filter: grayscale(0.35);
  opacity: 0.75;
}

.chip span {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.note {
  margin: 0;
  padding: 10px 12px;
  font-size: 11.5px;
  line-height: 1.55;
  color: var(--muted);
}

.note.small {
  padding-top: 0;
  font-size: 10.5px;
  color: #6b6b76;
}

footer {
  display: flex;
  gap: 6px;
  padding: 10px;
  border-top: 1px solid var(--border);
}

.btn {
  height: 30px;
  padding: 0 10px;
  border-radius: 7px;
  background: var(--panel-2);
  border: 1px solid var(--border);
  font-size: 11.5px;
  color: var(--text);
}

.btn.primary {
  flex: 1;
  background: var(--accent);
  border-color: transparent;
  color: #24101a;
  font-weight: 650;
}

.btn:disabled {
  opacity: 0.45;
  cursor: default;
}

.btn.ghost {
  color: var(--muted);
}

.status {
  margin: 0;
  padding: 0 12px 10px;
  font-size: 10.5px;
  color: var(--muted);
}
</style>
