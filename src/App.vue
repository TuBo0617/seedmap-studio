<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, reactive, ref, watch } from 'vue'
import {
  fetchProfiles,
  findFeatures,
  findSpawn,
  renderTile,
  type Dimension,
  type MapFeature,
  type VersionProfile,
} from './lib/api'
import { DEFAULT_KINDS, FEATURE_BITS, featureMeta, LEGEND_ORDER } from './lib/features'

const seedInput = ref('10292992')
const seedValue = ref('10292992')
const version = ref('26.3')
const dimension = ref<Dimension>('overworld')

const profiles = ref<VersionProfile[]>([])
const features = ref<MapFeature[]>([])
const activeKinds = ref<Set<string>>(new Set(DEFAULT_KINDS))
const SCALES = [1, 4, 16, 64, 256]
const TILE = 256
const scale = ref(4)
const center = reactive({ x: 0, z: 0 })
const viewport = reactive({ w: 1280, h: 720 })

const canvasRef = ref<HTMLCanvasElement | null>(null)
const overlayRef = ref<HTMLCanvasElement | null>(null)
const mapWrap = ref<HTMLDivElement | null>(null)

const busy = ref(false)
const hasPainted = ref(false)
const lastMs = ref(0)
const error = ref<string | null>(null)
const showLegend = ref(true)
const dragging = ref(false)
const dragOffset = reactive({ x: 0, z: 0 })
const hover = reactive({ x: 0, z: 0 })

let renderSeq = 0
let origin = { x: 0, z: 0, scale: 4, w: 1, h: 1 }
let dragStart: { x: number; y: number; cx: number; cz: number } | null = null
let observer: ResizeObserver | null = null

const javaProfiles = computed(() =>
  profiles.value.filter((profile) => profile.edition === 'java'),
)
const versions = computed(() => javaProfiles.value.map((profile) => profile.version))
const currentProfile = computed(
  () =>
    javaProfiles.value.find((profile) => profile.version === version.value) ??
    null,
)
const engineReady = computed(() => currentProfile.value?.available !== false)
const dimensions = computed<Dimension[]>(() => {
  const list = currentProfile.value?.dimensions
  return list && list.length ? list : ['overworld']
})

const DIM_LABEL: Record<Dimension, string> = {
  overworld: '主世界',
  nether: '下界',
  end: '末地',
}

const scaleLabel = computed(() =>
  scale.value === 1 ? '1 : 1' : `1 : ${scale.value}`,
)

const dimensionIndex = computed(() =>
  dimension.value === 'overworld' ? 0 : dimension.value === 'nether' ? 1 : 2,
)

const allowedMask = computed(() => {
  const masks = currentProfile.value?.featureMasks
  return masks?.[dimensionIndex.value] ?? 0x3ffffff
})

const featureMask = computed(() => {
  let mask = 0
  for (const kind of activeKinds.value) mask |= FEATURE_BITS[kind] ?? 0
  return mask & allowedMask.value
})

const legendItems = computed(() => {
  const counts = new Map<string, number>()
  for (const feature of features.value) {
    counts.set(feature.kind, (counts.get(feature.kind) ?? 0) + 1)
  }
  return LEGEND_ORDER.filter(
    (kind) => ((FEATURE_BITS[kind] ?? 0) & allowedMask.value) !== 0,
  ).map((kind) => ({
    kind,
    count: counts.get(kind) ?? 0,
    active: activeKinds.value.has(kind),
    ...featureMeta(kind),
  }))
})

function toggleKind(kind: string) {
  const next = new Set(activeKinds.value)
  if (next.has(kind)) next.delete(kind)
  else next.add(kind)
  activeKinds.value = next
  void loadFeatures(renderSeq)
}

function selectAllKinds(on: boolean) {
  activeKinds.value = on ? new Set(LEGEND_ORDER) : new Set()
  void loadFeatures(renderSeq)
}

watch(version, () => {
  if (!dimensions.value.includes(dimension.value)) {
    dimension.value = 'overworld'
  }
  void repaint()
})

watch(dimension, () => {
  void repaint()
})

function drawTile(
  buffer: ArrayBuffer,
  tw: number,
  th: number,
  dx: number,
  dy: number,
) {
  const canvas = canvasRef.value
  const ctx = canvas?.getContext('2d')
  if (!canvas || !ctx) return
  const image = new ImageData(new Uint8ClampedArray(buffer), tw, th)
  const off = document.createElement('canvas')
  off.width = tw
  off.height = th
  off.getContext('2d')?.putImageData(image, 0, 0)
  ctx.drawImage(off, dx, dy)
}

async function repaint() {
  const canvas = canvasRef.value
  if (!canvas) return
  const ctx = canvas.getContext('2d')
  if (!ctx) return

  const w = Math.max(320, Math.floor(viewport.w))
  const h = Math.max(240, Math.floor(viewport.h))
  if (canvas.width !== w) canvas.width = w
  if (canvas.height !== h) canvas.height = h

  const overlay = overlayRef.value
  if (overlay && (overlay.width !== w || overlay.height !== h)) {
    overlay.width = w
    overlay.height = h
  }

  const seq = ++renderSeq
  const s = scale.value
  // 对齐到 scale 的整数倍，保证相邻瓦片严丝合缝
  const left = Math.floor((center.x - (w / 2) * s) / s) * s
  const top = Math.floor((center.z - (h / 2) * s) / s) * s
  origin = { x: left, z: top, scale: s, w, h }

  busy.value = true
  error.value = null
  ctx.fillStyle = '#0e0e10'
  ctx.fillRect(0, 0, w, h)

  const started = performance.now()
  const cols = Math.ceil(w / TILE)
  const rows = Math.ceil(h / TILE)
  let failure: string | null = null

  const jobs: Promise<void>[] = []
  for (let row = 0; row < rows; row += 1) {
    for (let col = 0; col < cols; col += 1) {
      const tw = Math.min(TILE, w - col * TILE)
      const th = Math.min(TILE, h - row * TILE)
      jobs.push(
        renderTile({
          seed: seedValue.value,
          edition: 'java',
          version: version.value,
          dimension: dimension.value,
          x: left + col * TILE * s,
          z: top + row * TILE * s,
          scale: s,
          width: tw,
          height: th,
          elevation: 128,
          terrain: false,
          contours: false,
          highlightBiomes: null,
        })
          .then((buffer) => {
            if (seq !== renderSeq) return
            drawTile(buffer, tw, th, col * TILE, row * TILE)
          })
          .catch((reason) => {
            failure = String(reason)
          }),
      )
    }
  }

  await Promise.all(jobs)
  if (seq !== renderSeq) return

  busy.value = false
  hasPainted.value = true
  lastMs.value = Math.round(performance.now() - started)
  if (failure) error.value = failure
  drawMarkers()
  void loadFeatures(seq)
}

async function loadFeatures(seq: number) {
  const halfW = (origin.w / 2) * origin.scale
  const halfH = (origin.h / 2) * origin.scale
  try {
    const list = await findFeatures({
      seed: seedValue.value,
      edition: 'java',
      version: version.value,
      dimension: dimension.value,
      minX: Math.round(center.x - halfW),
      minZ: Math.round(center.z - halfH),
      maxX: Math.round(center.x + halfW),
      maxZ: Math.round(center.z + halfH),
      featureMask: featureMask.value || null,
    })
    if (seq !== renderSeq) return
    features.value = list
    drawMarkers()
  } catch {
    features.value = []
  }
}

function drawMarkers() {
  const overlay = overlayRef.value
  if (!overlay) return
  const ctx = overlay.getContext('2d')
  if (!ctx) return
  ctx.clearRect(0, 0, overlay.width, overlay.height)

  const r = Math.max(3.5, Math.min(7, 9 / Math.sqrt(origin.scale)))
  for (const feature of features.value) {
    if (!activeKinds.value.has(feature.kind)) continue
    const px = (feature.x - origin.x) / origin.scale
    const pz = (feature.z - origin.z) / origin.scale
    if (
      px < -14 ||
      pz < -14 ||
      px > overlay.width + 14 ||
      pz > overlay.height + 14
    ) {
      continue
    }
    const meta = featureMeta(feature.kind)
    ctx.beginPath()
    ctx.arc(px, pz, r, 0, Math.PI * 2)
    ctx.fillStyle = meta.color
    ctx.fill()
    ctx.lineWidth = 2
    ctx.strokeStyle = 'rgba(12,12,14,0.85)'
    ctx.stroke()
  }
}

function applySeed() {
  seedValue.value = seedInput.value.trim() || '0'
  void jumpToSpawn()
}

async function jumpToSpawn() {
  try {
    const spawn = await findSpawn(seedValue.value, 'java', version.value)
    center.x = spawn.x
    center.z = spawn.z
    await repaint()
  } catch (reason) {
    error.value = String(reason)
  }
}

function stepScale(direction: 1 | -1) {
  const index = SCALES.indexOf(scale.value)
  const next = Math.min(
    SCALES.length - 1,
    Math.max(0, (index < 0 ? 1 : index) + direction),
  )
  if (SCALES[next] === scale.value) return
  scale.value = SCALES[next]
  void repaint()
}

function resetView() {
  scale.value = 4
  center.x = 0
  center.z = 0
  void repaint()
}

function onWheel(event: WheelEvent) {
  event.preventDefault()
  const canvas = canvasRef.value
  if (!canvas) return
  const index = SCALES.indexOf(scale.value)
  const nextIndex = Math.min(
    SCALES.length - 1,
    Math.max(0, (index < 0 ? 1 : index) + (event.deltaY < 0 ? -1 : 1)),
  )
  if (nextIndex === index) return
  const next = SCALES[nextIndex]
  const rect = canvas.getBoundingClientRect()
  const mx = event.clientX - rect.left
  const my = event.clientY - rect.top
  const worldX = center.x + (mx - origin.w / 2) * scale.value
  const worldZ = center.z + (my - origin.h / 2) * scale.value
  scale.value = next
  center.x = worldX - (mx - origin.w / 2) * next
  center.z = worldZ - (my - origin.h / 2) * next
  void repaint()
}

function onPointerDown(event: PointerEvent) {
  if (event.button !== 0) return
  dragStart = {
    x: event.clientX,
    y: event.clientY,
    cx: center.x,
    cz: center.z,
  }
  dragging.value = true
  canvasRef.value?.setPointerCapture(event.pointerId)
}

function onPointerMove(event: PointerEvent) {
  const canvas = canvasRef.value
  if (canvas) {
    const rect = canvas.getBoundingClientRect()
    hover.x = Math.round(
      center.x + (event.clientX - rect.left - origin.w / 2) * scale.value,
    )
    hover.z = Math.round(
      center.z + (event.clientY - rect.top - origin.h / 2) * scale.value,
    )
  }
  if (!dragStart) return
  dragOffset.x = event.clientX - dragStart.x
  dragOffset.z = event.clientY - dragStart.y
  center.x = dragStart.cx - dragOffset.x * scale.value
  center.z = dragStart.cz - dragOffset.z * scale.value
}

function onPointerUp() {
  if (!dragStart) return
  dragStart = null
  dragging.value = false
  dragOffset.x = 0
  dragOffset.z = 0
  void repaint()
}

onMounted(async () => {
  try {
    profiles.value = await fetchProfiles()
    const first = javaProfiles.value.find((profile) => profile.available)
    if (first) version.value = first.version
  } catch (reason) {
    error.value = String(reason)
  }

  if (mapWrap.value) {
    observer = new ResizeObserver((entries) => {
      const box = entries[0]?.contentRect
      if (!box) return
      viewport.w = box.width
      viewport.h = box.height
    })
    observer.observe(mapWrap.value)
    viewport.w = mapWrap.value.clientWidth
    viewport.h = mapWrap.value.clientHeight
  }

  await jumpToSpawn()
})

onBeforeUnmount(() => observer?.disconnect())
</script>

<template>
  <div class="app">
    <header class="topbar">
      <div class="brand">
        <div class="logo" />
        <div class="brand-text">
          <div class="name">Seedmap Studio</div>
          <div class="sub">Minecraft 种子地图</div>
        </div>
      </div>

      <div class="controls">
        <label class="field seed">
          <span>种子</span>
          <input
            v-model="seedInput"
            spellcheck="false"
            @keydown.enter="applySeed"
          />
        </label>
        <button class="btn primary" @click="applySeed">生成</button>

        <label class="field">
          <span>版本</span>
          <select v-model="version">
            <option v-for="item in versions" :key="item" :value="item">
              {{ item }}
            </option>
          </select>
        </label>

        <div class="segmented">
          <button
            v-for="item in dimensions"
            :key="item"
            :class="{ active: dimension === item }"
            @click="dimension = item"
          >
            {{ DIM_LABEL[item] }}
          </button>
        </div>
      </div>

      <div class="engine" :class="{ warn: !engineReady }">
        {{ engineReady ? 'cubiomes 引擎就绪' : '引擎不可用' }}
      </div>
    </header>

    <main ref="mapWrap" class="map-area">
      <canvas
        ref="canvasRef"
        class="layer"
        :style="{ transform: `translate(${dragOffset.x}px, ${dragOffset.z}px)` }"
        @wheel.prevent="onWheel"
        @pointerdown="onPointerDown"
        @pointermove="onPointerMove"
        @pointerup="onPointerUp"
        @pointercancel="onPointerUp"
      />
      <canvas
        ref="overlayRef"
        class="layer overlay"
        :style="{ transform: `translate(${dragOffset.x}px, ${dragOffset.z}px)` }"
      />

      <div class="zoom">
        <button title="放大" @click="stepScale(-1)">＋</button>
        <div class="zoom-label">{{ scaleLabel }}</div>
        <button title="缩小" @click="stepScale(1)">－</button>
        <button class="home" title="回到出生点" @click="jumpToSpawn">⌂</button>
      </div>

      <section v-if="showLegend" class="legend">
        <header>
          <span>结构 · {{ features.length }}</span>
          <button @click="showLegend = false">×</button>
        </header>
        <div class="legend-actions">
          <button @click="selectAllKinds(true)">全部</button>
          <button @click="selectAllKinds(false)">清空</button>
        </div>
        <div class="legend-list">
          <div
            v-for="item in legendItems"
            :key="item.kind"
            :class="{ off: !item.active }"
            @click="toggleKind(item.kind)"
          >
            <i :style="{ background: item.active ? item.color : '#4a4a52' }" />
            <span>{{ item.label }}</span>
            <b v-if="item.active">{{ item.count }}</b>
          </div>
        </div>
      </section>
      <button v-else class="legend-toggle" @click="showLegend = true">
        图例
      </button>

      <footer class="hud">
        <span>X {{ Math.round(center.x) }} · Z {{ Math.round(center.z) }}</span>
        <span class="dim">光标 {{ hover.x }} , {{ hover.z }}</span>
        <span>1 像素 = {{ scale }} 格</span>
        <span>{{ lastMs }} ms</span>
        <span v-if="busy" class="busy">渲染中…</span>
      </footer>

      <div v-if="busy && !hasPainted" class="splash">
        <div class="spinner" />
        <p>正在生成地形…</p>
      </div>

      <div v-if="error" class="error">{{ error }}</div>
    </main>
  </div>
</template>

<style scoped>
.app {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: var(--bg);
}

.topbar {
  display: flex;
  align-items: center;
  gap: 18px;
  height: 58px;
  padding: 0 14px 0 16px;
  background: var(--panel);
  border-bottom: 1px solid var(--border);
  flex: 0 0 auto;
}

.brand {
  display: flex;
  align-items: center;
  gap: 10px;
}

.logo {
  width: 30px;
  height: 30px;
  border-radius: 8px;
  background:
    linear-gradient(135deg, #6aa84f 0%, #4d7f3d 55%, #3b6330 100%);
  box-shadow:
    inset 0 0 0 1px rgba(255, 255, 255, 0.14),
    0 3px 10px rgba(0, 0, 0, 0.45);
  position: relative;
  overflow: hidden;
}

.logo::after {
  content: '';
  position: absolute;
  inset: 6px;
  border-radius: 4px;
  background:
    linear-gradient(90deg, transparent 42%, #4d84ba 42%, #4d84ba 58%, transparent 58%);
  opacity: 0.9;
}

.brand-text .name {
  font-size: 13.5px;
  font-weight: 650;
  letter-spacing: 0.2px;
}

.brand-text .sub {
  font-size: 10.5px;
  color: var(--muted);
  margin-top: 1px;
}

.controls {
  display: flex;
  align-items: center;
  gap: 10px;
  flex: 1;
  min-width: 0;
}

.field {
  display: flex;
  align-items: center;
  gap: 7px;
  height: 32px;
  padding: 0 10px;
  background: var(--panel-2);
  border: 1px solid var(--border);
  border-radius: 8px;
  font-size: 11.5px;
  color: var(--muted);
}

.field.seed input {
  width: 132px;
  border: none;
  outline: none;
  background: transparent;
  font-size: 12.5px;
  font-variant-numeric: tabular-nums;
}

.field select {
  border: none;
  outline: none;
  background: transparent;
  font-size: 12.5px;
  padding-right: 4px;
}

.btn {
  height: 32px;
  padding: 0 14px;
  border-radius: 8px;
  font-size: 12.5px;
  font-weight: 560;
}

.btn.primary {
  background: var(--accent);
  color: #1a1016;
  box-shadow: 0 4px 14px rgba(255, 119, 185, 0.25);
}

.btn.primary:hover {
  filter: brightness(1.06);
}

.segmented {
  display: flex;
  padding: 3px;
  gap: 2px;
  background: var(--panel-2);
  border: 1px solid var(--border);
  border-radius: 9px;
}

.segmented button {
  height: 26px;
  padding: 0 12px;
  border-radius: 6px;
  font-size: 12px;
  color: var(--muted);
}

.segmented button.active {
  background: var(--accent-soft);
  color: #ff9ecb;
}

.engine {
  margin-left: auto;
  font-size: 11px;
  color: #7fbf93;
  background: rgba(90, 190, 130, 0.1);
  border: 1px solid rgba(90, 190, 130, 0.22);
  border-radius: 999px;
  padding: 4px 10px;
  white-space: nowrap;
}

.engine.warn {
  color: #e3a06b;
  background: rgba(227, 160, 107, 0.1);
  border-color: rgba(227, 160, 107, 0.25);
}

.map-area {
  position: relative;
  flex: 1;
  min-height: 0;
  overflow: hidden;
  background: #0e0e10;
}

canvas.layer {
  position: absolute;
  inset: 0;
  display: block;
  width: 100%;
  height: 100%;
  image-rendering: pixelated;
  touch-action: none;
}

canvas.layer:first-of-type {
  cursor: grab;
}

canvas.layer:first-of-type:active {
  cursor: grabbing;
}

canvas.overlay {
  pointer-events: none;
}

.legend-actions {
  display: flex;
  gap: 4px;
  padding: 6px 10px 0;
}

.legend-actions button {
  flex: 1;
  height: 22px;
  font-size: 10.5px;
  border-radius: 6px;
  background: var(--panel-2);
  color: var(--muted);
}

.legend-actions button:hover {
  color: var(--text);
  background: #2c2c34;
}

.legend-list div.off {
  opacity: 0.42;
}

.legend-list div.off span {
  color: var(--muted);
}

.zoom {
  position: absolute;
  left: 14px;
  top: 14px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  padding: 6px;
  background: rgba(24, 24, 28, 0.86);
  border: 1px solid var(--border);
  border-radius: 11px;
  backdrop-filter: blur(14px);
}

.zoom button {
  width: 30px;
  height: 30px;
  border-radius: 7px;
  background: var(--panel-2);
  font-size: 14px;
  line-height: 1;
}

.zoom button:hover {
  background: #2c2c34;
}

.zoom .zoom-label {
  font-size: 10px;
  color: var(--muted);
  font-variant-numeric: tabular-nums;
}

.zoom .home {
  margin-top: 2px;
  font-size: 13px;
}

.legend {
  position: absolute;
  left: 14px;
  bottom: 46px;
  width: 196px;
  max-height: 46%;
  display: flex;
  flex-direction: column;
  background: rgba(24, 24, 28, 0.9);
  border: 1px solid var(--border);
  border-radius: 12px;
  backdrop-filter: blur(16px);
  overflow: hidden;
}

.legend header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 9px 10px 8px 12px;
  font-size: 11.5px;
  color: var(--muted);
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
}

.legend header button {
  color: var(--muted);
  font-size: 14px;
  line-height: 1;
  padding: 2px 4px;
}

.legend-list {
  overflow-y: auto;
  padding: 6px 6px 8px;
}

.legend-list div {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 6px;
  font-size: 11.5px;
  border-radius: 6px;
}

.legend-list div:hover {
  background: rgba(255, 255, 255, 0.04);
}

.legend-list i {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex: 0 0 auto;
  box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.5);
}

.legend-list span {
  flex: 1;
  color: #c9c9d2;
}

.legend-list b {
  color: var(--muted);
  font-weight: 500;
  font-variant-numeric: tabular-nums;
}

.legend-list .empty {
  color: var(--muted);
  font-size: 11px;
  padding: 8px;
}

.legend-toggle {
  position: absolute;
  left: 14px;
  bottom: 46px;
  padding: 7px 12px;
  font-size: 11.5px;
  border-radius: 9px;
  background: rgba(24, 24, 28, 0.9);
  border: 1px solid var(--border);
  color: var(--muted);
  backdrop-filter: blur(14px);
}

.hud {
  position: absolute;
  left: 0;
  right: 0;
  bottom: 0;
  display: flex;
  align-items: center;
  gap: 18px;
  height: 30px;
  padding: 0 14px;
  font-size: 11px;
  color: var(--muted);
  background: rgba(20, 20, 22, 0.88);
  border-top: 1px solid rgba(255, 255, 255, 0.05);
  font-variant-numeric: tabular-nums;
  backdrop-filter: blur(10px);
}

.hud .dim {
  color: #6d6d78;
}

.hud .busy {
  margin-left: auto;
  color: var(--accent);
}

.splash {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 14px;
  background: rgba(14, 14, 16, 0.92);
  font-size: 12.5px;
  color: var(--muted);
}

.spinner {
  width: 26px;
  height: 26px;
  border-radius: 50%;
  border: 2px solid rgba(255, 255, 255, 0.14);
  border-top-color: var(--accent);
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.error {
  position: absolute;
  left: 50%;
  bottom: 42px;
  transform: translateX(-50%);
  max-width: 70%;
  padding: 8px 14px;
  font-size: 11.5px;
  border-radius: 8px;
  color: #ffb4b4;
  background: rgba(120, 30, 35, 0.85);
  border: 1px solid rgba(255, 120, 120, 0.28);
}
</style>
