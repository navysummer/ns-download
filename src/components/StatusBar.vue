<script setup lang="ts">
import { ref, computed } from "vue";
import { useDownloadStore } from "../lib/store";
import { ArrowDown, Circle, Gauge, Power, MessageSquarePlus } from "lucide-vue-next";

const store = useDownloadStore();
const showSpeedPopover = ref(false);
const showShutdownPopover = ref(false);
const customSpeed = ref("");
const shutdownMinutes = ref("5");

const activeCount = computed(() => store.tasks.filter(t => t.status === 1).length);
const totalCount = computed(() => store.tasks.length);
const pausedCount = computed(() => store.tasks.filter(t => t.status === 2).length);

const localLimited = ref(store.settings.speedLimit > 0);
const localLimitKbs = ref(store.settings.speedLimit > 0 ? store.settings.speedLimit / 1024 : 512);

function applySpeedLimit() {
  store.settings.speedLimit = localLimited.value ? localLimitKbs.value * 1024 : 0;
  store.saveSettingsDebounced();
}

function formatSpeed(bytes: number): string {
  if (bytes <= 0) return "0 B/s";
  if (bytes >= 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB/s`;
  return `${Math.round(bytes / 1024)} KB/s`;
}

const dlSpeed = computed(() => store.totalDownloadSpeed);
const ulSpeed = computed(() => store.totalUploadSpeed);

let shutdownTimer: ReturnType<typeof setTimeout> | null = null;
const shutdownScheduled = ref(false);

function scheduleShutdown(minutes: number) {
  if (shutdownTimer) clearTimeout(shutdownTimer);
  if (minutes <= 0) { store.shutdownSystem('shutdown'); return; }
  shutdownScheduled.value = true;
  shutdownTimer = setTimeout(() => {
    const activeTasks = store.tasks.filter(t => t.status === 1);
    if (activeTasks.length === 0) {
      store.shutdownSystem('shutdown');
    } else {
      // Wait until all tasks complete
      const check = setInterval(() => {
        if (store.tasks.filter(t => t.status === 1).length === 0) {
          clearInterval(check);
          store.shutdownSystem('shutdown');
        }
      }, 10000);
    }
    shutdownScheduled.value = false;
  }, minutes * 60 * 1000);
}

const speedPresets = [
  { label: '128 KB/s', kbs: 128 },
  { label: '512 KB/s', kbs: 512 },
  { label: '1 MB/s', kbs: 1024 },
  { label: '2 MB/s', kbs: 2048 },
  { label: '5 MB/s', kbs: 5120 },
];

const shutdownPresets = [
  { label: '立即', minutes: 0 },
  { label: '1 分钟', minutes: 1 },
  { label: '5 分钟', minutes: 5 },
  { label: '10 分钟', minutes: 10 },
  { label: '30 分钟', minutes: 30 },
];
</script>

<template>
  <footer :style="{ backgroundColor: '#2C2C2E', borderTop: '1px solid #48484A', color: '#8E8E93', height: '28px' }"
    class="flex items-center justify-between px-4 py-1 text-2xs select-none">
    <!-- Left: status + speed -->
    <div class="flex items-center gap-4">
      <div class="flex items-center gap-1.5">
        <Circle :style="{ color: activeCount > 0 ? '#22C55E' : '#8E8E93' }" class="h-2 w-2" />
        <span>{{ activeCount > 0 ? '下载中' : '空闲' }}</span>
      </div>
      <div class="flex items-center gap-1">
        <ArrowDown :style="{ color: activeCount > 0 ? '#22C55E' : '#8E8E93' }" class="h-2.5 w-2.5" />
        <span class="tabular-nums">{{ formatSpeed(dlSpeed) }}/s</span>
      </div>
      <!-- Speed sparkline -->
      <svg v-if="store.speedHistory.length > 1" width="40" height="16" class="shrink-0">
        <polyline
          :points="store.speedHistory.map((v, i) => {
            const x = (i / Math.max(store.speedHistory.length - 1, 1)) * 38 + 1;
            const max = Math.max(...store.speedHistory, 1);
            const y = 14 - (v / max) * 12;
            return `${x},${y}`;
          }).join(' ')"
          fill="none" stroke="#22C55E" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"
        />
      </svg>
      <span v-if="ulSpeed > 0" class="tabular-nums">↑ {{ formatSpeed(ulSpeed) }}</span>
      <span class="tabular-nums">{{ activeCount }} 活跃 · {{ pausedCount }} 暂停 · {{ totalCount }} 任务</span>
    </div>

    <!-- Right: speed limit + shutdown + feedback -->
    <div class="flex items-center gap-3">
      <!-- Speed limit -->
      <div class="relative">
          <button @click="showSpeedPopover = !showSpeedPopover"
            class="hover-bg-surface2 flex items-center gap-1 rounded px-1 py-0.5 transition-colors"
            :style="{ color: localLimited ? '#3B82F6' : '#8E8E93' }"
          >
            <Gauge class="h-2.5 w-2.5" />
            <span class="tabular-nums">{{ localLimited ? `${localLimitKbs} KB/s` : '无限' }}</span>
          </button>
        <div v-if="showSpeedPopover"
          class="fixed z-50 rounded-lg shadow-lg"
          :style="{
            backgroundColor: '#2C2C2E', border: '1px solid #48484A',
            bottom: '32px', right: '120px', width: '220px',
          }"
        >
          <div class="p-3">
            <div class="mb-2 flex items-center justify-between">
              <span :style="{ color: '#F5F5F7' }" class="text-xs font-semibold">速度限制</span>
              <button @click="localLimited = !localLimited; applySpeedLimit()"
                class="rounded-sm px-2 py-0.5 text-2xs font-medium transition-colors"
                :style="{
                  backgroundColor: localLimited ? '#3B82F6' : '#3A3A3C',
                  color: localLimited ? '#fff' : '#A1A1A6',
                }"
              >{{ localLimited ? '开启' : '关闭' }}</button>
            </div>
            <div class="flex flex-wrap gap-1.5">
              <button v-for="p in speedPresets" :key="p.kbs"
                @click="localLimited = true; localLimitKbs = p.kbs; applySpeedLimit()"
                class="rounded px-2 py-1 text-2xs transition-colors"
                :style="{
                  backgroundColor: localLimited && localLimitKbs === p.kbs ? '#3B82F6' : '#3A3A3C',
                  color: localLimited && localLimitKbs === p.kbs ? '#fff' : '#A1A1A6',
                  border: localLimited && localLimitKbs === p.kbs ? '1px solid #3B82F6' : '1px solid #48484A',
                }"
              >{{ p.label }}</button>
            </div>
            <div class="my-2" :style="{ borderTop: '1px solid #48484A' }"></div>
            <div class="flex items-center gap-2">
              <input v-model="customSpeed" placeholder="自定义"
                class="flex-1 rounded px-2 py-1 text-2xs outline-none tabular-nums"
                :style="{ backgroundColor: '#1C1C1E', border: '1px solid #48484A', color: '#F5F5F7' }"
                @change="localLimited = true; localLimitKbs = parseInt(customSpeed) || 512; applySpeedLimit()"
              />
              <span class="text-2xs">KB/s</span>
            </div>
          </div>
        </div>
      </div>

      <div :style="{ width: '1px', height: '12px', backgroundColor: '#48484A' }"></div>

      <!-- Shutdown -->
      <div class="relative">
        <button @click="showShutdownPopover = !showShutdownPopover"
          class="hover-bg-surface2 flex items-center gap-1 rounded px-1 py-0.5 transition-colors"
          :style="{ color: shutdownScheduled ? '#EF4444' : '#8E8E93' }"
        >
          <Power class="h-2.5 w-2.5" />
          <span>{{ shutdownScheduled ? '已预约' : '关机' }}</span>
        </button>
        <div v-if="showShutdownPopover"
          class="fixed z-50 rounded-lg shadow-lg"
          :style="{
            backgroundColor: '#2C2C2E', border: '1px solid #48484A',
            bottom: '32px', right: '12px', width: '240px',
          }"
        >
          <div class="p-3">
            <div class="mb-2 flex items-center justify-between">
              <span :style="{ color: '#F5F5F7' }" class="text-xs font-semibold">完成后关机</span>
            </div>
            <div class="flex flex-wrap gap-1.5">
              <button v-for="p in shutdownPresets" :key="p.minutes"
                @click="p.minutes === 0 ? store.shutdownSystem('shutdown') : scheduleShutdown(p.minutes)"
                class="rounded px-2 py-1 text-2xs transition-colors"
                :style="{
                  backgroundColor: '#3A3A3C', color: '#A1A1A6',
                  border: '1px solid #48484A',
                }"
              >{{ p.label }}</button>
            </div>
            <div class="my-2" :style="{ borderTop: '1px solid #48484A' }"></div>
            <div class="flex items-center gap-2">
              <input v-model="shutdownMinutes" placeholder="分钟"
                class="flex-1 rounded px-2 py-1 text-2xs outline-none tabular-nums"
                :style="{ backgroundColor: '#1C1C1E', border: '1px solid #48484A', color: '#F5F5F7' }"
                @change="scheduleShutdown(parseInt(shutdownMinutes) || 0)"
              />
              <span class="text-2xs">分钟后</span>
            </div>
          </div>
        </div>
      </div>

      <div :style="{ width: '1px', height: '12px', backgroundColor: '#48484A' }"></div>

      <!-- Feedback -->
      <button class="hover-bg-surface2 flex items-center gap-1 rounded px-1 py-0.5 transition-colors"
      >
        <MessageSquarePlus class="h-2.5 w-2.5" />
        <span>反馈</span>
      </button>
    </div>
  </footer>
</template>
