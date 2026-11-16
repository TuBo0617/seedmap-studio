<script setup lang="ts">
import { computed, ref } from 'vue'
import { biomeAt, type Dimension, type Edition, type MapFeature } from '../lib/api'
import { biomeName } from '../lib/biomes'
import { featureMeta } from '../lib/features'

const props = defineProps<{
  features: MapFeature[]
  seed: string
  edition: Edition
  version: string
  dimension: Dimension
  bounds: { minX: number; minZ: number; maxX: number; maxZ: number }
}>()

const emit = defineEmits<{ (event: 'close'): void }>()

const sampling = ref(false)
const progress = ref(0)
const biomes = ref<{ id: number; count: number }[]>([])

const centerX = computed(() => (props.bounds.minX + props.bounds.maxX) / 2)
const centerZ = computed(() => (props.bounds.minZ + props.bounds.maxZ) / 2)
const areaKm = computed(() => {
  const width = (props.bounds.maxX - props.bounds.minX) / 1000
  const height = (props.bounds.maxZ - props.bounds.minZ) / 1000
  return Math.abs(width * height).toFixed(1)
})

const rows = computed(() => {
  const counts = new Map<string, number>()
  const nearest = new Map<string, MapFeature>()
  for (const feature of props.features) {
    counts.set(feature.kind, (counts.get(feature.kind) ?? 0) + 1)
    const current = nearest.get(feature.kind)
    const distance = (item: MapFeature) =>
      (item.x - centerX.value) ** 2 + (item.z - centerZ.value) ** 2
    if (!current || distance(feature) < distance(current)) nearest.set(feature.kind, feature)
  }
  return [...counts.entries()]
    .map(([kind, count]) => {
      const feature = nearest.get(kind)!
      const dx = feature.x - centerX.value
      const dz = feature.z - centerZ.value
      return {
        kind,
        count,
        label: featureMeta(kind).label,
        color: featureMeta(kind).color,
        distance: Math.round(Math.hypot(dx, dz)),
        nearest: `${feature.x} , ${feature.z}`,
      }
    })
    .sort((a, b) => a.distance - b.distance)
})

const total = computed(() => props.features.length)

const biomeTotal = computed(() => biomes.value.reduce((sum, item) => sum + item.count, 0))

async function sampleBiomes() {
  if (sampling.value) return
  sampling.value = true
  progress.value = 0
  biomes.value = []
  const cols = 16
  const rowsCount = 10
  const points: { x: number; z: number }[] = []
  for (let row = 0; row < rowsCount; row += 1) {
    for (let col = 0; col < cols; col += 1) {
      points.push({
        x: props.bounds.minX + ((col + 0.5) / cols) * (props.bounds.maxX - props.bounds.minX),
        z: props.bounds.minZ + ((row + 0.5) / rowsCount) * (props.bounds.maxZ - props.bounds.minZ),
      })
    }
  }

  const counts = new Map<number, number>()
  let done = 0
  const queue = points.slice()
  const worker = async () => {
    while (queue.length) {
      const point = queue.shift()!
      try {
        const id = await biomeAt(
          props.seed,
          props.edition,
          props.version,
          props.dimension,
          Math.round(point.x),
          64,
          Math.round(point.z),
        )
        counts.set(id, (counts.get(id) ?? 0) + 1)
      } catch {
        // A failed probe just leaves that cell out of the histogram.
      }
      done += 1
      progress.value = Math.round((done / points.length) * 100)
    }
  }
  await Promise.all(Array.from({ length: 6 }, () => worker()))
  biomes.value = [...counts.entries()]
    .map(([id, count]) => ({ id, count }))
    .sort((a, b) => b.count - a.count)
  sampling.value = false
}
</script>

<template>
  <section class="panel">
    <header>
      <div>
        <b>区域分析</b>
        <span class="sub">{{ areaKm }} km² · {{ total }} 个结构</span>
      </div>
      <button class="close" aria-label="关闭" @click="emit('close')">×</button>
    </header>

    <div class="body">
      <div class="section">
        <h4>结构统计（当前视野）</h4>
        <div v-if="!rows.length" class="hint">视野内没有已开启的结构标记。</div>
        <div v-for="row in rows" :key="row.kind" class="row">
          <i :style="{ background: row.color }" />
          <span>{{ row.label }}</span>
          <em>×{{ row.count }}</em>
          <b>{{ row.distance }} 格</b>
        </div>
      </div>

      <div class="section">
        <div class="section-head">
          <h4>群系抽样</h4>
          <button class="mini" :disabled="sampling" @click="sampleBiomes">
            {{ sampling ? `${progress}%` : '抽样 160 点' }}
          </button>
        </div>
        <div v-if="!biomes.length" class="hint">
          按 16×10 网格采样视野内的生物群系，得到一个大致构成。
        </div>
        <div v-for="item in biomes" :key="item.id" class="biome">
          <span>{{ biomeName(item.id) }}</span>
          <div class="bar">
            <i :style="{ width: `${Math.round((item.count / biomeTotal) * 100)}%` }" />
          </div>
          <em>{{ Math.round((item.count / biomeTotal) * 100) }}%</em>
        </div>
      </div>
    </div>
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

.body {
  padding: 10px 12px;
  overflow-y: auto;
}

.section {
  margin-bottom: 14px;
}

.section-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

h4 {
  margin: 0 0 6px;
  font-size: 11px;
  font-weight: 600;
  color: var(--muted);
  letter-spacing: 0.4px;
}

.mini {
  height: 24px;
  padding: 0 8px;
  border-radius: 6px;
  background: var(--panel-2);
  border: 1px solid var(--border);
  font-size: 10.5px;
  color: var(--text);
}

.mini:disabled {
  opacity: 0.6;
}

.row {
  display: flex;
  align-items: center;
  gap: 7px;
  padding: 4px 2px;
  font-size: 11.5px;
}

.row i {
  width: 8px;
  height: 8px;
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
  color: var(--muted);
  font-variant-numeric: tabular-nums;
}

.row b {
  width: 62px;
  text-align: right;
  font-weight: 500;
  font-size: 10.5px;
  color: #7b7b86;
  font-variant-numeric: tabular-nums;
}

.biome {
  display: flex;
  align-items: center;
  gap: 7px;
  padding: 3px 0;
  font-size: 11px;
}

.biome span {
  width: 84px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.bar {
  flex: 1;
  height: 5px;
  border-radius: 3px;
  background: #26262d;
  overflow: hidden;
}

.bar i {
  display: block;
  height: 100%;
  background: linear-gradient(90deg, #6aa84f, #8bc34a);
}

.biome em {
  width: 34px;
  text-align: right;
  font-style: normal;
  color: var(--muted);
  font-variant-numeric: tabular-nums;
}

.hint {
  margin: 0;
  font-size: 11px;
  line-height: 1.6;
  color: var(--muted);
}
</style>
