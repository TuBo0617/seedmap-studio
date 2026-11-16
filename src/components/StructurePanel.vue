<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import {
  containerLoot,
  structureDetails,
  type Dimension,
  type Edition,
  type MapFeature,
  type StructureContainer,
  type StructureDetails,
  type StructureLoot,
} from '../lib/api'
import { FEATURE_BITS, featureMeta } from '../lib/features'

const props = defineProps<{
  feature: MapFeature
  seed: string
  edition: Edition
  version: string
  dimension: Dimension
}>()

const emit = defineEmits<{ (event: 'close'): void }>()

const details = ref<StructureDetails | null>(null)
const loading = ref(false)
const error = ref<string | null>(null)
const lootFor = ref<number | null>(null)
const loot = ref<StructureLoot | null>(null)
const lootLoading = ref(false)
const canvas = ref<HTMLCanvasElement | null>(null)
const showAllPieces = ref(false)

const meta = computed(() => featureMeta(props.feature.kind))
const mask = computed(() => FEATURE_BITS[props.feature.kind] ?? 0)

const query = computed(() => ({
  seed: props.seed,
  edition: props.edition,
  version: props.version,
  dimension: props.dimension,
  x: props.feature.x,
  z: props.feature.z,
  featureMask: mask.value,
}))

const variantFacts = computed(() => {
  const variant = details.value?.variant
  if (!variant) return []
  const rotation = ['南', '西', '北', '东'][variant.rotation & 3] ?? '?'
  const facts: { label: string; value: string }[] = [
    { label: '朝向', value: rotation },
  ]
  if (variant.biome >= 0) facts.push({ label: '群系 id', value: String(variant.biome) })
  if (variant.size) facts.push({ label: '规模', value: String(variant.size) })
  if (variant.abandoned) facts.push({ label: '状态', value: '已被废弃' })
  if (variant.giant) facts.push({ label: '状态', value: '巨型变体' })
  if (variant.underground) facts.push({ label: '位置', value: '地下' })
  if (variant.airPocket) facts.push({ label: '结构', value: '含气室' })
  if (variant.basement) facts.push({ label: '结构', value: '带地下室' })
  if (variant.cracked) facts.push({ label: '状态', value: '有裂纹' })
  return facts
})

const pieces = computed(() => {
  const list = details.value?.pieces ?? []
  return showAllPieces.value ? list : list.slice(0, 40)
})

const pieceName = (name: string) => {
  const tail = name.includes('/') ? name.slice(name.lastIndexOf('/') + 1) : name
  return tail.replaceAll('_', ' ')
}

async function load() {
  if (!mask.value) {
    error.value = '这个标记不支持结构详情。'
    return
  }
  loading.value = true
  error.value = null
  loot.value = null
  lootFor.value = null
  try {
    details.value = await structureDetails(query.value)
  } catch (reason) {
    error.value = String(reason)
    details.value = null
  } finally {
    loading.value = false
  }
  // The canvas only exists once `loading` is false, so draw after the flip.
  await nextTick()
  drawLayout()
}

function drawLayout() {
  const el = canvas.value
  const piecesList = details.value?.pieces ?? []
  if (!el || !piecesList.length) return
  const ctx = el.getContext('2d')
  if (!ctx) return

  let minX = Infinity
  let minZ = Infinity
  let maxX = -Infinity
  let maxZ = -Infinity
  for (const piece of piecesList) {
    const ax = piece.min?.x ?? piece.position.x
    const az = piece.min?.z ?? piece.position.z
    const bx = piece.max?.x ?? piece.position.x
    const bz = piece.max?.z ?? piece.position.z
    minX = Math.min(minX, ax, bx)
    minZ = Math.min(minZ, az, bz)
    maxX = Math.max(maxX, ax, bx)
    maxZ = Math.max(maxZ, az, bz)
  }
  if (!Number.isFinite(minX)) return

  const pad = 10
  const w = el.width
  const h = el.height
  const spanX = Math.max(16, maxX - minX + 1)
  const spanZ = Math.max(16, maxZ - minZ + 1)
  const k = Math.min((w - pad * 2) / spanX, (h - pad * 2) / spanZ)

  ctx.clearRect(0, 0, w, h)
  ctx.fillStyle = '#101014'
  ctx.fillRect(0, 0, w, h)

  for (const piece of piecesList) {
    const ax = piece.min?.x ?? piece.position.x
    const az = piece.min?.z ?? piece.position.z
    const bx = (piece.max?.x ?? piece.position.x) + 1
    const bz = (piece.max?.z ?? piece.position.z) + 1
    const x = pad + (ax - minX) * k
    const z = pad + (az - minZ) * k
    ctx.fillStyle = piece.endPortalRoom
      ? 'rgba(120, 220, 160, 0.85)'
      : piece.portalEyes
        ? 'rgba(255, 190, 90, 0.85)'
        : 'rgba(255, 119, 185, 0.42)'
    ctx.fillRect(x, z, Math.max(1.5, (bx - ax) * k), Math.max(1.5, (bz - az) * k))
    ctx.strokeStyle = 'rgba(12, 12, 16, 0.7)'
    ctx.lineWidth = 0.6
    ctx.strokeRect(x, z, Math.max(1.5, (bx - ax) * k), Math.max(1.5, (bz - az) * k))
  }
}

async function openLoot(container: StructureContainer) {
  if (lootFor.value === container.id) {
    lootFor.value = null
    loot.value = null
    return
  }
  lootFor.value = container.id
  lootLoading.value = true
  loot.value = null
  try {
    loot.value = await containerLoot(query.value, container.id)
  } catch (reason) {
    error.value = String(reason)
  } finally {
    lootLoading.value = false
  }
}

function itemName(id: string) {
  const tail = id.includes(':') ? id.slice(id.indexOf(':') + 1) : id
  return tail.replaceAll('_', ' ')
}

async function copyCoordinates() {
  const text = `${props.feature.x} ${props.feature.z}`
  try {
    await navigator.clipboard.writeText(text)
  } catch {
    // Clipboard access can be denied; the coordinates stay readable above.
  }
}

watch(
  () => [props.feature.x, props.feature.z, props.feature.kind, props.version, props.dimension],
  () => void load(),
  { immediate: true },
)
</script>

<template>
  <section class="panel">
    <header>
      <div class="title">
        <i :style="{ background: meta.color }" />
        <b>{{ meta.label }}</b>
        <span>X {{ feature.x }} · Z {{ feature.z }}</span>
      </div>
      <button class="close" aria-label="关闭" @click="emit('close')">×</button>
    </header>

    <div class="body">
      <p v-if="loading" class="hint">正在计算结构布局…</p>
      <p v-else-if="error" class="hint error">{{ error }}</p>

      <template v-else-if="details">
        <div class="section">
          <canvas ref="canvas" class="layout" width="300" height="180" />
          <p class="hint small">
            {{ details.pieces.length }} 段结构 · 俯视投影
            <span v-if="details.truncated">（结果被截断）</span>
          </p>
        </div>

        <div v-if="variantFacts.length" class="facts">
          <div v-for="fact in variantFacts" :key="fact.label">
            <span>{{ fact.label }}</span>
            <b>{{ fact.value }}</b>
          </div>
        </div>

        <div v-if="details.pieces.length" class="section">
          <h4>结构分层</h4>
          <div class="pieces">
            <div v-for="piece in pieces" :key="`${piece.id}-${piece.name}`">
              <span>{{ pieceName(piece.name) }}</span>
              <em>{{ piece.position.x }} , {{ piece.position.y }} , {{ piece.position.z }}</em>
            </div>
          </div>
          <button
            v-if="details.pieces.length > 40"
            class="more"
            @click="showAllPieces = !showAllPieces"
          >
            {{ showAllPieces ? '收起' : `展开全部 ${details.pieces.length} 段` }}
          </button>
        </div>

        <div v-if="details.containers.length" class="section">
          <h4>容器 · {{ details.containers.length }}</h4>
          <div class="containers">
            <div v-for="container in details.containers" :key="container.id">
              <button class="container" @click="openLoot(container)">
                <span>#{{ container.id }} {{ container.table ?? '战利品' }}</span>
                <em>{{ container.x }} , {{ container.z }}</em>
              </button>
              <div v-if="lootFor === container.id" class="loot">
                <p v-if="lootLoading" class="hint small">读取战利品表…</p>
                <template v-else-if="loot">
                  <p v-if="!loot.items?.length" class="hint small">这个容器没有可推断的物品。</p>
                  <div v-for="(item, index) in loot.items" :key="index" class="item">
                    <span>{{ itemName(item.id) }}</span>
                    <em>×{{ item.count }}</em>
                    <i v-if="item.enchantments?.length">
                      {{ item.enchantments.map((e) => `${itemName(e.id)} ${e.level}`).join(' · ') }}
                    </i>
                  </div>
                  <p v-if="loot.limits?.length" class="hint tiny">
                    限制：{{ loot.limits.join(' / ') }}
                  </p>
                </template>
              </div>
            </div>
          </div>
        </div>

        <p v-if="details.limits?.length" class="hint tiny">
          说明：{{ details.limits.join(' / ') }}
        </p>
      </template>
    </div>

    <footer>
      <button class="btn" @click="copyCoordinates">复制坐标</button>
      <button class="btn ghost" @click="emit('close')">关闭</button>
    </footer>
  </section>
</template>

<style scoped>
.panel {
  display: flex;
  flex-direction: column;
  width: 330px;
  max-height: min(78vh, 720px);
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: 12px;
  box-shadow: 0 18px 44px rgba(0, 0, 0, 0.55);
  overflow: hidden;
}

header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 10px 12px;
  border-bottom: 1px solid var(--border);
}

.title {
  display: flex;
  align-items: center;
  gap: 7px;
  min-width: 0;
}

.title i {
  width: 9px;
  height: 9px;
  border-radius: 3px;
}

.title b {
  font-size: 12.5px;
}

.title span {
  font-size: 10.5px;
  color: var(--muted);
  font-variant-numeric: tabular-nums;
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
  margin-bottom: 12px;
}

.layout {
  width: 100%;
  height: auto;
  border-radius: 8px;
  border: 1px solid var(--border);
  background: #101014;
}

.hint {
  margin: 0;
  font-size: 11.5px;
  color: var(--muted);
  line-height: 1.6;
}

.hint.small {
  font-size: 10.5px;
  margin-top: 6px;
}

.hint.tiny {
  font-size: 10px;
  color: #63636e;
}

.hint.error {
  color: #ff8a80;
}

.facts {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 6px;
  margin-bottom: 12px;
}

.facts div {
  display: flex;
  justify-content: space-between;
  gap: 6px;
  padding: 6px 8px;
  background: var(--panel-2);
  border: 1px solid var(--border);
  border-radius: 7px;
  font-size: 11px;
  color: var(--muted);
}

.facts b {
  color: var(--text);
  font-weight: 600;
}

h4 {
  margin: 0 0 6px;
  font-size: 11px;
  font-weight: 600;
  color: var(--muted);
  letter-spacing: 0.4px;
}

.pieces {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.pieces div {
  display: flex;
  justify-content: space-between;
  gap: 8px;
  padding: 4px 7px;
  border-radius: 6px;
  font-size: 11px;
  background: rgba(255, 255, 255, 0.02);
}

.pieces span {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.pieces em {
  font-style: normal;
  color: var(--muted);
  font-variant-numeric: tabular-nums;
  flex: 0 0 auto;
}

.more {
  margin-top: 6px;
  font-size: 10.5px;
  color: var(--accent);
}

.containers {
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.container {
  display: flex;
  justify-content: space-between;
  gap: 8px;
  width: 100%;
  padding: 6px 8px;
  border-radius: 7px;
  background: var(--panel-2);
  border: 1px solid var(--border);
  font-size: 11px;
  text-align: left;
}

.container em {
  font-style: normal;
  color: var(--muted);
  font-variant-numeric: tabular-nums;
}

.loot {
  padding: 6px 8px 8px;
}

.item {
  display: flex;
  align-items: baseline;
  gap: 6px;
  font-size: 11px;
  padding: 2px 0;
}

.item span {
  flex: 1;
}

.item em {
  font-style: normal;
  color: var(--muted);
}

.item i {
  font-style: normal;
  font-size: 10px;
  color: #9c8bd0;
}

footer {
  display: flex;
  gap: 6px;
  padding: 10px 12px;
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
  padding: 0 14px;
  color: var(--muted);
}
</style>
