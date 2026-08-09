<script setup lang="ts">
import { ref, onMounted } from "vue";
import { X, Plus, Trash2, Edit3, Check, Circle, Play, Pause } from "lucide-vue-next";
import { useDownloadStore } from "../lib/store";

const emit = defineEmits<{ close: [] }>();
const store = useDownloadStore();

interface QueueItem {
  id: string;
  label: string;
  running: boolean;
  editing?: boolean;
}

const queues = ref<QueueItem[]>([]);
const newName = ref("");
const editText = ref("");

onMounted(() => {
  syncFromStore();
});

function syncFromStore() {
  queues.value = Object.entries(store.queueStates).map(([id, running]) => ({
    id, label: store.queueLabels[id] || id, running,
  }));
}

function syncToStore() {
  const s: Record<string, boolean> = {};
  const l: Record<string, string> = {};
  for (const q of queues.value) {
    s[q.id] = q.running;
    l[q.id] = q.label;
  }
  store.queueStates = s;
  store.queueLabels = l;
  store.saveQueues();
}

function addQueue() {
  const name = newName.value.trim();
  if (!name) return;
  const id = `queue_${Date.now()}`;
  queues.value.push({ id, label: name, running: true });
  newName.value = "";
  syncToStore();
}

function startEdit(q: QueueItem) {
  q.editing = true;
  editText.value = q.label;
}

function saveEdit(q: QueueItem) {
  const name = editText.value.trim();
  if (name) q.label = name;
  q.editing = false;
  syncToStore();
}

function removeQueue(id: string) {
  if (id === 'default') return;
  const idx = queues.value.findIndex(q => q.id === id);
  if (idx >= 0) queues.value.splice(idx, 1);
  syncToStore();
}

function toggleRunning(id: string) {
  const q = queues.value.find(q => q.id === id);
  if (q) q.running = !q.running;
  syncToStore();
}
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center"
    :style="{ backgroundColor: 'rgba(0,0,0,0.5)' }"
    @click.self="emit('close')"
  >
    <div :style="{ backgroundColor: '#261C14', border: '1px solid #5A4330', borderRadius: '12px', width: '380px', maxHeight: '480px' }"
      class="flex flex-col overflow-hidden shadow-2xl"
    >
      <div class="flex items-center justify-between px-4 py-3" :style="{ borderBottom: '1px solid #5A4330' }">
        <span class="text-sm font-semibold" :style="{ color: '#EDE0C8' }">队列管理</span>
        <button @click="emit('close')" class="rounded p-1 transition-colors" :style="{ color: '#9C8260' }">
          <X class="h-4 w-4" />
        </button>
      </div>

      <div class="flex-1 overflow-y-auto px-3 py-2">
        <div v-for="q in queues" :key="q.id"
          :style="{ borderBottom: '1px solid #5A4330' }"
          class="flex items-center gap-2 px-2 py-2"
        >
          <Circle :style="{ color: q.running ? '#4E7A5A' : '#9C8260' }" class="h-2 w-2 fill-current shrink-0" />
          <template v-if="q.editing">
            <input v-model="editText" @keyup.enter="saveEdit(q)" @keyup.escape="q.editing = false"
              class="flex-1 rounded bg-transparent px-1 py-0.5 text-sm outline-none"
              :style="{ color: '#EDE0C8', border: '1px solid var(--accent)' }"
              autofocus
            />
            <button @click="saveEdit(q)" class="rounded p-1" :style="{ color: '#4E7A5A' }"><Check class="h-3.5 w-3.5" /></button>
          </template>
          <template v-else>
            <span class="flex-1 text-sm" :style="{ color: '#EDE0C8' }">{{ q.label }}</span>
            <div class="flex gap-1">
              <button @click="toggleRunning(q.id)" class="rounded p-1 transition-colors" :style="{ color: '#9C8260' }">
                <component :is="q.running ? Pause : Play" class="h-3.5 w-3.5" />
              </button>
              <button v-if="q.id !== 'default'" @click="startEdit(q)" class="rounded p-1 transition-colors" :style="{ color: '#9C8260' }">
                <Edit3 class="h-3.5 w-3.5" />
              </button>
              <button v-if="q.id !== 'default'" @click="removeQueue(q.id)" class="rounded p-1 transition-colors" :style="{ color: '#B53A2E' }">
                <Trash2 class="h-3.5 w-3.5" />
              </button>
            </div>
          </template>
        </div>
      </div>

      <div class="flex items-center gap-2 px-3 py-2" :style="{ borderTop: '1px solid #5A4330' }">
        <input v-model="newName" @keyup.enter="addQueue"
          placeholder="新建队列名称…"
          class="flex-1 rounded bg-transparent px-2 py-1.5 text-sm outline-none"
          :style="{ color: '#EDE0C8', border: '1px solid #5A4330' }"
        />
        <button @click="addQueue" class="flex items-center gap-1 rounded-md px-3 py-1.5 text-sm font-medium transition-colors"
          :style="{ backgroundColor: 'var(--accent)', color: '#EDE0C8' }">
          <Plus class="h-3.5 w-3.5" /> 添加
        </button>
      </div>
    </div>
  </div>
</template>
