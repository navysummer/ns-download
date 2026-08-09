<script setup lang="ts">
import { ref } from "vue";
import { useDownloadStore } from "../lib/store";
import { X, Plus, Trash2, Edit3, Check, File } from "lucide-vue-next";

const emit = defineEmits<{ close: [] }>();
const store = useDownloadStore();

const newName = ref("");
const newExts = ref("");
const editingId = ref<string | null>(null);
const editName = ref("");
const editExts = ref("");

function addCategory() {
  const name = newName.value.trim();
  if (!name) return;
  const exts = newExts.value.split(/[,;\s]+/).filter(Boolean);
  store.addCustomCategory(name, exts);
  newName.value = "";
  newExts.value = "";
}

function startEdit(id: string) {
  const cat = store.categories.find(c => c.id === id);
  if (!cat || cat.isBuiltin) return;
  editingId.value = id;
  editName.value = cat.name;
  editExts.value = cat.extensions.join(", ");
}

function saveEdit(id: string) {
  const cat = store.categories.find(c => c.id === id);
  if (!cat || cat.isBuiltin) return;
  const name = editName.value.trim();
  if (name) cat.name = name;
  cat.extensions = editExts.value.split(/[,;\s]+/).filter(Boolean);
  editingId.value = null;
}

function toggleVisible(id: string) {
  store.toggleCategoryVisibility(id);
}
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center"
    :style="{ backgroundColor: 'rgba(0,0,0,0.5)' }"
    @click.self="emit('close')"
  >
    <div :style="{ backgroundColor: 'var(--surface-bg)', border: '1px solid var(--surface-border)', borderRadius: '12px', width: '420px', maxHeight: '520px' }"
      class="flex flex-col overflow-hidden shadow-2xl"
    >
      <div class="flex items-center justify-between px-4 py-3" :style="{ borderBottom: '1px solid var(--surface-border)' }">
        <span class="text-sm font-semibold" :style="{ color: 'var(--text-primary)' }">分类管理</span>
        <button @click="emit('close')" class="rounded p-1 transition-colors" :style="{ color: 'var(--text-muted)' }">
          <X class="h-4 w-4" />
        </button>
      </div>

      <div class="flex-1 overflow-y-auto px-3 py-2">
        <div v-for="cat in store.categories.filter(c => c.id !== 'all')" :key="cat.id"
          :style="{ borderBottom: '1px solid var(--surface-border)' }"
          class="flex items-center gap-2 px-2 py-2"
        >
          <File class="h-3.5 w-3.5 shrink-0 cursor-pointer" :style="{ color: cat.visible ? 'var(--accent)' : 'var(--text-muted)' }" @click="store.toggleCategoryVisibility(cat.id)" />
          <template v-if="editingId === cat.id">
            <div class="flex-1 space-y-1">
              <input v-model="editName" @keyup.enter="saveEdit(cat.id)"
                class="w-full rounded bg-transparent px-1 py-0.5 text-sm outline-none"
                :style="{ color: 'var(--text-primary)', border: '1px solid var(--accent)' }"
                placeholder="分类名称"
              />
              <input v-model="editExts" @keyup.enter="saveEdit(cat.id)"
                class="w-full rounded bg-transparent px-1 py-0.5 text-2xs outline-none"
                :style="{ color: 'var(--text-secondary)', border: '1px solid var(--surface-border)' }"
                placeholder="扩展名，用逗号分隔"
              />
            </div>
            <button @click="saveEdit(cat.id)" class="rounded p-1" :style="{ color: 'var(--success)' }"><Check class="h-3.5 w-3.5" /></button>
          </template>
          <template v-else>
            <span class="flex-1 text-sm" :style="{ color: cat.visible ? 'var(--text-primary)' : 'var(--text-muted)' }">{{ cat.name }}</span>
            <span class="text-2xs" :style="{ color: 'var(--text-muted)' }">{{ cat.extensions.length }} 个扩展名</span>
            <button v-if="!cat.isBuiltin" @click="startEdit(cat.id)" class="rounded p-1" :style="{ color: 'var(--text-muted)' }"><Edit3 class="h-3.5 w-3.5" /></button>
            <button v-if="!cat.isBuiltin" @click="store.removeCategory(cat.id)" class="rounded p-1" :style="{ color: '#D64531' }"><Trash2 class="h-3.5 w-3.5" /></button>
          </template>
        </div>
      </div>

      <div class="space-y-1.5 px-3 py-2" :style="{ borderTop: '1px solid var(--surface-border)' }">
        <input v-model="newName" placeholder="分类名称"
          class="w-full rounded bg-transparent px-2 py-1.5 text-sm outline-none"
          :style="{ color: 'var(--text-primary)', border: '1px solid var(--surface-border)' }"
        />
        <div class="flex gap-2">
          <input v-model="newExts" placeholder="扩展名 (mp4, avi, mkv)"
            class="flex-1 rounded bg-transparent px-2 py-1.5 text-2xs outline-none"
            :style="{ color: 'var(--text-secondary)', border: '1px solid var(--surface-border)' }"
          />
          <button @click="addCategory" class="flex items-center gap-1 rounded-md px-3 py-1 text-xs font-medium transition-colors"
            :style="{ backgroundColor: 'var(--accent)', color: 'var(--text-primary)' }">
            <Plus class="h-3 w-3" /> 添加
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
