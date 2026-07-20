<script setup lang="ts">
import { ref, computed } from "vue";
import { useRouter, useRoute } from "vue-router";
import { useDownloadStore } from "../lib/store";
import {
  LayoutGrid, Download, CircleCheck, CirclePause, CircleAlert,
  Plus, ChevronDown, ChevronRight,
  Film, Music, FileText, ImageIcon, Package, Archive, File,
  Inbox, Clock, Tag, Circle, Pause, Play, SlidersHorizontal,
} from "lucide-vue-next";
import QueueManagerDialog from "./QueueManagerDialog.vue";

const router = useRouter();
const route = useRoute();
const store = useDownloadStore();

const queuesExpanded = ref(true);
const categoriesExpanded = ref(true);
const hoveredQueue = ref<string | null>(null);
const showQueueManager = ref(false);

const statusTabs = [
  { key: "all", icon: LayoutGrid, label: "全部", query: {} },
  { key: "downloading", icon: Download, label: "下载中", query: { status: "active" } },
  { key: "completed", icon: CircleCheck, label: "已完成", query: { status: "completed" } },
  { key: "paused", icon: CirclePause, label: "已暂停", query: { status: "paused" } },
  { key: "error", icon: CircleAlert, label: "错误", query: { status: "error" } },
];

const queues = computed(() => [
  { id: 'default', icon: Inbox, label: '默认', running: store.queueStates.default },
  { id: 'later', icon: Clock, label: '稍后下载', running: store.queueStates.later },
]);

const sidebarCategories = computed(() => {
  const cats = [
    { id: 'all', icon: Tag, label: '全部文件' },
    ...store.categories.filter(c => c.isBuiltin && c.id !== 'all' && c.visible).map(c => {
      const iconMap: Record<string, any> = {
        video: Film, audio: Music, document: FileText,
        image: ImageIcon, program: Package, archive: Archive,
      };
      return { id: c.id, icon: iconMap[c.id] || File, label: c.name };
    }),
  ];
  return cats;
});

const selectedQueue = ref('default');
const selectedCategory = ref('all');

const activeCount = computed(() => store.tasks.filter(t => t.status === 1).length);

function tabCount(key: string): number {
  const map: Record<string, string> = { all: "all", downloading: "active", completed: "completed", paused: "paused", error: "error" };
  const tab = store.filterTabs.find(t => t.id === map[key]);
  return tab ? tab.count : 0;
}

function isTabActive(tab: typeof statusTabs[number]): boolean {
  if (route.path !== '/tasks') return false;
  if (tab.query.status) return route.query.status === tab.query.status;
  return !route.query.status;
}

function navStyle(selected: boolean) {
  return selected
    ? { backgroundColor: 'rgba(59,130,246,0.18)', color: '#3B82F6', fontWeight: 500 }
    : { backgroundColor: 'transparent', color: '#A1A1A6', fontWeight: 400 };
}
</script>

<template>
  <aside :style="{ backgroundColor: '#2C2C2E', borderRight: '1px solid #48484A' }"
    class="flex w-56 flex-col overflow-hidden select-none">
    <div :style="{ borderBottom: '1px solid #48484A' }" class="flex items-center gap-2 px-4 py-3">
      <Download :style="{ color: '#3B82F6' }" class="h-5 w-5" />
      <span :style="{ color: '#F5F5F7' }" class="text-sm font-semibold">NS Download</span>
    </div>

    <nav class="flex-1 overflow-y-auto px-2 py-3">
      <!-- Status section header -->
      <div :style="{ color: '#8E8E93' }" class="mb-1 px-2 text-2xs font-semibold uppercase tracking-wider" style="letter-spacing: 0.5px;">状态</div>

      <!-- Status tabs -->
      <button
        v-for="tab in statusTabs" :key="tab.key"
        @click="store.setActiveFilter(tab.query.status || 'all'); router.push({ path: '/tasks', query: tab.query })"
        :class="['flex w-full items-center gap-2.5 rounded-md px-2.5 py-1.5 text-sm transition-colors', isTabActive(tab) ? '' : 'hover-bg']"
        :style="navStyle(isTabActive(tab))"
      >
        <span class="relative flex shrink-0">
          <component :is="tab.icon" class="h-4 w-4" />
          <span v-if="tab.key === 'downloading' && activeCount > 0"
            :style="{ backgroundColor: '#22C55E', borderColor: '#2C2C2E' }"
            class="absolute -right-1.5 -top-1.5 h-1.5 w-1.5 rounded-full border"
          ></span>
        </span>
        <span class="flex-1 text-left">{{ tab.label }}</span>
        <span :style="{ color: '#8E8E93' }" class="text-2xs tabular-nums">{{ tabCount(tab.key) }}</span>
      </button>

      <!-- Queues section -->
      <div class="mt-4 mb-1 px-2">
        <div class="hover-text-secondary flex cursor-pointer items-center justify-between rounded px-1 py-1 transition-colors"
          :style="{ color: '#8E8E93' }"
          @click="queuesExpanded = !queuesExpanded"
        >
          <div class="flex items-center gap-1">
            <component :is="queuesExpanded ? ChevronDown : ChevronRight" class="h-3 w-3" />
            <span class="text-2xs font-semibold uppercase tracking-wider" style="letter-spacing: 0.5px;">队列</span>
          </div>
          <button @click.stop="showQueueManager = true" class="rounded p-0.5 transition-colors"><Plus class="h-3 w-3" /></button>
        </div>
      </div>
      <template v-if="queuesExpanded">
        <div
          v-for="q in queues" :key="q.id"
          @click="selectedQueue = q.id"
          @mouseenter="hoveredQueue = q.id"
          @mouseleave="hoveredQueue = null"
          :class="['flex items-center gap-2 rounded-md px-2 py-1 text-sm transition-colors', selectedQueue === q.id ? '' : 'hover-bg']"
          :style="{ height: '32px', margin: '1px 8px', ...navStyle(selectedQueue === q.id) }"
        >
          <component :is="q.icon" class="h-3.5 w-3.5 shrink-0" />
          <span class="flex-1 truncate text-xs">{{ q.label }}</span>

          <template v-if="hoveredQueue === q.id">
            <span class="flex gap-0.5">
              <button @click.stop="store.toggleQueue(q.id)" class="rounded p-0.5" :style="{ color: '#8E8E93' }">
                <component :is="q.running ? Pause : Play" class="h-3 w-3" />
              </button>
              <button @click.stop="showQueueManager = true" class="rounded p-0.5" :style="{ color: '#8E8E93' }">
                <SlidersHorizontal class="h-3 w-3" />
              </button>
            </span>
          </template>
          <template v-else>
            <span :style="{ color: '#8E8E93' }" class="text-2xs tabular-nums">{{ store.queueTaskCounts[q.id] ?? 0 }}</span>
          </template>

          <Circle :style="{ color: q.running ? '#22C55E' : '#8E8E93' }" class="h-1.5 w-1.5 fill-current" />
        </div>
      </template>

      <!-- Categories section -->
      <div class="mt-4 mb-1 px-2">
        <div class="hover-text-secondary flex cursor-pointer items-center justify-between rounded px-1 py-1 transition-colors"
          :style="{ color: '#8E8E93' }"
          @click="categoriesExpanded = !categoriesExpanded"
        >
          <div class="flex items-center gap-1">
            <component :is="categoriesExpanded ? ChevronDown : ChevronRight" class="h-3 w-3" />
            <span class="text-2xs font-semibold uppercase tracking-wider" style="letter-spacing: 0.5px;">分类</span>
          </div>
        </div>
      </div>
      <template v-if="categoriesExpanded">
        <div
          v-for="cat in sidebarCategories" :key="cat.id"
          @click="selectedCategory = cat.id; store.setCategoryFilter(cat.id)"
          :class="['flex items-center gap-2 rounded-md px-2 py-1 text-sm transition-colors', selectedCategory === cat.id ? '' : 'hover-bg']"
          :style="{ height: '32px', margin: '1px 8px', ...navStyle(selectedCategory === cat.id) }"
        >
          <component :is="cat.icon" class="h-3.5 w-3.5 shrink-0" />
          <span class="flex-1 truncate text-xs">{{ cat.label }}</span>
          <span :style="{ color: '#8E8E93' }" class="text-2xs tabular-nums">{{ store.categoryCount(cat.id) }}</span>
        </div>
      </template>
    </nav>
  </aside>
  <QueueManagerDialog v-if="showQueueManager" @close="showQueueManager = false" />
</template>
