<script setup lang="ts">
import { ref, onMounted } from "vue";
import { X, Plus, Trash2, Edit3, Check, Circle } from "lucide-vue-next";

const emit = defineEmits<{ close: [] }>();

interface QueueItem {
  id: string;
  label: string;
  running: boolean;
  editing?: boolean;
}

const queues = ref<QueueItem[]>([
  { id: 'default', label: '默认', running: true },
  { id: 'later', label: '稍后下载', running: false },
]);

const newName = ref("");
const editText = ref("");

function addQueue() {
  const name = newName.value.trim();
  if (!name) return;
  const id = `queue_${Date.now()}`;
  queues.value.push({ id, label: name, running: true });
  newName.value = "";
}

function startEdit(q: QueueItem) {
  q.editing = true;
  editText.value = q.label;
}

function saveEdit(q: QueueItem) {
  const name = editText.value.trim();
  if (name) q.label = name;
  q.editing = false;
}

function removeQueue(id: string) {
  if (id === 'default') return;
  const idx = queues.value.findIndex(q => q.id === id);
  if (idx >= 0) queues.value.splice(idx, 1);
}

function toggleRunning(id: string) {
  const q = queues.value.find(q => q.id === id);
  if (q) q.running = !q.running;
}
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center"
    :style="{ backgroundColor: 'rgba(0,0,0,0.5)' }"
    @click.self="emit('close')"
  >
    <div :style="{ backgroundColor: '#2C2C2E', border: '1px solid #48484A', borderRadius: '12px', width: '380px', maxHeight: '480px' }"
      class="flex flex-col overflow-hidden shadow-2xl"
    >
      <div class="flex items-center justify-between px-4 py-3" :style="{ borderBottom: '1px solid #48484A' }">
        <span class="text-sm font-semibold" :style="{ color: '#F5F5F7' }">队列管理</span>
        <button @click="emit('close')" class="rounded p-1 transition-colors" :style="{ color: '#8E8E93' }">
          <X class="h-4 w-4" />
        </button>
      </div>

      <div class="flex-1 overflow-y-auto px-3 py-2">
        <div v-for="q in queues" :key="q.id"
          :style="{ borderBottom: '1px solid #3A3A3C' }"
          class="flex items-center gap-2 px-2 py-2"
        >
          <Circle :style="{ color: q.running ? '#22C55E' : '#8E8E93' }" class="h-2 w-2 fill-current shrink-0" />
          <template v-if="q.editing">
            <input v-model="editText" @keyup.enter="saveEdit(q)" @keyup.escape="q.editing = false"
              class="flex-1 rounded bg-transparent px-1 py-0.5 text-sm outline-none"
              :style="{ color: '#F5F5F7', border: '1px solid #3B82F6' }"
              autofocus
            />
            <button @click="saveEdit(q)" class="rounded p-1" :style="{ color: '#22C55E' }"><Check class="h-3.5 w-3.5" /></button>
          </template>
          <template v-else>
            <span class="flex-1 text-sm" :style="{ color: '#F5F5F7' }">{{ q.label }}</span>
            <div class="flex gap-1">
              <button @click="toggleRunning(q.id)" class="rounded p-1 transition-colors" :style="{ color: '#8E8E93' }">
                <component :is="q.running ? 'span' : 'span'" class="text-2xs">{{ q.running ? '暂停' : '启动' }}</component>
              </button>
              <button v-if="q.id !== 'default'" @click="startEdit(q)" class="rounded p-1 transition-colors" :style="{ color: '#8E8E93' }">
                <Edit3 class="h-3.5 w-3.5" />
              </button>
              <button v-if="q.id !== 'default'" @click="removeQueue(q.id)" class="rounded p-1 transition-colors" :style="{ color: '#EF4444' }">
                <Trash2 class="h-3.5 w-3.5" />
              </button>
            </div>
          </template>
        </div>
      </div>

      <div class="flex items-center gap-2 px-3 py-2" :style="{ borderTop: '1px solid #48484A' }">
        <input v-model="newName" @keyup.enter="addQueue"
          placeholder="新建队列名称…"
          class="flex-1 rounded bg-transparent px-2 py-1.5 text-sm outline-none"
          :style="{ color: '#F5F5F7', border: '1px solid #48484A' }"
        />
        <button @click="addQueue" class="flex items-center gap-1 rounded-md px-3 py-1.5 text-sm font-medium transition-colors"
          :style="{ backgroundColor: '#3B82F6', color: '#fff' }">
          <Plus class="h-3.5 w-3.5" /> 添加
        </button>
      </div>
    </div>
  </div>
</template>
