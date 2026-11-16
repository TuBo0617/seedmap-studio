<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import {
  undergroundTile,
  type Dimension,
  type Edition,
  type VersionProfile,
} from '../lib/api'

const props = defineProps<{
  seed: string
  edition: Edition
  version: string
  dimension: Dimension
  center: { x: number; z: number }
  profile: VersionProfile | null
}>()

const emit = defineEmits<{ (event: 'close'): void }>()

const SIZE = 256
const TILE = 64
const canvas = ref<HTMLCanvasElement | null>(null)
const layers = ref(7)
const loading = ref(false)
const error = ref<string | null>(null)
const lastMs = ref(0)

const dimensionIndex = computed(() =>
  props.dimension === 'overworld' ? 0 : props.dimension === 'nether' ? 1 : 2,
)

const allowedLayers = computed(() => {
  const masks = props.profile?.undergroundMasks
  return masks?.[dimensionIndex.value] ?? 0
})

const yRange = computed<[number, number]>(() => {
  const ranges = props.profile?.undergroundYRanges
  const range = ranges?.[dimensionIndex.value]
  if (Array.isArray(range) && range.length === 2) {
    return [range[0], range[1]]
  }
  return props.dimension === 'overworld' ? [-64, 319] : [0, 127]
})

const y = ref(Math.round((yRange.value[0] + yRange.value[1]) / 2))

const supported = computed(() => (allowedLayers.value & 7) !== 0)

const layerToggles = computed(() => [
  { bit: 1, label: '岩石', color: '#686b74', on: (layers.value & 1) !== 0, allowed: (allowedLayers.value & 1) !== 0 },
  { bit: 2, label: '矿洞', color: '#44aad6', on: (layers.value & 2) !== 0, allowed: (allowedLayers.value & 2) !== 0 },
  { bit: 4, label: '峡谷', color: '#e89742', on: (layers.value & 4) !== 0, allowed: (allowedLayers.value & 4) !== 0 },
])

let seq = 0

function toggleLayer(bit: number) {
  layers.value ^= bit
  void render()
}

async function render() {
  const el = canvas.value
  if (!el || !supported.value) return
  if (layers.value === 0) {
    el.getContext('2d')?.clearRect(0, 0, SIZE, SIZE)
    return
  }
  const current = ++seq
  loading.value = true
  error.value = null
  const started = performance.now()
  const ctx = el.getContext('2d')
  if (!ctx) return
  ctx.fillStyle = '#101014'
  ctx.fillRect(0, 0, SIZE, SIZE)

  const left = Math.floor((props.center.x - SIZE / 2) / 16) * 16
  const top = Math.floor((props.center.z - SIZE / 2) / 16) * 16
  const cols = SIZE / TILE
  const jobs: Promise<void>[] = []
  for (let row = 0; row < cols; row += 1) {
    for (let col = 0; col < cols; col += 1) {
      jobs.push(
        undergroundTile({
          tile: {
            seed: props.seed,
            edition: props.edition,
            version: props.version,
            dimension: props.dimension,
            x: left + col * TILE,
            z: top + row * TILE,
            scale: 1,
            width: TILE,
            height: TILE,
            elevation: y.value,
            terrain: false,
            contours: false,
          },
          layers: layers.value,
        })
          .then((buffer) => {
            if (current !== seq) return
            const image = new ImageData(new Uint8ClampedArray(buffer), TILE, TILE)
            const off = document.createElement('canvas')
            off.width = TILE
            off.height = TILE
            off.getContext('2d')?.putImageData(image, 0, 0)
            ctx.drawImage(off, col * TILE, row * TILE)
          })
          .catch((reason) => {
            if (current === seq) error.value = String(reason)
          }),
      )
    }
  }
  await Promise.all(jobs)
  if (current !== seq) return
  loading.value = false
  lastMs.value = Math.round(performance.now() - started)
}

watch(
  () => [props.seed, props.version, props.dimension],
  () => {
    y.value = Math.round((yRange.value[0] + yRange.value[1]) / 2)
    void render()
  },
)

// The version profile arrives after the first paint; retry once it does.
watch(
  () => props.profile,
  () => {
    y.value = Math.round((yRange.value[0] + yRange.value[1]) / 2)
    if (supported.value) void render()
  },
)

onMounted(() => void render())
</script>

<template>
  <section class="panel">
    <header>
      <div>
        <b>地下剖面</b>
        <span class="sub">Y = {{ y }} · 256×256 格</span>
      </div>
      <button class="close" aria-label="关闭" @click="emit('close')">×</button>
    </header>

    <div class="body">
      <p v-if="!supported" class="hint">
        这个版本或维度没有可用的地下切片（需要 Java 1.13+）。
      </p>
      <template v-else>
        <div class="canvas-wrap">
          <canvas ref="canvas" :width="SIZE" :height="SIZE" />
          <div v-if="loading" class="loading">生成中…</div>
        </div>

        <label class="slider">
          <span>Y {{ y }}</span>
          <input v-model.number="y" type="range" :min="yRange[0]" :max="yRange[1]" @change="render" />
        </label>

        <div class="layers">
          <button
            v-for="layer in layerToggles"
            :key="layer.bit"
            class="layer"
            :class="{ on: layer.on }"
            :disabled="!layer.allowed"
            @click="toggleLayer(layer.bit)"
          >
            <i :style="{ background: layer.on ? layer.color : '#3a3a43' }" />
            {{ layer.label }}
          </button>
        </div>

        <p class="hint small">
          颜色：岩石为灰、洞穴为蓝、峡谷为橙。拖动 Y 查看不同高度，值越大越靠近地表。
          <span v-if="lastMs"> · {{ lastMs }} ms</span>
        </p>
        <p v-if="error" class="hint error">{{ error }}</p>
      </template>
    </div>
  </section>
</template>

<style scoped>
.panel {
  display: flex;
  flex-direction: column;
  width: 288px;
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
  padding: 10px 12px 12px;
}

.canvas-wrap {
  position: relative;
  width: 100%;
  aspect-ratio: 1;
  border-radius: 8px;
  overflow: hidden;
  border: 1px solid var(--border);
  background: #101014;
}

canvas {
  display: block;
  width: 100%;
  height: 100%;
  image-rendering: pixelated;
}

.loading {
  position: absolute;
  inset: 0;
  display: grid;
  place-items: center;
  background: rgba(12, 12, 16, 0.55);
  font-size: 11px;
  color: var(--muted);
}

.slider {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 10px;
  font-size: 11px;
  color: var(--muted);
}

.slider span {
  width: 52px;
  font-variant-numeric: tabular-nums;
}

.slider input {
  flex: 1;
  accent-color: var(--accent);
}

.layers {
  display: flex;
  gap: 6px;
  margin-top: 8px;
}

.layer {
  display: flex;
  align-items: center;
  gap: 5px;
  height: 28px;
  padding: 0 9px;
  border-radius: 7px;
  background: var(--panel-2);
  border: 1px solid var(--border);
  font-size: 11px;
  color: var(--muted);
}

.layer.on {
  color: var(--text);
  border-color: rgba(255, 119, 185, 0.45);
}

.layer:disabled {
  opacity: 0.4;
}

.layer i {
  width: 8px;
  height: 8px;
  border-radius: 3px;
}

.hint {
  margin: 8px 0 0;
  font-size: 11px;
  line-height: 1.6;
  color: var(--muted);
}

.hint.small {
  font-size: 10.5px;
}

.hint.error {
  color: #ff8a80;
}
</style>
