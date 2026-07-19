<script setup lang="ts">
import { ref } from "vue";
import { useRouter } from "vue-router";
import Sidebar from "./components/Sidebar.vue";
import StatusBar from "./components/StatusBar.vue";
import { Search, Settings } from "lucide-vue-next";

const router = useRouter();
const searchQuery = ref("");
const searchFocused = ref(false);
</script>

<template>
  <div class="flex h-screen flex-col" :style="{ backgroundColor: '#1C1C1E', color: '#F5F5F7' }">
    <div class="flex flex-1 overflow-hidden">
      <Sidebar />
      <div class="flex flex-1 flex-col overflow-hidden">
        <!-- Header Bar (40px) -->
        <header class="flex items-center gap-3 px-4" :style="{ height: '40px', borderBottom: '1px solid #48484A', backgroundColor: '#2C2C2E' }">
          <!-- Search bar -->
          <div class="relative flex-1" style="max-width: 320px;">
            <div class="flex items-center rounded-md px-2.5 transition-colors"
              :style="{
                backgroundColor: searchFocused ? 'rgba(59,130,246,0.08)' : '#1C1C1E',
                border: `1px solid ${searchFocused ? '#3B82F6' : '#48484A'}`,
                height: '30px',
              }"
            >
              <Search class="h-3.5 w-3.5 shrink-0" :style="{ color: searchFocused ? '#3B82F6' : '#8E8E93' }" />
              <input
                v-model="searchQuery"
                placeholder="搜索任务…"
                class="ml-2 flex-1 bg-transparent text-sm outline-none"
                :style="{ color: '#F5F5F7' }"
                @focus="searchFocused = true"
                @blur="searchFocused = false"
              />
              <span v-if="!searchFocused && !searchQuery"
                class="rounded px-1.5 py-0.5 text-2xs font-medium"
                :style="{ backgroundColor: '#3A3A3C', color: '#8E8E93', border: '1px solid #48484A' }"
              >Ctrl+F</span>
            </div>
          </div>

          <div class="flex-1" />

          <!-- Settings -->
          <button @click="router.push('/settings')"
            class="flex items-center justify-center rounded p-1.5 transition-colors hover-bg"
            :style="{ color: '#8E8E93', width: '36px', height: '36px' }"
          >
            <Settings class="h-4 w-4" />
          </button>
        </header>

        <main class="flex-1 overflow-auto">
          <router-view />
        </main>
      </div>
    </div>
    <StatusBar />
  </div>
</template>
