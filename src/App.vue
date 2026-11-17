<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, reactive, ref, watch } from 'vue'
import {
  biomeAt,
  cancelOreScan,
  fetchProfiles,
  findFeatures,
  findSpawn,
  readLevelDat,
  renderTile,
  scanOres,
  type Dimension,
  type Edition,
  type MapFeature,
  type OreHit,
  type OreKind,
  type VersionProfile,
} from './lib/api'
import { DEFAULT_KINDS, FEATURE_BITS, featureMeta, LEGEND_ORDER } from './lib/features'
import { biomeName } from './lib/biomes'
import { defaultOres, oreMeta, oresForDimension } from './lib/ores'
import {
  cachedImage,
  preloadIcons,
  setIconReadyHandler,
  SPAWN_ICON,
  structureIcon,
  oreTileIcon,
} from './lib/icons'
import {
  clearHistory,
  loadHistory,
  recordHistory,
  removeHistory,
  type HistoryEntry,
} from './lib/history'
import AnalysisPanel from './components/AnalysisPanel.vue'
import BiomePanel from './components/BiomePanel.vue'
import HistoryPanel from './components/HistoryPanel.vue'
import OrePanel from './components/OrePanel.vue'
import StructurePanel from './components/StructurePanel.vue'
import UndergroundPanel from './components/UndergroundPanel.vue'

const seedInput = ref('10292992')
const seedValue = ref('10292992')
const version = ref('26.3')
const dimension = ref<Dimension>('overworld')

const profiles = ref<VersionProfile[]>([])
const features = ref<MapFeature[]>([])
const activeKinds = ref<Set<string>>(new Set(DEFAULT_KINDS))
const SCALES = [1, 4, 16, 64, 256]
const TILE = 256
/** 画布比视口大一圈，拖动时边缘不会立刻露出底色。 */
const OVERSCAN = 1.55
/** 瓦片位图缓存上限；滑动时几乎全部命中，不再反复向后端取图。 */
const TILE_CACHE_LIMIT = 900
/** 同时在算的瓦片数：太多会把内核全部占满，反而拖慢返回。 */
const TILE_CONCURRENCY = 4
const scale = ref(4)
const center = reactive({ x: 0, z: 0 })
const viewport = reactive({ w: 1280, h: 720 })
/** 实际画布尺寸（视口 × OVERSCAN）。 */
const render = reactive({ w: 1280, h: 720 })
const spawnPoint = ref<{ x: number; z: number } | null>(null)

const canvasRef = ref<HTMLCanvasElement | null>(null)
const overlayRef = ref<HTMLCanvasElement | null>(null)
const mapWrap = ref<HTMLDivElement | null>(null)

const busy = ref(false)
const hasPainted = ref(false)
const lastMs = ref(0)
const error = ref<string | null>(null)
const showLegend = ref(true)
const dragging = ref(false)
const hover = reactive({ x: 0, z: 0 })

// --- 扩展图层：群系高亮 / 矿石 / 地下剖面 / 结构详情 / 历史 ---------------

type SidePanel = 'biomes' | 'ores' | 'analysis' | 'history' | 'underground'

const sidePanel = ref<SidePanel | null>(null)
const highlightBiomes = ref<number[]>([])
const activeOres = ref<OreKind[]>([])
const oreHits = ref<OreHit[]>([])
const oreScanning = ref(false)
const oreStatus = ref('')
const selected = ref<MapFeature | null>(null)
const history = ref<HistoryEntry[]>([])
const dropActive = ref(false)
const importNote = ref<string | null>(null)

let oreSeq = 0
let oreRequestId: number | null = null

// --- mobile / touch state -------------------------------------------------

const MOBILE_QUERY = '(max-width: 860px), (pointer: coarse)'
const isMobile = ref(false)
const panel = ref<'legend' | 'settings' | null>(null)
const probe = ref<
  | { x: number; z: number; biome: string; left: number; top: number }
  | null
>(null)

/** 拖动/捏合期间只做 CSS 平移缩放，跨过预留边距时才重新取图对齐。 */
const view = reactive({ tx: 0, tz: 0, zoom: 1, ox: 0, oy: 0 })
const pinchActive = ref(false)

/** 画布比视口大出来的那一圈就是可以“免费”拖动的距离。 */
const dragLimit = computed(() => ({
  x: Math.max(48, (render.w - viewport.w) / 2 - 48),
  y: Math.max(48, (render.h - viewport.h) / 2 - 48),
}))

const stackStyle = computed(() => ({
  left: `${-(render.w - viewport.w) / 2}px`,
  top: `${-(render.h - viewport.h) / 2}px`,
  width: `${render.w}px`,
  height: `${render.h}px`,
  transform: `translate3d(${view.tx}px, ${view.tz}px, 0) scale(${view.zoom})`,
  transformOrigin: `${view.ox}px ${view.oy}px`,
}))

let media: MediaQueryList | null = null
let origin = { x: 0, z: 0, scale: 4, w: 1, h: 1 }
let observer: ResizeObserver | null = null
let featuresTimer: number | undefined
let saveTimer: number | undefined
let featureSeq = 0
let busyTimer: number | undefined
const tileCache = new Map<string, HTMLCanvasElement>()
const tileInflight = new Set<string>()
const tileQueue: TileJob[] = []
const tileQueued = new Set<string>()
let tileActive = 0
/** 上一次完整画面，缩放时先铺上去当过渡，避免整屏黑等瓦片。 */
let lastFrame: { canvas: HTMLCanvasElement; x: number; z: number; s: number; w: number; h: number } | null =
  null

interface TileJob {
  key: string
  wx: number
  wz: number
  s: number
  highlight: string
  dist: number
}

interface GestureState {
  mode: 'drag' | 'pinch'
  startX: number
  startY: number
  centerX: number
  centerZ: number
  distance: number
  scale: number
  focusX: number
  focusY: number
  focusClientX: number
  focusClientY: number
  moved: boolean
}

let gesture: GestureState | null = null
const pointers = new Map<number, { x: number; y: number }>()
let longPressTimer: number | undefined
let longPressOrigin = { x: 0, y: 0 }
let lastTapAt = 0
let lastTapPos = { x: 0, y: 0 }

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

const oresSupported = computed(
  () => currentProfile.value?.ores === true && dimension.value !== 'end',
)

const undergroundSupported = computed(() => {
  const masks = currentProfile.value?.undergroundMasks
  return ((masks?.[dimensionIndex.value] ?? 0) & 7) !== 0
})

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
    icon: structureIcon(kind),
    ...featureMeta(kind),
  }))
})

function toggleKind(kind: string) {
  const next = new Set(activeKinds.value)
  if (next.has(kind)) next.delete(kind)
  else next.add(kind)
  activeKinds.value = next
  drawMarkers()
  void loadFeatures()
}

function selectAllKinds(on: boolean) {
  activeKinds.value = on ? new Set(LEGEND_ORDER) : new Set()
  drawMarkers()
  void loadFeatures()
}

function openSide(next: SidePanel) {
  sidePanel.value = sidePanel.value === next ? null : next
  selected.value = null
  if (next === 'ores' && !activeOres.value.length) {
    activeOres.value = [...defaultOres(dimension.value)]
  }
}

function onBiomesChanged(list: number[]) {
  highlightBiomes.value = list
  void repaint('reset')
}

const viewBounds = computed(() => {
  const halfW = (viewport.w / 2) * scale.value
  const halfH = (viewport.h / 2) * scale.value
  return {
    minX: Math.round(center.x - halfW),
    minZ: Math.round(center.z - halfH),
    maxX: Math.round(center.x + halfW),
    maxZ: Math.round(center.z + halfH),
  }
})

/** 引擎单次最多收 96 个区块，所以按批扫描；12 区块半径 = 25×25 区块（400×400 格）。 */
const ORE_SCAN_RADIUS = 12
/** 画布上一次最多画这么多点，再多只会糊成一片，也拖慢重绘。 */
const ORE_DISPLAY_LIMIT = 8000
const ORE_BATCH = 96

async function scanViewportOres() {
  if (!oresSupported.value || oreScanning.value || !activeOres.value.length) return
  if (scale.value > 4) {
    oreStatus.value = '先放大到 1:4 或 1:1 再扫描，缩略图上的矿石点没有意义。'
    return
  }
  const seq = ++oreSeq
  oreScanning.value = true
  oreStatus.value = '扫描中…'
  oreHits.value = []
  drawMarkers()

  const originCx = Math.round(center.x / 16)
  const originCz = Math.round(center.z / 16)
  const radius = scale.value <= 1 ? 6 : ORE_SCAN_RADIUS
  const flat: number[] = []
  for (let dz = -radius; dz <= radius; dz += 1) {
    for (let dx = -radius; dx <= radius; dx += 1) {
      flat.push(originCx + dx, originCz + dz)
    }
  }
  const batches: number[][] = []
  for (let index = 0; index < flat.length; index += ORE_BATCH * 2) {
    batches.push(flat.slice(index, index + ORE_BATCH * 2))
  }

  const requestId = Math.floor(Math.random() * 1_000_000_000) + 1
  oreRequestId = requestId
  const hits: OreHit[] = []
  let truncated = false
  try {
    for (const [index, batch] of batches.entries()) {
      if (seq !== oreSeq) return
      const results = await scanOres(
        {
          seed: seedValue.value,
          edition: 'java',
          version: version.value,
          dimension: dimension.value,
          ores: activeOres.value,
          chunks: batch,
        },
        requestId,
      )
      if (seq !== oreSeq) return
      for (const chunk of results) {
        hits.push(...chunk.hits)
        truncated ||= chunk.truncated
      }
      oreHits.value = hits.slice()
      const percent = Math.round(((index + 1) / batches.length) * 100)
      oreStatus.value = `扫描中 ${percent}% · ${hits.length} 处`
      drawMarkers()
      if (hits.length >= ORE_DISPLAY_LIMIT) {
        truncated = true
        break
      }
    }
    oreStatus.value = `${hits.length} 处候选 · ${flat.length / 2} 个区块${
      truncated ? ' · 已达结果上限' : ''
    }`
  } catch (reason) {
    if (seq === oreSeq) oreStatus.value = String(reason)
  } finally {
    if (seq === oreSeq) {
      oreScanning.value = false
      oreRequestId = null
    }
  }
}

function setOres(value: OreKind[]) {
  activeOres.value = value
  clearOres()
}

function clearOres() {
  oreSeq += 1
  if (oreRequestId !== null) {
    void cancelOreScan(oreRequestId).catch(() => {})
    oreRequestId = null
  }
  oreScanning.value = false
  oreHits.value = []
  oreStatus.value = ''
  drawMarkers()
}

function pickFeature(clientX: number, clientY: number) {
  const canvas = canvasRef.value
  if (!canvas) return false
  const rect = canvas.getBoundingClientRect()
  const mx = clientX - rect.left
  const my = clientY - rect.top
  let best: MapFeature | null = null
  let bestDistance = 14
  for (const feature of features.value) {
    if (!activeKinds.value.has(feature.kind)) continue
    const px = (feature.x - origin.x) / origin.scale
    const pz = (feature.z - origin.z) / origin.scale
    const distance = Math.hypot(px - mx, pz - my)
    if (distance < bestDistance) {
      bestDistance = distance
      best = feature
    }
  }
  if (!best) return false
  selected.value = best
  sidePanel.value = null
  drawMarkers()
  return true
}

function closestVersion(candidate: string | null | undefined): string | null {
  if (!candidate) return null
  const list = versions.value
  if (!list.length) return null
  if (list.includes(candidate)) return candidate
  const [major, minor] = candidate.split('.').map(Number)
  const exact = list.find((item) => item.startsWith(`${major}.${minor}.`))
  if (exact) return exact
  return list.find((item) => item.startsWith(`${major}.${minor}`)) ?? null
}

async function importWorld(path: string) {
  importNote.value = '正在读取存档…'
  try {
    const info = await readLevelDat(path)
    seedInput.value = info.seed
    seedValue.value = info.seed
    const matched = closestVersion(info.version)
    if (matched) version.value = matched
    sidePanel.value = null
    selected.value = null
    clearOres()
    await jumpToSpawn()
    importNote.value = `已导入种子 ${info.seed}${
      info.version ? ` · ${info.version}${matched && matched !== info.version ? ` → ${matched}` : ''}` : ''
    }`
    setTimeout(() => {
      if (importNote.value?.startsWith('已导入')) importNote.value = null
    }, 6000)
  } catch (reason) {
    importNote.value = `导入失败：${String(reason)}`
  }
}

function pickHistory(entry: HistoryEntry) {
  seedInput.value = entry.seed
  seedValue.value = entry.seed
  version.value = entry.version
  dimension.value = entry.dimension
  sidePanel.value = null
  clearOres()
  rememberSeed()
  void jumpToSpawn()
}

function forgetHistory(entry: HistoryEntry) {
  history.value = removeHistory(entry.seed, entry.version)
}

function wipeHistory() {
  history.value = clearHistory()
}

function rememberSeed() {
  history.value = recordHistory({
    seed: seedValue.value,
    version: version.value,
    dimension: dimension.value,
    at: Date.now(),
  })
}

// --- 工作区记忆 ----------------------------------------------------------

const WORKSPACE_KEY = 'seedmap.workspace.v1'

interface Workspace {
  seed: string
  version: string
  dimension: Dimension
  scale: number
  x: number
  z: number
}

function saveWorkspace() {
  try {
    const data: Workspace = {
      seed: seedValue.value,
      version: version.value,
      dimension: dimension.value,
      scale: scale.value,
      x: center.x,
      z: center.z,
    }
    localStorage.setItem(WORKSPACE_KEY, JSON.stringify(data))
  } catch {
    // Storage may be unavailable; the map still works without persistence.
  }
}

function loadWorkspace(): Workspace | null {
  try {
    const raw = localStorage.getItem(WORKSPACE_KEY)
    if (!raw) return null
    const data = JSON.parse(raw) as Workspace
    if (typeof data?.seed !== 'string' || typeof data?.version !== 'string') return null
    return data
  } catch {
    return null
  }
}

/** 拖动时 center 每帧都在变，落盘必须防抖，否则每帧都要序列化一次 JSON。 */
function scheduleSaveWorkspace() {
  if (saveTimer !== undefined) window.clearTimeout(saveTimer)
  saveTimer = window.setTimeout(() => {
    saveTimer = undefined
    saveWorkspace()
  }, 700)
}

watch([seedValue, version, dimension, scale], scheduleSaveWorkspace)
watch(() => [center.x, center.z], scheduleSaveWorkspace)

watch(version, () => {
  if (!dimensions.value.includes(dimension.value)) {
    dimension.value = 'overworld'
  }
  void repaint('reset')
})

watch(dimension, () => {
  const allowed = new Set(oresForDimension(dimension.value).map((ore) => ore.kind))
  activeOres.value = activeOres.value.filter((kind) => allowed.has(kind))
  if (!activeOres.value.length) activeOres.value = [...defaultOres(dimension.value)]
  clearOres()
  void repaint('reset')
})

function cacheKey(x: number, z: number, s: number, highlight: string) {
  return `${seedValue.value}|${version.value}|${dimension.value}|${s}|${x}|${z}|${highlight}`
}

function tileBitmap(buffer: ArrayBuffer, tw: number, th: number, key: string) {
  const off = document.createElement('canvas')
  off.width = tw
  off.height = th
  const ctx = off.getContext('2d')
  if (ctx) {
    ctx.putImageData(new ImageData(new Uint8ClampedArray(buffer), tw, th), 0, 0)
  }
  if (tileCache.size >= TILE_CACHE_LIMIT) {
    const oldest = tileCache.keys().next().value
    if (oldest !== undefined) tileCache.delete(oldest)
  }
  tileCache.set(key, off)
  return off
}

/**
 * 重绘整块画布。
 *
 * `hard` 用于种子/版本/维度切换这类内容完全失效的场景，只在这时才清屏；
 * 平移和缩放保留上一帧像素，新瓦片逐块覆盖，避免拖动时反复闪黑。
 */
type PaintMode = 'none' | 'scale' | 'reset'

function repaint(mode: PaintMode = 'none') {
  const hard = mode !== 'none'
  const canvas = canvasRef.value
  if (!canvas) return
  const ctx = canvas.getContext('2d')
  if (!ctx) return

  const w = Math.max(240, Math.floor(viewport.w))
  const h = Math.max(200, Math.floor(viewport.h))
  const rw = Math.max(w, Math.round(w * OVERSCAN))
  const rh = Math.max(h, Math.round(h * OVERSCAN))
  render.w = rw
  render.h = rh

  const resized = canvas.width !== rw || canvas.height !== rh
  if (canvas.width !== rw) canvas.width = rw
  if (canvas.height !== rh) canvas.height = rh

  const overlay = overlayRef.value
  if (overlay && (overlay.width !== rw || overlay.height !== rh)) {
    overlay.width = rw
    overlay.height = rh
  }

  const s = scale.value
  // 先把当前画面连同它的世界坐标存下来，缩放后按比例摊开当过渡。
  let snapshot: typeof lastFrame = null
  if (mode === 'scale' && hasPainted.value && !resized) {
    const snap = document.createElement('canvas')
    snap.width = rw
    snap.height = rh
    snap.getContext('2d')?.drawImage(canvas, 0, 0)
    lastFrame = { canvas: snap, x: origin.x, z: origin.z, s: origin.scale, w: origin.w, h: origin.h }
    snapshot = lastFrame
  }
  // 左边界先对齐到 scale 的整数倍（像素不糊），瓦片再对齐到 TILE*scale，
  // 这样平移一个像素不会让所有缓存失效，只有新露出的那一列/行要重新取图。
  const left = Math.floor((center.x - (rw / 2) * s) / s) * s
  const top = Math.floor((center.z - (rh / 2) * s) / s) * s
  const step = TILE * s
  const baseX = Math.floor(left / step) * step
  const baseZ = Math.floor(top / step) * step
  const dx0 = (baseX - left) / s
  const dz0 = (baseZ - top) / s
  origin = { x: left, z: top, scale: s, w: rw, h: rh }

  const started = performance.now()
  const highlight = highlightBiomes.value.length ? highlightBiomes.value.join(',') : ''

  if (hard || resized || !hasPainted.value) {
    ctx.fillStyle = '#0e0e10'
    ctx.fillRect(0, 0, rw, rh)
    if (mode === 'reset') {
      tileCache.clear()
      lastFrame = null
    }
  }

  if (mode === 'scale' && snapshot) {
    const ratio = snapshot.s / s
    ctx.imageSmoothingEnabled = true
    ctx.drawImage(
      snapshot.canvas,
      (snapshot.x - left) / s,
      (snapshot.z - top) / s,
      snapshot.w * ratio,
      snapshot.h * ratio,
    )
    ctx.imageSmoothingEnabled = false
  }

  const cols = Math.ceil((left + rw * s - baseX) / step)
  const rows = Math.ceil((top + rh * s - baseZ) / step)
  const cx = left + (rw / 2) * s
  const cz = top + (rh / 2) * s
  const missing: TileJob[] = []

  for (let row = 0; row < rows; row += 1) {
    for (let col = 0; col < cols; col += 1) {
      const wx = baseX + col * step
      const wz = baseZ + row * step
      const key = cacheKey(wx, wz, s, highlight)
      const cached = tileCache.get(key)
      if (cached) {
        ctx.drawImage(cached, dx0 + col * TILE, dz0 + row * TILE)
        continue
      }
      missing.push({
        key,
        wx,
        wz,
        s,
        highlight,
        dist: Math.hypot(wx + step / 2 - cx, wz + step / 2 - cz),
      })
    }
  }
  const pending = missing.length
  // 只保留当前视口还需要排队的活，远离视野的旧请求直接丢掉。
  const keep = new Set(missing.map((job) => job.key))
  for (let i = tileQueue.length - 1; i >= 0; i -= 1) {
    const job = tileQueue[i]
    if (!keep.has(job.key)) {
      tileQueue.splice(i, 1)
      tileQueued.delete(job.key)
    }
  }
  // 中间的块先来，视觉上先成形。
  missing.sort((a, b) => a.dist - b.dist)
  for (const job of missing) {
    if (tileQueued.has(job.key) || tileInflight.has(job.key)) continue
    tileQueue.push(job)
    tileQueued.add(job.key)
  }
  pumpTiles()

  lastMs.value = Math.round(performance.now() - started)
  drawMarkers()
  scheduleFeatures()

  if (pending === 0) {
    hasPainted.value = true
    busy.value = false
    return
  }
  if (!hasPainted.value) {
    // 首帧仍缺块：保留启动遮罩，等第一批瓦片回来再揭开。
    busy.value = true
    return
  }
  // 有缺块时延迟一点再显示“渲染中”，避免快速平移时状态栏闪烁。
  if (busyTimer !== undefined) window.clearTimeout(busyTimer)
  busyTimer = window.setTimeout(() => {
    busyTimer = undefined
    busy.value = tileActive > 0 || tileQueue.length > 0
  }, 320)
}

function pumpTiles() {
  while (tileActive < TILE_CONCURRENCY && tileQueue.length) {
    const job = tileQueue.shift()
    if (!job) break
    tileQueued.delete(job.key)
    if (tileCache.has(job.key) || tileInflight.has(job.key)) continue
    tileActive += 1
    tileInflight.add(job.key)
    renderTile({
      seed: seedValue.value,
      edition: 'java',
      version: version.value,
      dimension: dimension.value,
      x: job.wx,
      z: job.wz,
      scale: job.s,
      width: TILE,
      height: TILE,
      elevation: 128,
      terrain: false,
      contours: false,
      highlightBiomes: job.highlight ? job.highlight.split(',').map(Number) : null,
    })
      .then((buffer) => {
        const off = tileBitmap(buffer, TILE, TILE, job.key)
        paintTile(job, off)
      })
      .catch((reason) => {
        error.value = String(reason)
      })
      .finally(() => {
        tileInflight.delete(job.key)
        tileActive -= 1
        if (tileActive === 0 && tileQueue.length === 0) {
          busy.value = false
          hasPainted.value = true
        }
        pumpTiles()
      })
  }
}

/** 瓦片到达时按当前视图落位，拖动/缩放进行中也能立刻补上。 */
function paintTile(job: TileJob, off: HTMLCanvasElement) {
  if (origin.scale !== job.s) return
  const ctx = canvasRef.value?.getContext('2d')
  if (!ctx) return
  const dx = Math.round((job.wx - origin.x) / origin.scale)
  const dy = Math.round((job.wz - origin.z) / origin.scale)
  if (dx > origin.w || dy > origin.h || dx + TILE < 0 || dy + TILE < 0) return
  ctx.drawImage(off, dx, dy)
}

/** 结构点查询比瓦片贵得多，等地图停下来再问后端。 */
function scheduleFeatures(delay = 150) {
  if (featuresTimer !== undefined) window.clearTimeout(featuresTimer)
  featuresTimer = window.setTimeout(() => {
    featuresTimer = undefined
    void loadFeatures()
  }, delay)
}

async function loadFeatures() {
  const token = ++featureSeq
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
    if (token !== featureSeq) return
    features.value = list
    drawMarkers()
  } catch {
    if (token !== featureSeq) return
    features.value = []
    drawMarkers()
  }
}

function drawMarkers() {
  const overlay = overlayRef.value
  if (!overlay) return
  const ctx = overlay.getContext('2d')
  if (!ctx) return
  ctx.clearRect(0, 0, overlay.width, overlay.height)

  // 1:64 一次全图扫描会把上千个标记挤到几像素内，所以图标随缩放收缩、
  // 过密时去掉阴影与描边，否则地图会糊成一整片彩色。
  const dense = features.value.length > 900
  const size = dense
    ? Math.max(11, Math.min(20, 48 / Math.sqrt(origin.scale)))
    : Math.max(14, Math.min(36, 72 / Math.sqrt(origin.scale)))
  const half = size / 2

  for (const feature of features.value) {
    if (!activeKinds.value.has(feature.kind)) continue
    const px = (feature.x - origin.x) / origin.scale
    const pz = (feature.z - origin.z) / origin.scale
    if (px < -size || pz < -size || px > overlay.width + size || pz > overlay.height + size) {
      continue
    }
    const meta = featureMeta(feature.kind)
    const icon = cachedImage(structureIcon(feature.kind, feature.endShip))
    const selectedNow = selected.value === feature
    if (icon) {
      if (!dense) {
        ctx.shadowColor = 'rgba(0,0,0,0.5)'
        ctx.shadowBlur = 5
        ctx.shadowOffsetY = 2
      }
      const box = selectedNow ? size + 4 : size
      ctx.drawImage(icon, px - box / 2, pz - box / 2, box, box)
      ctx.shadowColor = 'transparent'
    } else {
      ctx.beginPath()
      ctx.arc(px, pz, half, 0, Math.PI * 2)
      ctx.fillStyle = meta.color
      ctx.fill()
      if (!dense) {
        ctx.lineWidth = 2
        ctx.strokeStyle = 'rgba(12,12,14,0.85)'
        ctx.stroke()
      }
    }
  }

  if (spawnPoint.value) {
    const point = spawnPoint.value
    const px = (point.x - origin.x) / origin.scale
    const pz = (point.z - origin.z) / origin.scale
    if (px > -24 && pz > -24 && px < overlay.width + 24 && pz < overlay.height + 24) {
      const icon = cachedImage(SPAWN_ICON)
      const box = Math.max(16, Math.min(38, 84 / Math.sqrt(origin.scale)))
      if (icon) {
        ctx.shadowColor = 'rgba(0,0,0,0.5)'
        ctx.shadowBlur = 5
        ctx.shadowOffsetY = 2
        ctx.drawImage(icon, px - box / 2, pz - box / 2, box, box)
        ctx.shadowColor = 'transparent'
      } else {
        ctx.fillStyle = '#2e3138'
        ctx.strokeStyle = '#ffffff'
        ctx.lineWidth = 2
        ctx.beginPath()
        ctx.arc(px, pz, box / 2, 0, Math.PI * 2)
        ctx.fill()
        ctx.stroke()
      }
    }
  }

  // 矿石点用原版的 16×16 平铺纹理，和结构图标一眼就能区分开。
  const oreSize = Math.max(3, Math.min(9, 14 / Math.sqrt(origin.scale)))
  const deepslate = dimension.value !== 'nether'
  for (const hit of oreHits.value) {
    const px = (hit.x - origin.x) / origin.scale
    const pz = (hit.z - origin.z) / origin.scale
    if (px < -8 || pz < -8 || px > overlay.width + 8 || pz > overlay.height + 8) {
      continue
    }
    const icon = cachedImage(oreTileIcon(hit.ore, deepslate && hit.y < 0))
    if (icon) {
      ctx.imageSmoothingEnabled = false
      ctx.drawImage(icon, px - oreSize / 2, pz - oreSize / 2, oreSize, oreSize)
      ctx.imageSmoothingEnabled = true
      ctx.strokeStyle = 'rgba(12,12,14,0.7)'
      ctx.lineWidth = 1
      ctx.strokeRect(px - oreSize / 2, pz - oreSize / 2, oreSize, oreSize)
    } else {
      ctx.fillStyle = oreMeta(hit.ore).color
      ctx.fillRect(px - oreSize / 2, pz - oreSize / 2, oreSize, oreSize)
    }
  }

  if (selected.value) {
    const px = (selected.value.x - origin.x) / origin.scale
    const pz = (selected.value.z - origin.z) / origin.scale
    ctx.beginPath()
    ctx.arc(px, pz, half + 8, 0, Math.PI * 2)
    ctx.strokeStyle = '#ff77b9'
    ctx.lineWidth = 2.5
    ctx.stroke()
  }
}

function applySeed() {
  seedValue.value = seedInput.value.trim() || '0'
  panel.value = null
  selected.value = null
  clearOres()
  rememberSeed()
  void jumpToSpawn()
}

/** 出生点只是地图上的一个参考图标，失败不影响其它功能。 */
async function loadSpawnMarker() {
  try {
    const spawn = await findSpawn(seedValue.value, 'java', version.value)
    spawnPoint.value = { x: spawn.x, z: spawn.z }
    drawMarkers()
  } catch {
    spawnPoint.value = null
  }
}

async function jumpToSpawn() {
  try {
    const spawn = await findSpawn(seedValue.value, 'java', version.value)
    center.x = spawn.x
    center.z = spawn.z
    spawnPoint.value = { x: spawn.x, z: spawn.z }
    repaint('reset')
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
  void repaint('scale')
}

/** Changes the zoom level while keeping the world point under the cursor. */
function scaleAtPoint(next: number, clientX: number, clientY: number) {
  const canvas = canvasRef.value
  if (!canvas) return
  if (next === scale.value) return
  const rect = canvas.getBoundingClientRect()
  const mx = clientX - rect.left
  const my = clientY - rect.top
  const worldX = center.x + (mx - origin.w / 2) * scale.value
  const worldZ = center.z + (my - origin.h / 2) * scale.value
  scale.value = next
  center.x = worldX - (mx - origin.w / 2) * next
  center.z = worldZ - (my - origin.h / 2) * next
  void repaint('scale')
}

function nearestScale(value: number) {
  return SCALES.reduce((best, candidate) =>
    Math.abs(Math.log(candidate / value)) < Math.abs(Math.log(best / value))
      ? candidate
      : best,
  )
}

function resetView() {
  scale.value = 4
  center.x = 0
  center.z = 0
  void repaint('scale')
}

function onWheel(event: WheelEvent) {
  event.preventDefault()
  const index = SCALES.indexOf(scale.value)
  const nextIndex = Math.min(
    SCALES.length - 1,
    Math.max(0, (index < 0 ? 1 : index) + (event.deltaY < 0 ? -1 : 1)),
  )
  if (nextIndex === index) return
  scaleAtPoint(SCALES[nextIndex], event.clientX, event.clientY)
}

// --- pointer & touch handling --------------------------------------------

/**
 * 把 CSS 平移量吃进画布坐标系：位移基准前移、transform 归零、按新中心重绘。
 * 三步在同一个任务里完成，浏览器只会合成最终状态，看不到中间跳动。
 */
function realignDrag() {
  if (!gesture) return
  // 基准整体前移：指针位置和世界中心都换成“此刻”的值，避免丢掉刚才的位移。
  gesture.startX += view.tx
  gesture.startY += view.tz
  gesture.centerX = center.x
  gesture.centerZ = center.z
  view.tx = 0
  view.tz = 0
  repaint('scale')
}

function cancelLongPress() {
  if (longPressTimer !== undefined) {
    window.clearTimeout(longPressTimer)
    longPressTimer = undefined
  }
}

async function runProbe(clientX: number, clientY: number) {
  const canvas = canvasRef.value
  const wrap = mapWrap.value
  if (!canvas || !wrap) return
  const rect = canvas.getBoundingClientRect()
  const wrapRect = wrap.getBoundingClientRect()
  const wx = Math.round(center.x + (clientX - rect.left - origin.w / 2) * scale.value)
  const wz = Math.round(center.z + (clientY - rect.top - origin.h / 2) * scale.value)
  probe.value = {
    x: wx,
    z: wz,
    biome: '查询中…',
    left: clientX - wrapRect.left,
    top: clientY - wrapRect.top,
  }
  try {
    const id = await biomeAt(
      seedValue.value,
      'java',
      version.value,
      dimension.value,
      wx,
      dimension.value === 'overworld' ? 64 : 64,
      wz,
    )
    if (probe.value && probe.value.x === wx && probe.value.z === wz) {
      probe.value = { ...probe.value, biome: biomeName(id) }
    }
  } catch {
    if (probe.value) probe.value = { ...probe.value, biome: '无法读取' }
  }
  navigator.vibrate?.(12)
}

function beginLongPress(event: PointerEvent) {
  cancelLongPress()
  longPressOrigin = { x: event.clientX, y: event.clientY }
  longPressTimer = window.setTimeout(() => {
    longPressTimer = undefined
    if (gesture && gesture.moved) return
    void runProbe(longPressOrigin.x, longPressOrigin.y)
  }, 460)
}

function onPointerDown(event: PointerEvent) {
  if (event.button !== 0 && event.pointerType === 'mouse') return
  const canvas = canvasRef.value
  if (!canvas) return
  canvas.setPointerCapture(event.pointerId)
  pointers.set(event.pointerId, { x: event.clientX, y: event.clientY })
  probe.value = null

  if (pointers.size === 1) {
    gesture = {
      mode: 'drag',
      startX: event.clientX,
      startY: event.clientY,
      centerX: center.x,
      centerZ: center.z,
      distance: 0,
      scale: scale.value,
      focusX: 0,
      focusY: 0,
      focusClientX: 0,
      focusClientY: 0,
      moved: false,
    }
    dragging.value = true
    pinchActive.value = false
    view.zoom = 1
    beginLongPress(event)
  } else if (pointers.size === 2) {
    cancelLongPress()
    // 单指拖到一半再加第二指：先把平移吃进画布，捏合才从正确位置放大。
    if (view.tx !== 0 || view.tz !== 0) {
      view.tx = 0
      view.tz = 0
      repaint('scale')
    }
    const [a, b] = [...pointers.values()]
    const rect = canvas.getBoundingClientRect()
    const focusClientX = (a.x + b.x) / 2
    const focusClientY = (a.y + b.y) / 2
    gesture = {
      mode: 'pinch',
      startX: 0,
      startY: 0,
      centerX: center.x,
      centerZ: center.z,
      distance: Math.hypot(a.x - b.x, a.y - b.y) || 1,
      scale: scale.value,
      focusX: focusClientX - rect.left,
      focusY: focusClientY - rect.top,
      focusClientX,
      focusClientY,
      moved: true,
    }
    dragging.value = false
    pinchActive.value = true
    view.zoom = 1
    view.ox = gesture.focusX
    view.oy = gesture.focusY
  }
}

function onPointerMove(event: PointerEvent) {
  const canvas = canvasRef.value
  if (!canvas) return
  if (pointers.has(event.pointerId)) {
    pointers.set(event.pointerId, { x: event.clientX, y: event.clientY })
  }

  if (gesture?.mode === 'pinch' && pointers.size >= 2) {
    const [a, b] = [...pointers.values()]
    const distance = Math.hypot(a.x - b.x, a.y - b.y) || 1
    const factor = Math.min(6, Math.max(0.2, distance / gesture.distance))
    view.zoom = factor
    return
  }

  if (gesture?.mode === 'drag') {
    const dx = event.clientX - gesture.startX
    const dy = event.clientY - gesture.startY
    if (Math.hypot(dx, dy) > 6) {
      gesture.moved = true
      cancelLongPress()
    }
    center.x = gesture.centerX - dx * scale.value
    center.z = gesture.centerZ - dy * scale.value
    view.tx = dx
    view.tz = dy
    const rect = canvas.getBoundingClientRect()
    hover.x = Math.round(center.x + (event.clientX - rect.left - origin.w / 2) * scale.value)
    hover.z = Math.round(center.z + (event.clientY - rect.top - origin.h / 2) * scale.value)
    // 手指在预留边距内时只做合成层平移（不重绘）；越过边距再取一次图并归零。
    if (Math.abs(dx) > dragLimit.value.x || Math.abs(dy) > dragLimit.value.y) {
      realignDrag()
    }
  }
}

function onPointerUp(event: PointerEvent) {
  pointers.delete(event.pointerId)
  cancelLongPress()

  if (gesture?.mode === 'pinch') {
    if (pointers.size >= 1) return
    const factor = view.zoom
    const target = nearestScale(gesture.scale / factor)
    const { focusClientX, focusClientY } = gesture
    pinchActive.value = false
    view.zoom = 1
    view.ox = 0
    view.oy = 0
    gesture = null
    dragging.value = false
    if (target !== scale.value) {
      scaleAtPoint(target, focusClientX, focusClientY)
    }
    return
  }

  if (gesture?.mode === 'drag') {
    const moved = gesture.moved
    const wasMobileTouch = event.pointerType !== 'mouse'
    gesture = null
    dragging.value = false
    const shifted = view.tx !== 0 || view.tz !== 0
    view.tx = 0
    view.tz = 0

    if (!moved) {
      const now = performance.now()
      const near =
        Math.hypot(event.clientX - lastTapPos.x, event.clientY - lastTapPos.y) < 44
      if (now - lastTapAt < 320 && near) {
        const index = SCALES.indexOf(scale.value)
        const nextIndex = Math.max(0, index - 1)
        lastTapAt = 0
        if (nextIndex !== index) scaleAtPoint(SCALES[nextIndex], event.clientX, event.clientY)
      } else {
        lastTapAt = now
        lastTapPos = { x: event.clientX, y: event.clientY }
        const hit = pickFeature(event.clientX, event.clientY)
        if (!hit && wasMobileTouch && selected.value) {
          selected.value = null
          drawMarkers()
        }
      }
      if (wasMobileTouch && !moved) {
        // A plain tap dismisses the probe; long press keeps it visible.
        probe.value = null
      }
    }
    void (shifted ? repaint('scale') : repaint())
  }
}

function onPointerCancel(event: PointerEvent) {
  pointers.delete(event.pointerId)
  cancelLongPress()
  if (pointers.size === 0) {
    gesture = null
    dragging.value = false
    pinchActive.value = false
    view.zoom = 1
    void repaint()
  }
}

function openPanel(next: 'legend' | 'settings') {
  panel.value = panel.value === next ? null : next
}

function openTool(next: SidePanel) {
  panel.value = null
  selected.value = null
  sidePanel.value = next
  if (next === 'ores' && !activeOres.value.length) {
    activeOres.value = [...defaultOres(dimension.value)]
  }
}

function syncMobile() {
  const next = media?.matches ?? false
  if (next === isMobile.value) return
  isMobile.value = next
  if (!next) panel.value = null
}

onMounted(async () => {
  media = window.matchMedia(MOBILE_QUERY)
  isMobile.value = media.matches
  media.addEventListener('change', syncMobile)

  history.value = loadHistory()
  activeOres.value = [...defaultOres('overworld')]

  // 结构/矿石/群系图标来自原启动器素材，预热后 canvas 绘制是同步的。
  preloadIcons()
  setIconReadyHandler(() => drawMarkers())

  try {
    profiles.value = await fetchProfiles()
    const first = javaProfiles.value.find((profile) => profile.available)
    if (first) version.value = first.version
  } catch (reason) {
    error.value = String(reason)
  }

  const restored = loadWorkspace()
  let resumed = false
  if (restored && javaProfiles.value.some((profile) => profile.version === restored.version)) {
    seedInput.value = restored.seed
    seedValue.value = restored.seed
    version.value = restored.version
    if (restored.dimension === 'nether' || restored.dimension === 'end') {
      dimension.value = restored.dimension
    }
    if (SCALES.includes(restored.scale)) scale.value = restored.scale
    if (Number.isFinite(restored.x) && Number.isFinite(restored.z)) {
      center.x = restored.x
      center.z = restored.z
      resumed = true
    }
  }

  // Dragging a world folder or level.dat onto the window imports its seed.
  try {
    const { getCurrentWebview } = await import('@tauri-apps/api/webview')
    await getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === 'enter' || event.payload.type === 'over') {
        dropActive.value = true
      } else if (event.payload.type === 'leave') {
        dropActive.value = false
      } else if (event.payload.type === 'drop') {
        dropActive.value = false
        const path = event.payload.paths?.[0]
        if (path) void importWorld(path)
      }
    })
  } catch {
    // Running in a plain browser (vite dev without the shell): no drag-drop API.
  }

  if (isMobile.value) {
    scale.value = 16
  }

  if (mapWrap.value) {
    observer = new ResizeObserver((entries) => {
      const box = entries[0]?.contentRect
      if (!box) return
      viewport.w = box.width
      viewport.h = box.height
      if (hasPainted.value && !dragging.value) void repaint('scale')
    })
    observer.observe(mapWrap.value)
    viewport.w = mapWrap.value.clientWidth
    viewport.h = mapWrap.value.clientHeight
  }

  rememberSeed()

  if (resumed) {
    repaint()
    void loadSpawnMarker()
  } else {
    await jumpToSpawn()
  }
})

onBeforeUnmount(() => {
  observer?.disconnect()
  media?.removeEventListener('change', syncMobile)
  cancelLongPress()
  setIconReadyHandler(null)
  if (busyTimer !== undefined) window.clearTimeout(busyTimer)
  if (featuresTimer !== undefined) window.clearTimeout(featuresTimer)
  if (saveTimer !== undefined) {
    window.clearTimeout(saveTimer)
    saveWorkspace()
  }
})
</script>

<template>
  <div class="app" :class="{ mobile: isMobile }">
    <header class="topbar">
      <div class="brand">
        <div class="logo" />
        <div class="brand-text">
          <div class="name">Seedmap Studio</div>
          <div v-if="!isMobile" class="sub">Minecraft 种子地图</div>
        </div>
      </div>

      <div v-if="!isMobile" class="controls">
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

      <div v-if="!isMobile" class="engine" :class="{ warn: !engineReady }">
        {{ engineReady ? 'cubiomes 引擎就绪' : '引擎不可用' }}
      </div>

      <button
        v-else
        class="mobile-more"
        :class="{ open: panel === 'settings' }"
        aria-label="设置"
        @click="openPanel('settings')"
      >
        <i /><i /><i />
      </button>
    </header>

    <div v-if="isMobile" class="mobile-search">
      <label class="seed-box">
        <span>种子</span>
        <input
          v-model="seedInput"
          inputmode="text"
          spellcheck="false"
          enterkeyhint="go"
          @keydown.enter="applySeed"
        />
      </label>
      <button class="btn primary" @click="applySeed">生成</button>
    </div>

    <main ref="mapWrap" class="map-area">
      <div class="map-stack" :style="stackStyle">
        <canvas
          ref="canvasRef"
          class="layer"
          @wheel.prevent="onWheel"
          @pointerdown="onPointerDown"
          @pointermove="onPointerMove"
          @pointerup="onPointerUp"
          @pointercancel="onPointerCancel"
        />
        <canvas ref="overlayRef" class="layer overlay" />
      </div>

      <div v-if="isMobile" class="zoom-rail">
        <button aria-label="放大" @click="stepScale(-1)">＋</button>
        <span>{{ scaleLabel }}</span>
        <button aria-label="缩小" @click="stepScale(1)">－</button>
      </div>

      <div v-if="!isMobile" class="zoom">
        <button title="放大" @click="stepScale(-1)">＋</button>
        <div class="zoom-label">{{ scaleLabel }}</div>
        <button title="缩小" @click="stepScale(1)">－</button>
        <button class="home" title="回到出生点" @click="jumpToSpawn">⌂</button>
      </div>

      <nav v-if="!isMobile" class="side-dock">
        <button
          :class="{ active: sidePanel === 'biomes' }"
          title="生物群系高亮"
          @click="openSide('biomes')"
        >
          群系
        </button>
        <button
          :class="{ active: sidePanel === 'ores' }"
          :disabled="!oresSupported"
          title="矿石分布"
          @click="openSide('ores')"
        >
          矿石
        </button>
        <button
          :class="{ active: sidePanel === 'underground' }"
          :disabled="!undergroundSupported"
          title="地下剖面"
          @click="openSide('underground')"
        >
          剖面
        </button>
        <button
          :class="{ active: sidePanel === 'analysis' }"
          title="区域分析"
          @click="openSide('analysis')"
        >
          分析
        </button>
        <button
          :class="{ active: sidePanel === 'history' }"
          title="最近种子"
          @click="openSide('history')"
        >
          历史
        </button>
        <button :class="{ active: showLegend }" title="结构图例" @click="showLegend = !showLegend">
          图例
        </button>
      </nav>

      <div v-if="sidePanel || selected" class="side-panel">
        <BiomePanel
          v-if="sidePanel === 'biomes'"
          :biomes="currentProfile?.biomes ?? []"
          :selected="highlightBiomes"
          @update:selected="onBiomesChanged"
          @close="sidePanel = null"
        />
        <OrePanel
          v-else-if="sidePanel === 'ores'"
          :dimension="dimension"
          :supported="oresSupported"
          :active="activeOres"
          :scanning="oreScanning"
          :status="oreStatus"
          :hits="oreHits.length"
          @update:active="setOres"
          @scan="scanViewportOres"
          @clear="clearOres"
          @close="sidePanel = null"
        />
        <UndergroundPanel
          v-else-if="sidePanel === 'underground'"
          :seed="seedValue"
          edition="java"
          :version="version"
          :dimension="dimension"
          :center="center"
          :profile="currentProfile"
          @close="sidePanel = null"
        />
        <AnalysisPanel
          v-else-if="sidePanel === 'analysis'"
          :features="features"
          :seed="seedValue"
          edition="java"
          :version="version"
          :dimension="dimension"
          :bounds="viewBounds"
          @close="sidePanel = null"
        />
        <HistoryPanel
          v-else-if="sidePanel === 'history'"
          :entries="history"
          :current="seedValue"
          @pick="pickHistory"
          @remove="forgetHistory"
          @clear="wipeHistory"
          @close="sidePanel = null"
        />
        <StructurePanel
          v-else-if="selected"
          :feature="selected"
          :seed="seedValue"
          edition="java"
          :version="version"
          :dimension="dimension"
          @close="selected = null"
        />
      </div>

      <section v-if="!isMobile && showLegend" class="legend">
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
            <i class="legend-icon" :class="{ dim: !item.active }">
              <img v-if="item.icon" :src="item.icon" :alt="item.label" />
              <b v-else :style="{ background: item.active ? item.color : '#4a4a52' }" />
            </i>
            <span>{{ item.label }}</span>
            <b v-if="item.active">{{ item.count }}</b>
          </div>
        </div>
      </section>
      <footer class="hud">
        <span>X {{ Math.round(center.x) }} · Z {{ Math.round(center.z) }}</span>
        <span v-if="!isMobile" class="dim">光标 {{ hover.x }} , {{ hover.z }}</span>
        <span>1 像素 = {{ scale }} 格</span>
        <span v-if="!isMobile">{{ lastMs }} ms</span>
        <span v-if="busy" class="busy">渲染中…</span>
      </footer>

      <div
        v-if="probe"
        class="probe"
        :style="{ left: `${probe.left}px`, top: `${probe.top}px` }"
      >
        <b>{{ probe.biome }}</b>
        <span>X {{ probe.x }} · Y 64 · Z {{ probe.z }}</span>
      </div>

      <div v-if="busy && !hasPainted" class="splash">
        <div class="spinner" />
        <p>正在生成地形…</p>
      </div>

      <div v-if="error" class="error">{{ error }}</div>

      <div v-if="dropActive" class="drop-hint">
        <span>松开以导入存档</span>
        <em>level.dat 或解压后的世界文件夹</em>
      </div>
      <div v-if="importNote" class="toast">{{ importNote }}</div>

      <!-- mobile floating dock -->
      <nav v-if="isMobile" class="dock">
        <div class="dock-dims">
          <button
            v-for="item in dimensions"
            :key="item"
            :class="{ active: dimension === item }"
            @click="dimension = item"
          >
            {{ DIM_LABEL[item] }}
          </button>
        </div>
        <button class="dock-btn" @click="openPanel('legend')">
          图例<em v-if="features.length">{{ features.length }}</em>
        </button>
        <button class="dock-btn" @click="openPanel('settings')">工具</button>
        <button class="dock-btn" aria-label="回到出生点" @click="jumpToSpawn">⌂</button>
      </nav>

      <div v-if="isMobile && panel" class="scrim" @click="panel = null" />

      <transition name="sheet">
        <section v-if="isMobile && panel" class="sheet">
          <header>
            <span>{{ panel === 'legend' ? '结构图例' : '设置' }}</span>
            <button aria-label="关闭" @click="panel = null">×</button>
          </header>

          <div v-if="panel === 'legend'" class="sheet-body">
            <div class="legend-actions">
              <button @click="selectAllKinds(true)">全部显示</button>
              <button @click="selectAllKinds(false)">全部隐藏</button>
            </div>
            <div class="legend-list">
              <div
                v-for="item in legendItems"
                :key="item.kind"
                :class="{ off: !item.active }"
                @click="toggleKind(item.kind)"
              >
                <i class="legend-icon" :class="{ dim: !item.active }">
                  <img v-if="item.icon" :src="item.icon" :alt="item.label" />
                  <b v-else :style="{ background: item.active ? item.color : '#4a4a52' }" />
                </i>
                <span>{{ item.label }}</span>
                <b v-if="item.active">{{ item.count }}</b>
              </div>
            </div>
          </div>

          <div v-else class="sheet-body settings">
            <label class="row">
              <span>种子</span>
              <input v-model="seedInput" spellcheck="false" @keydown.enter="applySeed" />
            </label>
            <label class="row">
              <span>版本</span>
              <select v-model="version">
                <option v-for="item in versions" :key="item" :value="item">
                  {{ item }}
                </option>
              </select>
            </label>
            <div class="row">
              <span>维度</span>
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
            <div class="row static">
              <span>引擎</span>
              <b :class="{ warn: !engineReady }">
                {{ engineReady ? 'cubiomes 就绪' : '不可用' }}
              </b>
            </div>
            <div class="row static">
              <span>渲染</span>
              <b>{{ lastMs }} ms · 1 px = {{ scale }} 格</b>
            </div>
            <button class="btn ghost" @click="resetView">重置视图</button>
            <div class="tool-grid">
              <button :disabled="!oresSupported" @click="openTool('ores')">矿石分布</button>
              <button :disabled="!undergroundSupported" @click="openTool('underground')">
                地下剖面
              </button>
              <button @click="openTool('biomes')">群系高亮</button>
              <button @click="openTool('analysis')">区域分析</button>
              <button @click="openTool('history')">最近种子</button>
            </div>
            <p class="hint">单指拖动平移 · 双指捏合缩放 · 双击放大 · 长按查群系</p>
          </div>
        </section>
      </transition>
    </main>
  </div>
</template>

<style scoped>
.app {
  display: flex;
  flex-direction: column;
  height: 100vh;
  height: 100dvh;
  background: var(--bg);
}

/* WKWebView reports real safe-area insets; Android is inset natively in
   MainActivity, so only iOS needs the CSS padding. */
@supports (-webkit-touch-callout: none) {
  .app {
    padding-top: env(safe-area-inset-top);
  }

  .dock {
    bottom: calc(10px + env(safe-area-inset-bottom));
  }

  .sheet {
    padding-bottom: env(safe-area-inset-bottom);
  }
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

.btn.ghost {
  margin-top: 6px;
  background: var(--panel-2);
  border: 1px solid var(--border);
  color: var(--text);
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

/* ---------------- mobile ---------------- */

.app.mobile {
  overscroll-behavior: none;
}

.app.mobile .topbar {
  height: 50px;
  padding: 0 10px 0 12px;
  gap: 10px;
}

.app.mobile .logo {
  width: 26px;
  height: 26px;
  border-radius: 7px;
}

.app.mobile .brand-text .name {
  font-size: 13px;
}

.mobile-more {
  margin-left: auto;
  width: 36px;
  height: 36px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 4px;
  border-radius: 9px;
  background: var(--panel-2);
  border: 1px solid var(--border);
}

.mobile-more i {
  width: 3.5px;
  height: 3.5px;
  border-radius: 50%;
  background: var(--muted);
}

.mobile-more.open {
  border-color: rgba(255, 119, 185, 0.4);
}

.mobile-more.open i {
  background: var(--accent);
}

.mobile-search {
  display: flex;
  gap: 8px;
  padding: 8px 10px;
  background: var(--panel);
  border-bottom: 1px solid var(--border);
  flex: 0 0 auto;
}

.seed-box {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  height: 40px;
  padding: 0 12px;
  background: var(--panel-2);
  border: 1px solid var(--border);
  border-radius: 10px;
  font-size: 11.5px;
  color: var(--muted);
}

.seed-box input {
  flex: 1;
  min-width: 0;
  border: none;
  outline: none;
  background: transparent;
  font-size: 15px;
  color: var(--text);
  font-variant-numeric: tabular-nums;
}

.app.mobile .btn {
  height: 40px;
  padding: 0 16px;
  font-size: 14px;
  border-radius: 10px;
}

.map-area {
  position: relative;
  flex: 1;
  min-height: 0;
  overflow: hidden;
  background: #0e0e10;
}

.map-stack {
  position: absolute;
  left: 0;
  top: 0;
  will-change: transform;
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

.zoom-rail {
  position: absolute;
  right: calc(10px + env(safe-area-inset-right));
  top: 50%;
  transform: translateY(-50%);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 5px;
  background: rgba(24, 24, 28, 0.82);
  border: 1px solid var(--border);
  border-radius: 12px;
  backdrop-filter: blur(14px);
}

.zoom-rail button {
  width: 38px;
  height: 38px;
  border-radius: 9px;
  background: var(--panel-2);
  font-size: 17px;
  line-height: 1;
}

.zoom-rail button:active {
  background: #34343d;
}

.zoom-rail span {
  font-size: 9.5px;
  color: var(--muted);
  font-variant-numeric: tabular-nums;
  padding: 1px 0;
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

.legend-list i.legend-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  flex: 0 0 auto;
  border-radius: 5px;
  background: rgba(255, 255, 255, 0.05);
  box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.5);
}

.legend-icon img {
  width: 18px;
  height: 18px;
  object-fit: contain;
  image-rendering: -webkit-optimize-contrast;
}

.legend-icon b {
  width: 9px;
  height: 9px;
  border-radius: 50%;
}

.legend-icon.dim {
  filter: grayscale(1);
  opacity: 0.38;
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

.app.mobile .hud {
  display: none;
}

.probe {
  position: absolute;
  z-index: 5;
  transform: translate(-50%, calc(-100% - 14px));
  min-width: 130px;
  padding: 8px 11px;
  border-radius: 10px;
  background: rgba(18, 18, 21, 0.94);
  border: 1px solid rgba(255, 119, 185, 0.35);
  box-shadow: 0 10px 26px rgba(0, 0, 0, 0.55);
  pointer-events: none;
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.probe b {
  font-size: 12.5px;
  color: #ffd0e7;
  font-weight: 600;
}

.probe span {
  font-size: 10.5px;
  color: var(--muted);
  font-variant-numeric: tabular-nums;
}

.probe::after {
  content: '';
  position: absolute;
  left: 50%;
  bottom: -5px;
  width: 9px;
  height: 9px;
  transform: translateX(-50%) rotate(45deg);
  background: rgba(18, 18, 21, 0.94);
  border-right: 1px solid rgba(255, 119, 185, 0.35);
  border-bottom: 1px solid rgba(255, 119, 185, 0.35);
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

/* ---------------- mobile dock & sheets ---------------- */

.dock {
  position: absolute;
  left: 10px;
  right: 10px;
  bottom: calc(10px + env(safe-area-inset-bottom));
  z-index: 6;
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px;
  background: rgba(24, 24, 28, 0.9);
  border: 1px solid var(--border);
  border-radius: 16px;
  backdrop-filter: blur(18px);
  box-shadow: 0 12px 30px rgba(0, 0, 0, 0.5);
}

.dock-dims {
  display: flex;
  flex: 1;
  gap: 3px;
  padding: 3px;
  background: var(--panel-2);
  border-radius: 11px;
}

.dock-dims button {
  flex: 1;
  height: 38px;
  border-radius: 9px;
  font-size: 13px;
  color: var(--muted);
}

.dock-dims button.active {
  background: var(--accent-soft);
  color: #ff9ecb;
}

.dock-btn {
  position: relative;
  min-width: 48px;
  height: 44px;
  padding: 0 10px;
  border-radius: 11px;
  background: var(--panel-2);
  font-size: 13px;
  color: #d6d6de;
}

.dock-btn:active {
  background: #34343d;
}

.dock-btn em {
  position: absolute;
  top: 3px;
  right: 4px;
  font-size: 9px;
  font-style: normal;
  color: var(--accent);
}

.scrim {
  position: absolute;
  inset: 0;
  z-index: 7;
  background: rgba(0, 0, 0, 0.45);
}

.sheet {
  position: absolute;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 8;
  max-height: 68%;
  display: flex;
  flex-direction: column;
  background: #1b1b20;
  border-top: 1px solid var(--border);
  border-radius: 18px 18px 0 0;
  box-shadow: 0 -18px 40px rgba(0, 0, 0, 0.55);
}

.sheet header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 16px 10px;
  font-size: 13.5px;
  font-weight: 600;
}

.sheet header button {
  width: 32px;
  height: 32px;
  font-size: 18px;
  color: var(--muted);
  border-radius: 8px;
  background: var(--panel-2);
}

.sheet-body {
  overflow-y: auto;
  padding: 0 12px 14px;
  -webkit-overflow-scrolling: touch;
}

.sheet .legend-list {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 2px;
  padding: 4px 0 0;
}

.sheet .legend-list div {
  padding: 9px 8px;
  font-size: 12.5px;
}

.sheet .legend-actions {
  padding: 0 0 6px;
}

.sheet .legend-actions button {
  height: 34px;
  font-size: 12.5px;
  border-radius: 9px;
}

.sheet-body.settings {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 0 14px 18px;
}

.row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  min-height: 46px;
  padding: 0 12px;
  background: var(--panel-2);
  border: 1px solid var(--border);
  border-radius: 11px;
  font-size: 12.5px;
  color: var(--muted);
}

.row.static b {
  color: #c9c9d2;
  font-weight: 500;
}

.row.static b.warn {
  color: #e3a06b;
}

.row input {
  flex: 1;
  min-width: 0;
  border: none;
  outline: none;
  background: transparent;
  text-align: right;
  font-size: 14px;
  color: var(--text);
  font-variant-numeric: tabular-nums;
}

.row select {
  border: none;
  outline: none;
  background: transparent;
  font-size: 14px;
  color: var(--text);
}

.row .segmented {
  padding: 2px;
}

.row .segmented button {
  height: 30px;
  padding: 0 10px;
  font-size: 12px;
}

.hint {
  margin: 4px 2px 0;
  font-size: 11px;
  line-height: 1.6;
  color: #6f6f7a;
}

.sheet-enter-active,
.sheet-leave-active {
  transition: transform 0.22s ease;
}

.sheet-enter-from,
.sheet-leave-to {
  transform: translateY(100%);
}

/* --- 右侧工具列与浮层 --------------------------------------------------- */

.side-dock {
  position: absolute;
  right: 14px;
  top: 14px;
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 5px;
  background: rgba(24, 24, 28, 0.88);
  border: 1px solid var(--border);
  border-radius: 10px;
  backdrop-filter: blur(14px);
}

.side-dock button {
  width: 42px;
  height: 28px;
  border-radius: 7px;
  font-size: 11px;
  color: var(--muted);
}

.side-dock button:hover {
  background: var(--panel-2);
  color: var(--text);
}

.side-dock button.active {
  background: var(--accent-soft);
  color: var(--text);
}

.side-dock button:disabled {
  opacity: 0.35;
  cursor: default;
}

.side-panel {
  position: absolute;
  right: 68px;
  top: 14px;
  z-index: 6;
  max-height: calc(100% - 64px);
  display: flex;
}

.drop-hint {
  position: absolute;
  inset: 12px;
  z-index: 8;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 6px;
  border: 2px dashed rgba(255, 119, 185, 0.6);
  border-radius: 14px;
  background: rgba(20, 12, 16, 0.72);
  backdrop-filter: blur(4px);
  pointer-events: none;
}

.drop-hint span {
  font-size: 14px;
  font-weight: 600;
}

.drop-hint em {
  font-style: normal;
  font-size: 11.5px;
  color: var(--muted);
}

.toast {
  position: absolute;
  left: 50%;
  bottom: 44px;
  transform: translateX(-50%);
  z-index: 7;
  max-width: 80%;
  padding: 8px 14px;
  border-radius: 9px;
  background: rgba(28, 28, 34, 0.94);
  border: 1px solid var(--border);
  font-size: 11.5px;
  line-height: 1.5;
  color: var(--text);
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.45);
}

.tool-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 6px;
  margin-top: 8px;
}

.tool-grid button {
  height: 34px;
  border-radius: 8px;
  background: var(--panel-2);
  border: 1px solid var(--border);
  font-size: 12px;
  color: var(--text);
}

.tool-grid button:disabled {
  opacity: 0.4;
}

@media (max-width: 860px) {
  .map-area {
    touch-action: none;
  }

  .side-panel {
    position: fixed;
    right: 0;
    left: 0;
    bottom: 0;
    top: auto;
    z-index: 30;
    max-height: 78dvh;
    justify-content: center;
    padding: 0 8px;
  }

  .side-panel > * {
    width: 100%;
    max-width: none;
    max-height: 78dvh;
    border-radius: 16px 16px 0 0;
  }

  .toast {
    bottom: 92px;
  }
}
</style>
