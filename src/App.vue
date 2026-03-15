<template>
  <div class="h-full w-full app-shell">
    <div class="content-shell">
    <aside class="panel-nav shrink-0 p-4 border-r border-white/10 bg-panel/35 backdrop-blur-md flex flex-col min-h-0">
      <div class="flex items-center gap-3 px-2">
        <div class="h-9 w-9 rounded-xl bg-purple-600/90 flex items-center justify-center font-bold">OC</div>
        <div class="leading-tight">
          <div class="font-semibold">OpenClaw</div>
          <div class="text-xs text-subtext">macOS Console</div>
        </div>
      </div>

      <div class="mt-6 space-y-1">
        <NavItem label="本机状态" :active="activeView === 'local'" @select="switchView('local')" />
        <NavItem label="会话面板" :active="activeView === 'sessions'" @select="switchView('sessions')" />
        <NavItem label="Agent 面板" :active="activeView === 'agents'" @select="switchView('agents')" />
        <NavItem label="Cron 面板" :active="activeView === 'cron'" @select="switchView('cron')" />
        <NavItem label="Skill 面板" :active="activeView === 'skills'" @select="switchView('skills')" />
        <NavItem label="日志面板" :active="activeView === 'gatewayLogs'" @select="switchView('gatewayLogs')" />
      </div>

      <div class="mt-auto pt-6">
        <div class="p-3 rounded-xl border border-white/10 bg-white/5 text-xs text-subtext">
          macOS App Only
        </div>
      </div>
    </aside>

    <section v-if="activeView === 'sessions'" class="panel-sessions shrink-0 p-4 border-r border-white/10 bg-white/[0.02] h-full min-h-0 flex flex-col">
      <div class="flex items-center justify-between mb-3">
        <div class="font-semibold">Sessions</div>
        <div class="flex items-center gap-2">
          <button class="btn" @click="refreshSessions">刷新列表</button>
        </div>
      </div>

      <div class="flex items-center gap-2 mb-3">
        <input v-model="sessionQuery" class="input" placeholder="Session ID..." />
      </div>

      <div class="space-y-2 overflow-auto pr-1 flex-1 min-h-0">
        <div class="text-[11px] text-subtext px-1">今天</div>
        <button
          v-for="s in groupedSessions.today"
          :key="s.sessionKey"
          class="w-full text-left p-3 rounded-xl border transition"
          :class="selected?.sessionKey === s.sessionKey ? 'border-purple-500/60 bg-purple-600/10' : 'border-white/10 bg-white/5 hover:bg-white/7'"
          @click="selectSession(s)"
        >
          <div class="font-medium truncate">{{ s.label || s.sessionKey }}</div>
          <div class="text-xs text-subtext truncate">{{ s.sessionKey }}</div>
          <div class="mt-1 flex items-center justify-between text-[11px] text-subtext">
            <span>{{ sessionKindLabel(s) }}</span>
            <span>{{ formatTimeLabel(s.updatedAt) }}</span>
          </div>
        </button>
        <div v-if="groupedSessions.today.length === 0" class="text-xs text-subtext px-1">暂无</div>

        <div class="text-[11px] text-subtext px-1 pt-2">过往</div>
        <button
          v-for="s in groupedSessions.past"
          :key="s.sessionKey"
          class="w-full text-left p-3 rounded-xl border transition"
          :class="selected?.sessionKey === s.sessionKey ? 'border-purple-500/60 bg-purple-600/10' : 'border-white/10 bg-white/5 hover:bg-white/7'"
          @click="selectSession(s)"
        >
          <div class="font-medium truncate">{{ s.label || s.sessionKey }}</div>
          <div class="text-xs text-subtext truncate">{{ s.sessionKey }}</div>
          <div class="mt-1 flex items-center justify-between text-[11px] text-subtext">
            <span>{{ sessionKindLabel(s) }}</span>
            <span>{{ formatTimeLabel(s.updatedAt) }}</span>
          </div>
        </button>
        <div v-if="groupedSessions.past.length === 0" class="text-xs text-subtext px-1">暂无</div>
      </div>
    </section>

    <main v-if="activeView === 'sessions'" class="flex-1 min-w-0 p-4 bg-black/10 h-full min-h-0 flex flex-col">
      <div class="flex items-start justify-between mb-3 gap-3">
        <div class="min-w-0 flex-1">
          <div class="font-semibold">Debug Console</div>
          <div v-if="lastError" class="text-xs text-red-300 mt-1 whitespace-pre-wrap break-all max-w-[900px]">{{ lastError }}</div>
        </div>
        <div class="shrink-0 flex flex-nowrap items-center gap-2 mt-0.5">
          <button v-if="messageView!=='chat'" class="btn" @click="messageView='chat'">对话视图</button>
          <button v-if="messageView!=='debug'" class="btn" @click="messageView='debug'">调试视图</button>
          <button v-if="messageView==='chat'" class="btn" @click="chatOnlyText = !chatOnlyText">
            {{ chatOnlyText ? '显示工具块' : '仅文本' }}
          </button>
          <span class="text-xs text-subtext">Auto refresh</span>
          <button class="btn" @click="togglePolling">{{ polling ? '暂停' : '继续' }}</button>
        </div>
      </div>

      <div class="rounded-2xl border border-white/10 bg-white/5 shadow-soft overflow-hidden min-w-0 flex-1 min-h-0 flex flex-col">
        <div class="flex-1 min-h-0 overflow-auto p-4 space-y-3 min-w-0" ref="scrollRef">
          <div v-if="!selected" class="text-subtext">选择一个 session 开始查看对话。</div>
          <template v-else>
            <div
              v-for="(m, idx) in renderedMessages"
              :key="idx"
              class="flex"
              :class="messageView === 'chat' && m.role === 'user' ? 'justify-end' : 'justify-start'"
            >
              <div class="min-w-0 rounded-[16px] px-4 py-2.5 border shadow-sm"
                   :class="messageView === 'debug' ? 'w-full bubble-debug' : (m.role==='user' ? 'max-w-[72%] bubble-user' : 'max-w-[72%] bubble-assistant')">
                <template v-if="messageView === 'chat' && (m.kind === 'toolCall' || m.kind === 'toolResult')">
                  <details class="text-xs">
                    <summary class="cursor-pointer select-none">
                      {{ m.kind === 'toolCall' ? 'Tool Call' : 'Tool Result' }}: {{ m.toolName || 'tool' }}
                    </summary>
                    <pre class="mt-2 whitespace-pre-wrap break-all text-[11px] text-subtext">{{ m.toolPayload || m.text }}</pre>
                  </details>
                </template>
                <div v-else class="whitespace-pre-wrap break-all"
                     :class="messageView === 'debug' ? 'text-xs font-mono leading-5' : 'text-sm'">{{ m.text }}</div>
                <div class="text-[10px] mt-1.5 opacity-70" v-if="m.meta">{{ formatClock(m.meta) }}</div>
              </div>
            </div>
          </template>
        </div>

        <div class="p-3 border-t border-white/10 flex items-center gap-2 bg-black/10">
          <input v-model="draft" class="input flex-1" placeholder="Message to selected session... (Cmd+Enter)" @keydown.enter="send" />
          <button class="btn-primary" :disabled="sending || !selected || !draft.trim()" @click="send">
            {{ sending ? '发送中...' : '发送' }}
          </button>
        </div>
      </div>
    </main>

    <aside v-if="activeView === 'sessions'" class="panel-runtime shrink-0 p-4 border-l border-white/10 bg-white/[0.02] h-full min-h-0 flex flex-col">
      <div class="font-semibold mb-3">Local Runtime</div>
      <div class="p-3 rounded-xl border border-white/10 bg-white/5 space-y-2">
        <InfoRow label="OPENCLAW CLI" :value="status.openclawFound ? 'FOUND' : 'MISSING'" />
        <InfoRow label="VERSION" :value="status.openclawVersion || '-'" />
        <InfoRow label="SESSIONS FILE" :value="status.sessionsFileExists ? 'READY' : 'NOT FOUND'" />
        <InfoRow label="FILE READABLE" :value="status.sessionsFileReadable ? 'YES' : 'NO'" />
        <InfoRow label="SESSIONS COUNT" :value="String(status.sessionsCount)" />
        <InfoRow label="LATEST ACTIVE" :value="formatTimeLabel(status.latestSessionUpdatedAt)" />
      </div>

      <div class="mt-3 p-3 rounded-xl border border-white/10 bg-black/20">
        <div class="text-[11px] text-subtext mb-1">OPENCLAW PATH</div>
        <div class="text-xs break-all">{{ status.openclawPath || '-' }}</div>
      </div>

      <div class="mt-3 p-3 rounded-xl border border-white/10 bg-black/20">
        <div class="text-[11px] text-subtext mb-1">RUNTIME ISSUES</div>
        <div v-if="status.errors.length === 0" class="text-xs text-green-300">No blocking issues</div>
        <div v-for="(err, i) in status.errors" :key="i" class="text-xs text-amber-300 break-words">{{ err }}</div>
      </div>

      <div class="mt-3 p-3 rounded-xl border border-white/10 bg-black/20 space-y-2">
        <InfoRow label="SELECTED KEY" :value="selected?.sessionKey || '-'" />
        <InfoRow label="SELECTED ID" :value="selected?.sessionId || '-'" />
        <InfoRow label="MODEL" :value="selected?.model || '-'" />
        <InfoRow label="PROVIDER" :value="selected?.modelProvider || '-'" />
        <InfoRow label="UPDATED" :value="formatTimeLabel(selected?.updatedAt)" />
        <div class="pt-1">
          <div class="text-[11px] text-subtext mb-1">SWITCH MODEL</div>
          <div class="flex items-center gap-2 min-w-0">
            <select
              v-model="selectedModelDraft"
              class="input text-xs flex-1 min-w-0"
              :disabled="!selected || sending || modelOptionsLoading"
            >
              <option value="">跟随当前会话</option>
              <option v-for="m in modelOptions" :key="m.value" :value="m.value">
                {{ m.label }}
              </option>
            </select>
            <button class="btn shrink-0 whitespace-nowrap" :disabled="!selected || sending" @click="applySelectedModel">
              应用
            </button>
          </div>
        </div>
      </div>

      <div class="mt-4 font-semibold mb-3">Session Log</div>
      <div class="mb-2 flex items-center gap-2">
        <button class="btn" @click="logFilter='all'">全部</button>
        <button class="btn" @click="logFilter='tool'">工具</button>
        <button class="btn" @click="logFilter='warn'">异常</button>
        <button class="btn" @click="logFilter='info'">信息</button>
      </div>
      <div class="p-3 rounded-xl border border-white/10 bg-white/5 overflow-auto flex-1 min-h-0">
        <div v-if="filteredLogs.length===0" class="text-subtext">暂无日志（工具调用/事件将显示在这里）。</div>
        <div v-for="(l, i) in filteredLogs" :key="i" class="text-xs">
          <div
            class="flex items-center gap-2 py-1 rounded px-1 cursor-pointer hover:bg-white/5 transition-colors"
            :title="l.text"
            @click="expandedLogIndex = expandedLogIndex === i ? -1 : i"
          >
            <span class="text-subtext">{{ l.ts }}</span>
            <span class="text-green-400" v-if="l.level==='info'">•</span>
            <span class="text-purple-400" v-else-if="l.level==='tool'">•</span>
            <span class="text-amber-300" v-else>•</span>
            <span class="truncate">{{ l.text }}</span>
          </div>
          <div v-if="expandedLogIndex === i" class="pl-[64px] pr-2 pb-2 text-[11px] text-subtext whitespace-pre-wrap break-all">
            {{ l.text }}
          </div>
        </div>
      </div>
    </aside>

    <main v-else-if="activeView === 'agents'" class="flex-1 p-6 bg-black/10 overflow-auto">
      <div class="max-w-[980px] mx-auto">
        <div class="flex items-center justify-between mb-4">
          <div>
            <div class="text-xl font-semibold">Agent 面板</div>
            <div class="text-xs text-subtext">与会话面板同级，查看 agent 运行状态与活跃度</div>
          </div>
          <button class="btn" :disabled="agentPanelLoading" @click="refreshAgentPanel">
            {{ agentPanelLoading ? '加载中...' : '刷新 Agent 状态' }}
          </button>
        </div>

        <div class="p-4 rounded-xl border border-white/10 bg-white/5 mb-3">
          <InfoRow label="DEFAULT AGENT" :value="agentPanel.defaultAgentId || '-'" />
        </div>

        <div class="p-4 rounded-xl border border-white/10 bg-black/20">
          <div v-if="agentPanelLoading" class="text-sm text-subtext">Agent 状态加载中...</div>
          <div v-else-if="agentPanel.items.length===0" class="text-sm text-subtext">暂无 Agent 数据</div>
          <div v-for="a in agentPanel.items" :key="a.agentId" class="py-3 border-b border-white/10 last:border-b-0">
            <div class="flex items-center justify-between">
              <div class="text-sm font-medium">{{ a.agentId }}</div>
              <div class="text-xs" :class="a.running ? 'text-green-300' : 'text-subtext'">{{ a.status }}</div>
            </div>
            <div class="text-xs text-subtext mt-1">
              sessions: {{ a.sessionsCount }} | latest: {{ formatTimeLabel(a.latestUpdatedAt) }} | bootstrap: {{ a.bootstrapPending ? 'pending' : 'ready' }}
            </div>
          </div>
        </div>

        <div class="mt-3 p-4 rounded-xl border border-white/10 bg-black/20">
          <div class="text-[11px] text-subtext mb-2">AGENT ISSUES</div>
          <div v-if="agentPanel.errors.length === 0" class="text-sm text-green-300">No blocking issues</div>
          <div v-for="(err, i) in agentPanel.errors" :key="i" class="text-sm text-amber-300 break-words py-1">{{ err }}</div>
        </div>
      </div>
    </main>

    <main v-else-if="activeView === 'cron'" class="flex-1 p-6 bg-black/10 overflow-auto">
      <div class="max-w-[1180px] mx-auto">
        <div class="flex items-center justify-between mb-4 gap-3">
          <div>
            <div class="text-xl font-semibold">Cron 面板</div>
            <div class="text-xs text-subtext">定时任务健康中心：调度、执行、失败与风险一屏查看</div>
          </div>
          <button class="btn" :disabled="cronLoading" @click="refreshCronPanel">
            {{ cronLoading ? '刷新中...' : '刷新 Cron 状态' }}
          </button>
        </div>

        <div class="grid grid-cols-5 gap-3 mb-3">
          <div class="p-3 rounded-xl border border-white/10 bg-white/5">
            <div class="text-xs text-subtext">TOTAL JOBS</div>
            <div class="text-lg font-semibold">{{ cronSummary.total }}</div>
          </div>
          <div class="p-3 rounded-xl border border-white/10 bg-white/5">
            <div class="text-xs text-subtext">ENABLED</div>
            <div class="text-lg font-semibold">{{ cronSummary.enabled }}</div>
          </div>
          <div class="p-3 rounded-xl border border-white/10 bg-white/5">
            <div class="text-xs text-subtext">PAUSED</div>
            <div class="text-lg font-semibold">{{ cronSummary.disabled }}</div>
          </div>
          <div class="p-3 rounded-xl border border-white/10 bg-white/5">
            <div class="text-xs text-subtext">FAIL (24H)</div>
            <div class="text-lg font-semibold">{{ cronSummary.fail24h }}</div>
          </div>
          <div class="p-3 rounded-xl border border-white/10 bg-white/5">
            <div class="text-xs text-subtext">SUCCESS RATE (24H)</div>
            <div class="text-lg font-semibold">{{ cronSummary.successRate24h }}</div>
          </div>
        </div>

        <div class="p-3 rounded-xl border border-white/10 bg-black/20 mb-3 grid grid-cols-[1fr_auto_auto_auto] gap-2 items-center">
          <input v-model="cronSearch" class="input" placeholder="搜索任务 ID / 名称 / 调度 / 工作区..." />
          <select v-model="cronStatusFilter" class="input text-xs w-[140px]">
            <option value="all">全部状态</option>
            <option value="enabled">仅启用</option>
            <option value="disabled">仅暂停</option>
            <option value="failing">仅异常</option>
          </select>
          <label class="text-xs text-subtext flex items-center gap-2">
            <input v-model="cronRiskOnly" type="checkbox" />
            仅显示风险任务
          </label>
          <button class="btn" :disabled="cronLoading" @click="refreshCronPanel">应用</button>
        </div>

        <div class="grid grid-cols-[1.2fr_1fr] gap-3 mb-3">
          <div class="rounded-xl border border-white/10 bg-black/20 overflow-hidden">
            <div class="px-4 py-3 border-b border-white/10 text-sm font-medium">任务列表</div>
            <div class="max-h-[430px] overflow-auto">
              <div v-if="cronLoading" class="p-4 text-sm text-subtext">加载中...</div>
              <div v-else-if="filteredCronJobs.length === 0" class="p-4 text-sm text-subtext">没有匹配的任务</div>
              <div
                v-for="job in filteredCronJobs"
                :key="job.id"
                class="px-4 py-3 border-b border-white/10 cursor-pointer transition-colors hover:bg-white/5"
                :class="selectedCronId === job.id ? 'bg-white/8' : ''"
                @click="selectCronJob(job.id)"
              >
                <div class="flex items-center justify-between gap-2">
                  <div class="text-sm font-medium truncate">{{ job.name || job.id }}</div>
                  <div class="text-[11px] px-2 py-0.5 rounded border border-white/20"
                       :class="job.enabled ? 'text-green-300' : 'text-amber-300'">
                    {{ job.enabled ? 'enabled' : 'paused' }}
                  </div>
                </div>
                <div class="text-xs text-subtext mt-1 truncate">{{ job.id }}</div>
                <div class="text-xs text-subtext mt-1 truncate">
                  {{ job.schedule || '-' }} · next: {{ formatTimeLabel(job.nextRunAt) }}
                </div>
                <div class="mt-2 flex items-center gap-2 flex-wrap">
                  <span v-for="flag in job.riskFlags" :key="`${job.id}-${flag}`"
                        class="text-[10px] px-1.5 py-0.5 rounded bg-amber-500/20 text-amber-200 border border-amber-400/40">
                    {{ flag }}
                  </span>
                </div>
                <div class="mt-2 flex items-center gap-2">
                  <button
                    class="btn text-xs"
                    :disabled="!!cronActionLoading"
                    @click.stop="runCronJob(job.id)"
                  >立即执行</button>
                  <button
                    class="btn text-xs"
                    :disabled="!!cronActionLoading"
                    @click.stop="toggleCronJob(job.id, !job.enabled)"
                  >{{ job.enabled ? '暂停' : '启用' }}</button>
                  <button
                    class="btn text-xs text-amber-200"
                    :disabled="!!cronActionLoading"
                    @click.stop="removeCronJob(job.id)"
                  >删除</button>
                </div>
              </div>
            </div>
          </div>

          <div class="rounded-xl border border-white/10 bg-black/20 overflow-hidden">
            <div class="px-4 py-3 border-b border-white/10 text-sm font-medium">任务详情</div>
            <div v-if="!selectedCronJob" class="p-4 text-sm text-subtext">选择左侧任务查看详情与运行记录</div>
            <div v-else class="p-4 space-y-3">
              <InfoRow label="JOB ID" :value="selectedCronJob.id" />
              <InfoRow label="NAME" :value="selectedCronJob.name || '-'" />
              <InfoRow label="SCHEDULE" :value="selectedCronJob.schedule || '-'" />
              <InfoRow label="TIMEZONE" :value="selectedCronJob.timezone || '-'" />
              <InfoRow label="NEXT RUN" :value="formatTimeLabel(selectedCronJob.nextRunAt)" />
              <InfoRow label="LAST RUN" :value="formatTimeLabel(selectedCronJob.lastRunAt)" />
              <InfoRow label="LAST RESULT" :value="selectedCronJob.lastStatus || '-'" />
              <InfoRow label="LAST DURATION" :value="formatDuration(selectedCronJob.lastDurationMs)" />
              <InfoRow label="FAIL STREAK" :value="String(selectedCronJob.consecutiveFailures || 0)" />
              <InfoRow label="WORKSPACE" :value="selectedCronJob.workspace || '-'" />
              <div class="pt-1">
                <div class="text-[11px] text-subtext mb-1">PROMPT / COMMAND</div>
                <div class="text-xs whitespace-pre-wrap break-all rounded-lg border border-white/10 bg-black/20 p-2">
                  {{ selectedCronJob.prompt || '-' }}
                </div>
              </div>
              <div class="pt-1">
                <div class="flex items-center justify-between mb-1">
                  <div class="text-[11px] text-subtext">最近执行</div>
                  <button class="btn text-xs" :disabled="cronRunsLoading" @click="refreshSelectedCronRuns">
                    {{ cronRunsLoading ? '加载中...' : '刷新记录' }}
                  </button>
                </div>
                <div class="max-h-[190px] overflow-auto rounded-lg border border-white/10 bg-black/20 p-2">
                  <div v-if="cronRunsLoading" class="text-xs text-subtext">加载中...</div>
                  <div v-else-if="selectedCronRuns.length === 0" class="text-xs text-subtext">暂无运行记录</div>
                  <div v-for="(r, idx) in selectedCronRuns" :key="`${r.jobId}-${idx}`" class="text-xs py-1 border-b border-white/10 last:border-b-0">
                    <div class="flex items-center justify-between gap-2">
                      <span :class="cronRunStatusClass(r.status)">{{ r.status || 'unknown' }}</span>
                      <span class="text-subtext">{{ formatTimeLabel(r.startedAt || r.finishedAt) }}</span>
                    </div>
                    <div class="text-subtext mt-0.5">
                      duration: {{ formatDuration(r.durationMs) }} | exit: {{ r.exitCode ?? '-' }}
                    </div>
                    <div v-if="r.summary || r.error" class="mt-0.5 whitespace-pre-wrap break-all text-subtext">
                      {{ r.error || r.summary }}
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div class="rounded-xl border border-white/10 bg-black/20 mb-3 overflow-hidden">
          <div class="px-4 py-3 border-b border-white/10 flex items-center justify-between gap-2">
            <div class="text-sm font-medium">全局运行历史</div>
            <input v-model="cronHistorySearch" class="input max-w-[340px]" placeholder="搜索 job/status/summary..." />
          </div>
          <div class="max-h-[280px] overflow-auto">
            <div v-if="visibleCronRuns.length === 0" class="p-4 text-sm text-subtext">暂无历史记录</div>
            <div v-for="(r, idx) in visibleCronRuns" :key="`${r.jobId}-${idx}-${r.startedAt || r.finishedAt || idx}`"
                 class="px-4 py-2 border-b border-white/10 last:border-b-0 text-xs">
              <div class="flex items-center justify-between gap-2">
                <span class="font-medium">{{ r.jobId }}</span>
                <span :class="cronRunStatusClass(r.status)">{{ r.status || 'unknown' }}</span>
              </div>
              <div class="text-subtext mt-0.5">
                {{ formatTimeLabel(r.startedAt || r.finishedAt) }} | duration: {{ formatDuration(r.durationMs) }} | exit: {{ r.exitCode ?? '-' }}
              </div>
              <div v-if="r.summary || r.error" class="text-subtext mt-0.5 whitespace-pre-wrap break-all">{{ r.error || r.summary }}</div>
            </div>
          </div>
        </div>

        <div class="mt-3 p-4 rounded-xl border border-white/10 bg-black/20">
          <div class="text-[11px] text-subtext mb-2">CRON ISSUES</div>
          <div v-if="cronPanel.errors.length === 0" class="text-sm text-green-300">No blocking issues</div>
          <div v-for="(err, i) in cronPanel.errors" :key="i" class="text-sm text-amber-300 break-words py-1">{{ err }}</div>
        </div>
      </div>
    </main>

    <main v-else-if="activeView === 'skills'" class="flex-1 p-6 bg-black/10 overflow-auto">
      <div class="max-w-[980px] mx-auto">
        <div class="flex items-center justify-between mb-4">
          <div>
            <div class="text-xl font-semibold">Skills</div>
            <div class="text-xs text-subtext">已安装/可用技能清单</div>
          </div>
          <button class="btn" :disabled="skillsLoading" @click="refreshSkills">
            {{ skillsLoading ? '刷新中...' : '刷新 Skills' }}
          </button>
        </div>

        <div class="grid grid-cols-3 gap-3 mb-3">
          <div class="p-3 rounded-xl border border-white/10 bg-white/5">
            <div class="text-xs text-subtext">TOTAL</div>
            <div class="text-lg font-semibold">{{ skillsData.skills.length }}</div>
          </div>
          <div class="p-3 rounded-xl border border-white/10 bg-white/5">
            <div class="text-xs text-subtext">ELIGIBLE</div>
            <div class="text-lg font-semibold">{{ skillsData.skills.filter(s => s.eligible).length }}</div>
          </div>
          <div class="p-3 rounded-xl border border-white/10 bg-white/5">
            <div class="text-xs text-subtext">MISSING</div>
            <div class="text-lg font-semibold">{{ skillsData.skills.filter(s => !s.eligible).length }}</div>
          </div>
        </div>

        <div class="p-3 rounded-xl border border-white/10 bg-black/20 mb-3 flex items-center gap-2">
          <input v-model="skillsSearch" class="input max-w-[300px]" placeholder="Search skills" />
          <button class="btn" @click="skillsFilter='all'">All</button>
          <button class="btn" @click="skillsFilter='eligible'">Eligible</button>
          <button class="btn" @click="skillsFilter='missing'">Missing</button>
        </div>

        <div class="p-3 rounded-xl border border-white/10 bg-black/20">
          <div v-if="visibleSkills.length===0" class="text-sm text-subtext">No skills</div>
          <div v-for="(s, i) in visibleSkills" :key="`${s.name}-${i}`" class="py-3 border-b border-white/10 last:border-b-0">
            <div class="flex items-center justify-between gap-3">
              <div class="font-medium">{{ s.name || '-' }}</div>
              <div class="text-xs" :class="s.eligible ? 'text-green-300' : 'text-amber-300'">
                {{ s.eligible ? 'eligible' : 'missing deps' }}
              </div>
            </div>
            <div class="text-xs text-subtext mt-1">{{ s.description || '-' }}</div>
            <div class="text-[11px] text-subtext mt-1">source: {{ s.source || '-' }}</div>
          </div>
        </div>
      </div>
    </main>

    <main v-else-if="activeView === 'gatewayLogs'" class="flex-1 p-6 bg-black/10 overflow-auto">
      <div class="max-w-[980px] mx-auto">
        <div class="flex items-center justify-between mb-4">
          <div>
            <div class="text-xl font-semibold">Logs</div>
            <div class="text-xs text-subtext">Gateway file logs (JSONL)</div>
          </div>
          <div class="flex items-center gap-2">
            <button class="btn" :disabled="gatewayLogsLoading" @click="refreshGatewayLogs">
              {{ gatewayLogsLoading ? '刷新中...' : 'Refresh' }}
            </button>
            <button class="btn" @click="exportVisibleGatewayLogs">Export visible</button>
          </div>
        </div>

        <div class="mb-3 text-xs text-subtext">
          更新于 {{ formatTimeLabel(String(gatewayLogs.fetchedAt || '')) }}
        </div>

        <div class="p-4 rounded-xl border border-white/10 bg-black/20 mb-3">
          <div class="text-sm mb-2">Filter</div>
          <div class="flex items-center gap-2 mb-3">
            <input v-model="gatewayLogSearch" class="input max-w-[320px]" placeholder="Search logs" />
            <label class="text-xs flex items-center gap-1">
              <input type="checkbox" v-model="gatewayLogAutoFollow" />
              Auto-follow
            </label>
          </div>
          <div class="flex items-center gap-2 text-xs">
            <label v-for="lvl in ['trace','debug','info','warn','error','fatal']" :key="lvl" class="px-2 py-1 rounded-lg border border-white/10 flex items-center gap-1">
              <input type="checkbox" v-model="gatewayLogLevelEnabled[lvl]" />
              <span :class="lvl==='warn' ? 'text-amber-300' : (lvl==='error' || lvl==='fatal' ? 'text-red-300' : 'text-subtext')">{{ lvl }}</span>
            </label>
          </div>
        </div>

        <div id="gateway-log-scroll" class="p-0 rounded-xl border border-white/10 bg-black/20 overflow-auto" style="max-height: calc(100vh - 290px)">
          <div v-if="visibleGatewayLogs.length === 0" class="text-sm text-subtext p-4">暂无日志</div>
          <div v-for="line in visibleGatewayLogs" :key="line.id" class="grid grid-cols-[90px_80px_120px_1fr] gap-3 px-3 py-2 border-b border-white/5 text-xs">
            <div class="text-subtext">{{ line.ts }}</div>
            <div>
              <span class="px-2 py-0.5 rounded-md border border-white/10"
                    :class="line.level==='warn' ? 'text-amber-300' : (line.level==='error' || line.level==='fatal' ? 'text-red-300' : 'text-blue-300')">
                {{ line.level }}
              </span>
            </div>
            <div class="text-subtext truncate">{{ line.source }}</div>
            <div class="font-mono whitespace-pre-wrap break-all">{{ line.message }}</div>
          </div>
        </div>
      </div>
    </main>

    <main v-else class="flex-1 p-6 bg-black/10 overflow-auto">
      <div class="max-w-[980px] mx-auto">
        <div class="flex items-center justify-between mb-4">
          <div>
            <div class="text-xl font-semibold">本机状态</div>
            <div class="text-xs text-subtext">检查 OpenClaw CLI、Session 存储和运行时异常</div>
          </div>
          <div class="flex items-center gap-2">
            <button class="btn" @click="refreshLocalStatus">刷新环境</button>
            <button class="btn" :disabled="actionLoading !== ''" @click="handleRestartGateway">
              {{ actionLoading === 'restart' ? '重启中...' : '重启' }}
            </button>
            <button class="btn" :disabled="actionLoading !== ''" @click="handleDoctor">
              {{ actionLoading === 'doctor' ? '执行中...' : '诊断' }}
            </button>
          </div>
        </div>

        <div class="grid grid-cols-2 gap-3">
          <div class="p-4 rounded-xl border border-white/10 bg-white/5 space-y-2">
            <InfoRow label="OPENCLAW CLI" :value="status.openclawFound ? 'FOUND' : 'MISSING'" />
            <InfoRow label="VERSION" :value="status.openclawVersion || '-'" />
            <InfoRow label="PATH" :value="status.openclawPath || '-'" />
            <div class="pt-2 mt-2 border-t border-white/10">
              <div class="text-[11px] text-subtext mb-1">GATEWAY PORT</div>
              <div class="flex items-center gap-2">
                <select v-model="gatewayPortDraft" class="input text-xs" @change="applyGatewayPortSelection">
                  <option value="">自动（{{ status.resolvedGatewayPort ? `当前 ${status.resolvedGatewayPort}` : '默认' }}）</option>
                  <option v-for="p in gatewayPorts" :key="`${p.port}-${p.pid||0}`" :value="String(p.port)">
                    {{ p.port }}{{ p.pid ? ` (pid:${p.pid})` : '' }}
                  </option>
                </select>
                <button class="btn" :disabled="gatewayPortsLoading" @click="refreshGatewayPorts">
                  {{ gatewayPortsLoading ? '刷新中...' : '刷新端口' }}
                </button>
              </div>
            </div>
          </div>
          <div class="p-4 rounded-xl border border-white/10 bg-white/5 space-y-2">
            <InfoRow label="TARGET PORT" :value="displayTargetPort" />
            <InfoRow label="GATEWAY STATE" :value="status.gatewayState || '-'" />
            <InfoRow label="GATEWAY RPC" :value="status.gatewayRpcOk ? 'OK' : 'FAILED'" />
            <InfoRow label="CURRENT CHANNEL" :value="status.currentChannel || '-'" />
            <InfoRow label="SESSIONS FILE" :value="status.sessionsFileExists ? 'READY' : 'NOT FOUND'" />
            <InfoRow label="FILE READABLE" :value="status.sessionsFileReadable ? 'YES' : 'NO'" />
            <InfoRow label="SESSIONS COUNT" :value="String(status.sessionsCount)" />
            <InfoRow label="LATEST ACTIVE" :value="formatTimeLabel(status.latestSessionUpdatedAt)" />
          </div>
        </div>

        <div class="mt-3 grid grid-cols-2 gap-3">
          <div class="p-4 rounded-xl border border-white/10 bg-black/20">
            <div class="text-[11px] text-subtext mb-1">CONFIGURED CHANNELS</div>
            <div v-if="status.configuredChannels.length === 0" class="text-xs text-subtext">No channel summary</div>
            <div v-for="(c, i) in status.configuredChannels" :key="i" class="text-xs py-0.5 break-all">{{ c }}</div>
          </div>
          <div class="p-4 rounded-xl border border-white/10 bg-black/20">
            <div class="text-[11px] text-subtext mb-2">RUNTIME ISSUES</div>
            <div v-if="status.errors.length === 0" class="text-sm text-green-300">No blocking issues</div>
            <div v-for="(err, i) in status.errors" :key="i" class="text-sm text-amber-300 break-words py-1">{{ err }}</div>
          </div>
        </div>

        <div v-if="actionResult" class="mt-3 p-4 rounded-xl border border-white/10 bg-black/20 text-xs space-y-1">
          <div :class="actionResult.ok ? 'text-green-300' : 'text-amber-300'">
            {{ actionResult.ok ? 'Command success' : 'Command failed' }}: {{ actionResult.command }}
          </div>
          <div v-if="actionResult.error" class="text-amber-300 break-words">{{ actionResult.error }}</div>
          <pre v-if="actionResult.stdout" class="whitespace-pre-wrap break-words text-subtext">{{ actionResult.stdout }}</pre>
        </div>

        <div class="mt-3 p-4 rounded-xl border border-white/10 bg-black/20">
          <div class="flex items-center justify-between mb-2">
            <div>
              <div class="text-sm font-semibold">openclaw.json</div>
              <div class="text-[11px] text-subtext break-all">{{ openclawConfig.path || '-' }}</div>
            </div>
            <div class="flex items-center gap-2">
              <button class="btn" :disabled="openclawConfigLoading || openclawConfigSaving" @click="refreshOpenclawConfig">
                {{ openclawConfigLoading ? '读取中...' : '重新加载' }}
              </button>
              <button class="btn" :disabled="openclawConfigLoading || openclawConfigSaving" @click="persistOpenclawConfig">
                {{ openclawConfigSaving ? '保存中...' : '保存修改' }}
              </button>
            </div>
          </div>
          <textarea
            v-model="openclawConfigDraft"
            class="w-full h-[280px] rounded-lg bg-black/30 border border-white/10 p-3 text-xs font-mono outline-none focus:border-purple-500/50"
            spellcheck="false"
          />
          <div v-if="openclawConfigMsg" class="text-xs mt-2" :class="openclawConfigMsg === '保存成功' ? 'text-green-300' : 'text-amber-300'">
            {{ openclawConfigMsg }}
          </div>
        </div>
      </div>
    </main>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import NavItem from './components/NavItem.vue'
import InfoRow from './components/InfoRow.vue'
import {
  deleteCronJob,
  getAgentPanelStatus,
  getCronJobRuns,
  getCronPanelStatus,
  getGatewayLogs,
  getLocalStatus,
  getLocalStatusQuick,
  getOpenclawConfig,
  getSessionHistory,
  listGatewayPorts,
  listModels,
  listSessions,
  listSkills,
  restartGateway,
  runCronJobNow,
  runDoctor,
  saveOpenclawConfig,
  sendMessage,
  setCronJobEnabled,
  type ActionResult,
  type AgentPanelData,
  type CronJobItem,
  type CronPanelData,
  type CronRunItem,
  type GatewayLogs,
  type GatewayPortItem,
  type LocalStatus,
  type ModelOption,
  type OpenclawConfigPayload,
  type SessionItem,
  type SkillItem,
  type SkillsPayload,
} from './api'

const sessions = ref<SessionItem[]>([])
const selected = ref<SessionItem | null>(null)
const sessionQuery = ref('')

type ChatMessage = {
  role: string;
  text: string;
  meta?: string;
  kind?: 'text' | 'toolCall' | 'toolResult';
  toolName?: string;
  toolPayload?: string;
}

const messages = ref<ChatMessage[]>([])
const debugMessages = ref<ChatMessage[]>([])
const logs = ref<{ ts: string; level: 'info' | 'tool' | 'warn'; text: string }[]>([])
const messageView = ref<'chat' | 'debug'>('chat')
const chatOnlyText = ref(false)
const logFilter = ref<'all' | 'info' | 'tool' | 'warn'>('all')
const expandedLogIndex = ref(-1)

const draft = ref('')
const sending = ref(false)
const lastError = ref<string | null>(null)
const polling = ref(false)
const activeView = ref<'sessions' | 'agents' | 'cron' | 'skills' | 'gatewayLogs' | 'local'>('local')
let pollTimer: ReturnType<typeof setInterval> | null = null

const status = ref<LocalStatus>({
  openclawFound: false,
  openclawVersion: '',
  openclawPath: '',
  sessionsDir: '',
  sessionsFileExists: false,
  sessionsFileReadable: false,
  sessionsCount: 0,
  latestSessionUpdatedAt: '',
  errors: [],
  gatewayState: '',
  gatewayRpcOk: false,
  gatewayRpcError: '',
  currentChannel: '',
  configuredChannels: [],
})
const actionResult = ref<ActionResult | null>(null)
const actionLoading = ref<'restart' | 'doctor' | ''>('')
const cronPanel = ref<CronPanelData>({
  scheduler: {},
  jobs: [],
  recentRuns: [],
  errors: [],
  fetchedAt: 0,
})
const cronLoading = ref(false)
const cronSearch = ref('')
const cronStatusFilter = ref<'all' | 'enabled' | 'disabled' | 'failing'>('all')
const cronRiskOnly = ref(false)
const cronActionLoading = ref('')
const selectedCronId = ref('')
const selectedCronRuns = ref<CronRunItem[]>([])
const cronRunsLoading = ref(false)
const cronHistorySearch = ref('')
const agentPanel = ref<AgentPanelData>({
  defaultAgentId: '',
  items: [],
  errors: [],
})
const agentPanelLoading = ref(false)
const agentPanelLoaded = ref(false)
const gatewayLogs = ref<GatewayLogs>({ lines: [], fetchedAt: 0 })
const gatewayLogsLoading = ref(false)
const skillsData = ref<SkillsPayload>({ workspaceDir: '', managedSkillsDir: '', skills: [] })
const skillsLoading = ref(false)
const skillsSearch = ref('')
const skillsFilter = ref<'all' | 'eligible' | 'missing'>('all')
const gatewayLogSearch = ref('')
const gatewayLogAutoFollow = ref(true)
const gatewayLogLevelEnabled = ref<Record<string, boolean>>({
  trace: true,
  debug: true,
  info: true,
  warn: true,
  error: true,
  fatal: true,
})
const openclawConfig = ref<OpenclawConfigPayload>({ path: '', content: '' })
const openclawConfigDraft = ref('')
const openclawConfigLoading = ref(false)
const openclawConfigSaving = ref(false)
const openclawConfigMsg = ref('')
const selectedModelDraft = ref('')
const modelOptions = ref<ModelOption[]>([])
const modelOptionsLoading = ref(false)
const gatewayPorts = ref<GatewayPortItem[]>([])
const gatewayPortsLoading = ref(false)
const gatewayPortDraft = ref('')
const viewBootstrapped = ref<Record<string, boolean>>({
  sessions: false,
  agents: false,
  cron: false,
  skills: false,
  gatewayLogs: false,
  local: true,
})

const scrollRef = ref<HTMLElement | null>(null)
const selectedGatewayPort = computed(() => {
  const n = Number(gatewayPortDraft.value)
  return Number.isFinite(n) && n > 0 ? n : undefined
})
const displayTargetPort = computed(() => {
  if (selectedGatewayPort.value) return String(selectedGatewayPort.value)
  if (status.value.resolvedGatewayPort) return String(status.value.resolvedGatewayPort)
  return '-'
})

const filteredSessions = computed(() => {
  const q = sessionQuery.value.trim().toLowerCase()
  if (!q) return sessions.value
  return sessions.value.filter(s =>
    (s.sessionKey || '').toLowerCase().includes(q) ||
    (s.sessionId || '').toLowerCase().includes(q) ||
    (s.label || '').toLowerCase().includes(q),
  )
})

const dedupedSessions = computed(() => {
  const map = new Map<string, SessionItem>()
  for (const s of filteredSessions.value) {
    const id = (s.sessionId || '').trim()
    const key = id || s.sessionKey
    const prev = map.get(key)
    if (!prev) {
      map.set(key, s)
      continue
    }
    const prevTs = Number(prev.updatedAt || 0)
    const curTs = Number(s.updatedAt || 0)
    if (curTs >= prevTs) map.set(key, s)
  }
  return Array.from(map.values())
})

function isToday(raw?: string) {
  if (!raw) return false
  const n = Number(raw)
  const d = Number.isFinite(n) ? new Date(n) : new Date(raw)
  if (Number.isNaN(d.getTime())) return false
  const now = new Date()
  return d.getFullYear() === now.getFullYear() &&
    d.getMonth() === now.getMonth() &&
    d.getDate() === now.getDate()
}

const groupedSessions = computed(() => {
  const today: SessionItem[] = []
  const past: SessionItem[] = []
  for (const s of dedupedSessions.value) {
    if (isToday(s.updatedAt)) today.push(s)
    else past.push(s)
  }
  return { today, past }
})

const filteredLogs = computed(() => {
  if (logFilter.value === 'all') return logs.value
  return logs.value.filter(l => l.level === logFilter.value)
})

watch(logFilter, () => {
  expandedLogIndex.value = -1
})

const displayedMessages = computed(() => {
  if (messageView.value === 'debug') return debugMessages.value
  if (!chatOnlyText.value) return messages.value
  return messages.value.filter(m => m.kind !== 'toolCall' && m.kind !== 'toolResult')
})

const renderedMessages = computed(() => {
  // Keep DOM size bounded to avoid panel switch jank.
  const max = messageView.value === 'debug' ? 40 : 80
  const list = displayedMessages.value
  if (list.length <= max) return list
  return list.slice(list.length - max)
})

const parsedGatewayLogs = computed(() => {
  return gatewayLogs.value.lines.map((line, idx) => parseGatewayLine(line, idx))
})

const visibleGatewayLogs = computed(() => {
  const q = gatewayLogSearch.value.trim().toLowerCase()
  return parsedGatewayLogs.value.filter((it) => {
    if (!gatewayLogLevelEnabled.value[it.level]) return false
    if (!q) return true
    return (
      it.source.toLowerCase().includes(q) ||
      it.message.toLowerCase().includes(q) ||
      it.raw.toLowerCase().includes(q)
    )
  })
})

const visibleSkills = computed(() => {
  const q = skillsSearch.value.trim().toLowerCase()
  return (skillsData.value.skills || []).filter((s: SkillItem) => {
    if (skillsFilter.value === 'eligible' && !s.eligible) return false
    if (skillsFilter.value === 'missing') {
      const m = s.missing || {}
      const hasMissing = (m.bins?.length || 0) + (m.anyBins?.length || 0) + (m.env?.length || 0) + (m.config?.length || 0) + (m.os?.length || 0) > 0
      if (!hasMissing) return false
    }
    if (!q) return true
    return `${s.name || ''} ${s.description || ''} ${s.source || ''}`.toLowerCase().includes(q)
  })
})

const filteredCronJobs = computed(() => {
  const q = cronSearch.value.trim().toLowerCase()
  return (cronPanel.value.jobs || []).filter((job: CronJobItem) => {
    if (cronStatusFilter.value === 'enabled' && !job.enabled) return false
    if (cronStatusFilter.value === 'disabled' && job.enabled) return false
    if (cronStatusFilter.value === 'failing') {
      const failed = (job.consecutiveFailures || 0) > 0 || (job.lastStatus || '').toLowerCase().includes('fail')
      if (!failed) return false
    }
    if (cronRiskOnly.value && (job.riskFlags || []).length === 0) return false
    if (!q) return true
    return `${job.id} ${job.name || ''} ${job.schedule || ''} ${job.workspace || ''}`.toLowerCase().includes(q)
  })
})

const selectedCronJob = computed(() => {
  if (!selectedCronId.value) return null
  return cronPanel.value.jobs.find((j) => j.id === selectedCronId.value) || null
})

const visibleCronRuns = computed(() => {
  const q = cronHistorySearch.value.trim().toLowerCase()
  const all = cronPanel.value.recentRuns || []
  if (!q) return all
  return all.filter((r: CronRunItem) =>
    `${r.jobId} ${r.status || ''} ${r.summary || ''} ${r.error || ''}`.toLowerCase().includes(q),
  )
})

const cronSummary = computed(() => {
  const jobs = cronPanel.value.jobs || []
  const runs = cronPanel.value.recentRuns || []
  const now = Date.now()
  const in24h = runs.filter((r: CronRunItem) => {
    const raw = r.startedAt || r.finishedAt
    if (!raw) return false
    const n = Number(raw)
    const d = Number.isFinite(n) ? new Date(n) : new Date(raw)
    if (Number.isNaN(d.getTime())) return false
    return now - d.getTime() <= 24 * 60 * 60 * 1000
  })
  const fail24h = in24h.filter((r: CronRunItem) => {
    const t = String(r.status || '').toLowerCase()
    return t.includes('fail') || t.includes('error') || t.includes('timeout') || (r.exitCode ?? 0) !== 0
  }).length
  const ok24h = in24h.filter((r: CronRunItem) => {
    const t = String(r.status || '').toLowerCase()
    return t.includes('ok') || t.includes('success') || t.includes('done')
  }).length
  const total24h = in24h.length
  const successRate24h = total24h ? `${Math.round((ok24h / total24h) * 100)}%` : '-'
  return {
    total: jobs.length,
    enabled: jobs.filter((j) => j.enabled).length,
    disabled: jobs.filter((j) => !j.enabled).length,
    fail24h,
    successRate24h,
  }
})

function sessionKindLabel(s: SessionItem) {
  if (s.kind && s.kind.trim()) return s.kind
  if (s.sessionKey.includes(':cron:')) return 'cron'
  return 'direct'
}

function parseGatewayLine(line: string, idx: number) {
  const raw = line ?? ''
  try {
    const j = JSON.parse(raw)
    const levelRaw = String(j.logLevelName || j.level || j.severity || 'info').toLowerCase()
    const level = ['trace', 'debug', 'info', 'warn', 'error', 'fatal'].includes(levelRaw) ? levelRaw : 'info'
    const source = String(j.name || j.module || j.subsystem || j.fileName || '-')
    const message = String(
      j.message ??
      j.msg ??
      j.text ??
      (j.error ? JSON.stringify(j.error) : raw),
    )
    const t = String(j.time || j.timestamp || '')
    const ts = t ? formatClock(t) : '--:--:--'
    return { id: idx, ts, level, source, message, raw }
  } catch {
    const lower = raw.toLowerCase()
    const level = lower.includes('error') ? 'error' : (lower.includes('warn') ? 'warn' : 'info')
    return { id: idx, ts: '--:--:--', level, source: '-', message: raw, raw }
  }
}

function exportVisibleGatewayLogs() {
  const content = visibleGatewayLogs.value.map(x => x.raw).join('\n')
  const blob = new Blob([content], { type: 'text/plain;charset=utf-8' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `gateway-logs-${Date.now()}.log`
  a.click()
  URL.revokeObjectURL(url)
}

async function refreshLocalStatus() {
  status.value = await getLocalStatus(selectedGatewayPort.value)
}

async function refreshAgentPanel() {
  if (agentPanelLoading.value) return
  agentPanelLoading.value = true
  try {
    agentPanel.value = await getAgentPanelStatus(selectedGatewayPort.value)
    agentPanelLoaded.value = true
  } finally {
    agentPanelLoading.value = false
  }
}

async function refreshCronPanel() {
  if (cronLoading.value) return
  cronLoading.value = true
  try {
    cronPanel.value = await getCronPanelStatus(8, selectedGatewayPort.value)
    if (!selectedCronId.value || !cronPanel.value.jobs.some((j) => j.id === selectedCronId.value)) {
      selectedCronId.value = cronPanel.value.jobs[0]?.id || ''
    }
    if (selectedCronId.value) {
      refreshSelectedCronRuns().catch(() => {})
    } else {
      selectedCronRuns.value = []
    }
  } finally {
    cronLoading.value = false
  }
}

async function refreshSelectedCronRuns() {
  const id = selectedCronId.value
  if (!id) return
  cronRunsLoading.value = true
  try {
    selectedCronRuns.value = await getCronJobRuns(id, 20, selectedGatewayPort.value)
  } finally {
    cronRunsLoading.value = false
  }
}

function selectCronJob(id: string) {
  selectedCronId.value = id
  refreshSelectedCronRuns().catch(() => {})
}

async function runCronJob(jobId: string) {
  try {
    cronActionLoading.value = `run:${jobId}`
    actionResult.value = await runCronJobNow(jobId, selectedGatewayPort.value)
    await refreshCronPanel()
  } finally {
    cronActionLoading.value = ''
  }
}

async function toggleCronJob(jobId: string, enabled: boolean) {
  try {
    cronActionLoading.value = `toggle:${jobId}`
    actionResult.value = await setCronJobEnabled(jobId, enabled, selectedGatewayPort.value)
    await refreshCronPanel()
  } finally {
    cronActionLoading.value = ''
  }
}

async function removeCronJob(jobId: string) {
  const ok = window.confirm(`确认删除 cron 任务 ${jobId} 吗？该操作不可恢复。`)
  if (!ok) return
  try {
    cronActionLoading.value = `delete:${jobId}`
    actionResult.value = await deleteCronJob(jobId, selectedGatewayPort.value)
    await refreshCronPanel()
  } finally {
    cronActionLoading.value = ''
  }
}

async function refreshGatewayLogs() {
  gatewayLogsLoading.value = true
  try {
    gatewayLogs.value = await getGatewayLogs(120, selectedGatewayPort.value)
    if (gatewayLogAutoFollow.value) {
      nextTick(() => {
        const el = document.getElementById('gateway-log-scroll')
        if (el) el.scrollTop = el.scrollHeight
      })
    }
  } finally {
    gatewayLogsLoading.value = false
  }
}

async function refreshSkills() {
  skillsLoading.value = true
  try {
    skillsData.value = await listSkills(selectedGatewayPort.value)
  } finally {
    skillsLoading.value = false
  }
}

async function refreshModelOptions() {
  modelOptionsLoading.value = true
  try {
    modelOptions.value = await listModels(selectedGatewayPort.value)
  } finally {
    modelOptionsLoading.value = false
  }
}

async function refreshGatewayPorts() {
  gatewayPortsLoading.value = true
  try {
    gatewayPorts.value = await listGatewayPorts()
    if (gatewayPortDraft.value && !gatewayPorts.value.some((p) => String(p.port) === gatewayPortDraft.value)) {
      gatewayPortDraft.value = ''
    }
    const first = gatewayPorts.value[0]
    if (!gatewayPortDraft.value && first) {
      gatewayPortDraft.value = String(first.port)
      applyGatewayPortSelection()
    }
  } finally {
    gatewayPortsLoading.value = false
  }
}

function applyGatewayPortSelection() {
  refreshLocalStatus().catch(() => {})
  if (activeView.value === 'agents') refreshAgentPanel().catch(() => {})
  if (activeView.value === 'cron') refreshCronPanel().catch(() => {})
  if (activeView.value === 'skills') refreshSkills().catch(() => {})
  if (activeView.value === 'gatewayLogs') refreshGatewayLogs().catch(() => {})
  if (activeView.value === 'sessions') {
    refreshModelOptions().catch(() => {})
    refreshSessions().catch(() => {})
  }
}

function sessionModelValue(s: SessionItem | null) {
  if (!s) return ''
  const provider = (s.modelProvider || '').trim()
  const model = (s.model || '').trim()
  if (provider && model) return `${provider}/${model}`
  return ''
}

async function refreshOpenclawConfig() {
  openclawConfigLoading.value = true
  openclawConfigMsg.value = ''
  try {
    const data = await getOpenclawConfig()
    openclawConfig.value = data
    openclawConfigDraft.value = data.content
  } catch (e: any) {
    openclawConfigMsg.value = e?.message ?? String(e)
  } finally {
    openclawConfigLoading.value = false
  }
}

async function persistOpenclawConfig() {
  openclawConfigSaving.value = true
  openclawConfigMsg.value = ''
  try {
    const data = await saveOpenclawConfig(openclawConfigDraft.value)
    openclawConfig.value = data
    openclawConfigDraft.value = data.content
    openclawConfigMsg.value = '保存成功'
    refreshLocalStatus().catch(() => {})
  } catch (e: any) {
    openclawConfigMsg.value = e?.message ?? String(e)
  } finally {
    openclawConfigSaving.value = false
  }
}

function switchView(v: 'sessions' | 'agents' | 'cron' | 'skills' | 'gatewayLogs' | 'local') {
  activeView.value = v
  if (v !== 'sessions' && polling.value) {
    polling.value = false
    stopPolling()
  }
  if (viewBootstrapped.value[v]) return
  viewBootstrapped.value[v] = true
  // defer heavy data loading so the panel switch paints first
  setTimeout(() => {
    if (v === 'sessions' && modelOptions.value.length === 0 && !modelOptionsLoading.value) {
      refreshModelOptions().catch(() => {})
    }
    if (v === 'agents' && !agentPanelLoaded.value && !agentPanelLoading.value) {
      refreshAgentPanel().catch(() => {})
    }
    if (v === 'cron' && cronPanel.value.jobs.length === 0 && !cronLoading.value) {
      refreshCronPanel().catch(() => {})
    }
    if (v === 'skills' && skillsData.value.skills.length === 0) {
      refreshSkills().catch(() => {})
    }
    if (v === 'gatewayLogs' && gatewayLogs.value.lines.length === 0) {
      refreshGatewayLogs().catch(() => {})
    }
    if (v === 'local' && !openclawConfig.value.path && !openclawConfigLoading.value) {
      refreshOpenclawConfig().catch(() => {})
    }
  }, 0)
}

async function refreshSessions() {
  lastError.value = null
  sessions.value = await listSessions()
  if (selected.value) {
    const hit = sessions.value.find(s => s.sessionKey === selected.value?.sessionKey)
    if (hit) {
      selected.value = hit
      if (!selectedModelDraft.value.trim()) {
        selectedModelDraft.value = sessionModelValue(hit)
      }
    }
  }
}

function toText(content: any): string {
  if (content == null) return ''
  if (typeof content === 'string') return content
  if (Array.isArray(content)) return content.map(toText).join('\n')
  if (typeof content === 'object') {
    if (content.text) return String(content.text)
    if (content.type === 'text' && content.text) return String(content.text)
    if (content.content) return toText(content.content)
    return JSON.stringify(content, null, 2)
  }
  return String(content)
}

function toChatText(content: any): string {
  if (content == null) return ''
  if (typeof content === 'string') return content
  if (Array.isArray(content)) {
    return content.map(toChatText).filter(Boolean).join('\n')
  }
  if (typeof content === 'object') {
    if (typeof content.text === 'string') return content.text
    if (content.type === 'text' && typeof content.text === 'string') return content.text
    if (content.content != null) return toChatText(content.content)
    return ''
  }
  return ''
}

function formatTimeLabel(raw?: string) {
  if (!raw) return '-'
  const n = Number(raw)
  const d = Number.isFinite(n) ? new Date(n) : new Date(raw)
  if (Number.isNaN(d.getTime())) return String(raw)
  return `${d.toLocaleDateString()} ${d.toLocaleTimeString()}`
}

function formatDuration(ms?: number) {
  if (ms == null || !Number.isFinite(ms)) return '-'
  if (ms < 1000) return `${ms} ms`
  const s = ms / 1000
  if (s < 60) return `${s.toFixed(1)} s`
  const m = Math.floor(s / 60)
  const r = Math.round(s % 60)
  return `${m}m ${r}s`
}

function cronRunStatusClass(status?: string) {
  const t = String(status || '').toLowerCase()
  if (t.includes('ok') || t.includes('success') || t.includes('done')) return 'text-green-300'
  if (t.includes('fail') || t.includes('error') || t.includes('timeout')) return 'text-amber-300'
  return 'text-subtext'
}

function formatClock(raw?: string) {
  if (!raw) return ''
  const d = new Date(raw)
  if (Number.isNaN(d.getTime())) return String(raw).slice(11, 19)
  return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })
}

function nowIso() {
  return new Date().toISOString()
}

function clipText(s: string, n = 96) {
  const t = (s || '').replace(/\s+/g, ' ').trim()
  if (!t) return ''
  return t.length > n ? `${t.slice(0, n - 1)}…` : t
}

function summarizeToolCall(toolName: string, payload: any) {
  const args = payload?.arguments || payload?.message?.arguments || payload?.message?.content?.[0]?.arguments || {}
  const name = toolName || payload?.name || payload?.toolName || 'tool'
  const command = typeof args?.command === 'string' ? args.command : ''
  const query = typeof args?.query === 'string' ? args.query : ''
  const path = typeof args?.path === 'string' ? args.path : ''
  if (command) return `${name}: ${clipText(command)}`
  if (query) return `${name}: query=${clipText(query, 72)}`
  if (path) return `${name}: ${clipText(path, 72)}`
  return `${name}: call`
}

function summarizeToolResult(toolName: string, payload: any) {
  const name = toolName || payload?.toolName || payload?.name || 'tool'
  const details = payload?.details || payload?.message?.details || {}
  const status = String(details?.status || '')
  const exitCode = details?.exitCode
  const firstText = clipText(
    toChatText(payload?.content || payload?.message?.content) ||
    toText(payload?.content || payload?.message?.content),
    80,
  )
  const statusText = status ? `${status}${exitCode !== undefined ? ` exit=${exitCode}` : ''}` : (exitCode !== undefined ? `exit=${exitCode}` : 'result')
  return firstText ? `${name}: ${statusText} · ${firstText}` : `${name}: ${statusText}`
}

function extractGatewayTokenFromConfig(content?: string) {
  if (!content) return ''
  try {
    const j = JSON.parse(content)
    const token = j?.gateway?.auth?.token
    return typeof token === 'string' ? token.trim() : ''
  } catch {
    return ''
  }
}

function extractSseDelta(payload: any): string {
  if (!payload) return ''
  if (typeof payload === 'string') return payload
  const candidates = [
    payload?.message?.content,
    payload?.delta,
    payload?.content,
    payload?.text,
    payload?.choices?.[0]?.delta?.content,
    payload?.choices?.[0]?.message?.content,
  ]
  for (const c of candidates) {
    if (typeof c === 'string' && c) return c
  }
  return ''
}

async function sendViaSse(sessionKey: string, sessionId: string, msg: string) {
  const port = selectedGatewayPort.value ?? status.value.resolvedGatewayPort
  if (!port) throw new Error('gateway port unresolved, cannot open SSE stream')

  let token = extractGatewayTokenFromConfig(openclawConfig.value.content || openclawConfigDraft.value)
  if (!token) {
    try {
      const cfg = await getOpenclawConfig()
      openclawConfig.value = cfg
      if (!openclawConfigDraft.value) openclawConfigDraft.value = cfg.content
      token = extractGatewayTokenFromConfig(cfg.content)
    } catch {
      // ignore token preload errors
    }
  }

  const headers: Record<string, string> = {
    Accept: 'text/event-stream',
    'Content-Type': 'application/json',
  }
  if (token) headers.Authorization = `Bearer ${token}`

  const body = {
    model: 'openclaw',
    stream: true,
    sessionKey,
    sessionId,
    messages: [{ role: 'user', content: msg }],
  }

  const res = await fetch(`http://127.0.0.1:${port}/api/chat`, {
    method: 'POST',
    headers,
    body: JSON.stringify(body),
  })
  if (!res.ok || !res.body) {
    throw new Error(`SSE request failed (${res.status} ${res.statusText || 'error'})`)
  }

  const assistantMsg: ChatMessage = {
    role: 'assistant',
    text: '',
    meta: nowIso(),
    kind: 'text',
  }
  messages.value.push(assistantMsg)

  const reader = res.body.getReader()
  const decoder = new TextDecoder()
  let buffer = ''
  let eventName = ''
  let dataLines: string[] = []
  let gotDelta = false

  const flushEvent = async () => {
    if (dataLines.length === 0) return
    const rawData = dataLines.join('\n')
    dataLines = []
    if (rawData === '[DONE]') return
    let parsed: any = null
    try {
      parsed = JSON.parse(rawData)
    } catch {
      parsed = { raw: rawData }
    }
    const delta = extractSseDelta(parsed)
    if (delta) {
      assistantMsg.text += delta
      gotDelta = true
      await nextTick()
      if (scrollRef.value) scrollRef.value.scrollTop = scrollRef.value.scrollHeight
    }
    debugMessages.value.push({
      role: 'assistant',
      text: JSON.stringify({ type: 'sse', event: eventName || 'message', data: parsed }, null, 2),
      meta: nowIso(),
    })
    eventName = ''
  }

  while (true) {
    const { value, done } = await reader.read()
    if (done) break
    buffer += decoder.decode(value, { stream: true })
    let idx = buffer.indexOf('\n')
    while (idx !== -1) {
      let line = buffer.slice(0, idx)
      buffer = buffer.slice(idx + 1)
      if (line.endsWith('\r')) line = line.slice(0, -1)
      if (!line) {
        await flushEvent()
      } else if (line.startsWith('event:')) {
        eventName = line.slice(6).trim()
      } else if (line.startsWith('data:')) {
        dataLines.push(line.slice(5).trimStart())
      }
      idx = buffer.indexOf('\n')
    }
  }
  await flushEvent()

  if (!gotDelta && !assistantMsg.text.trim()) {
    assistantMsg.text = '(SSE connected but no textual delta returned)'
  }
}

function appendSendError(errText: string) {
  const t = nowIso()
  messages.value.push({
    role: 'assistant',
    text: `发送失败\n${errText}`,
    meta: t,
    kind: 'text',
  })
  debugMessages.value.push({
    role: 'assistant',
    text: JSON.stringify({ type: 'send_error', error: errText }, null, 2),
    meta: t,
  })
  logs.value.push({ ts: formatClock(t), level: 'warn', text: errText.slice(0, 240) })
}

async function quickRefresh() {
  await Promise.all([
    refreshSessions(),
    refreshLocalStatus(),
    refreshAgentPanel(),
    refreshCronPanel(),
    refreshGatewayLogs(),
    refreshHistory(),
  ])
}

async function handleRestartGateway() {
  try {
    actionLoading.value = 'restart'
    actionResult.value = await restartGateway(selectedGatewayPort.value)
    await refreshLocalStatus()
  } finally {
    actionLoading.value = ''
  }
}

async function handleDoctor() {
  try {
    actionLoading.value = 'doctor'
    actionResult.value = await runDoctor(selectedGatewayPort.value)
    await refreshLocalStatus()
  } finally {
    actionLoading.value = ''
  }
}

function onWindowKeydown(e: KeyboardEvent) {
  if (e.metaKey && e.key.toLowerCase() === 'r') {
    e.preventDefault()
    quickRefresh().catch(() => {})
    return
  }
  if (e.metaKey && e.key === 'Enter') {
    e.preventDefault()
    send().catch(() => {})
    return
  }
}

async function refreshHistory() {
  if (!selected.value) return

  lastError.value = null
  const items = await getSessionHistory(selected.value.sessionKey, 120)

  const nextMsgs: ChatMessage[] = []
  const nextDebugMsgs: ChatMessage[] = []
  const nextLogs: { ts: string; level: 'info' | 'tool' | 'warn'; text: string }[] = []

  for (const it of items) {
    const typ = it.type || it.customType
    const tsRaw = it.timestamp || it.ts || it.createdAt || ''
    const ts = tsRaw ? String(tsRaw).slice(11, 19) : '--:--:--'
    const msgNode = it.message ?? null
    const innerType = msgNode?.type || msgNode?.customType
    const isToolCall = typ === 'toolCall' || typ === 'tool_call' || innerType === 'toolCall' || innerType === 'tool_call'
    const isToolResult = typ === 'toolResult' || typ === 'tool_result' || innerType === 'toolResult' || innerType === 'tool_result'
    nextDebugMsgs.push({
      role: 'assistant',
      text: typeof it === 'object' ? JSON.stringify(it, null, 2) : String(it),
      meta: it.timestamp || it.ts,
    })

    if (isToolCall) {
      const toolName = it.name || it.toolName || msgNode?.name || msgNode?.toolName || 'tool'
      nextMsgs.push({
        role: 'assistant',
        text: `${toolName} call`,
        meta: it.timestamp,
        kind: 'toolCall',
        toolName,
        toolPayload: JSON.stringify(it, null, 2),
      })
      nextLogs.push({ ts, level: 'tool', text: summarizeToolCall(String(toolName), it) })
      continue
    }
    if (isToolResult) {
      const toolName = it.name || it.toolName || msgNode?.name || msgNode?.toolName || 'tool'
      nextMsgs.push({
        role: 'assistant',
        text: `${toolName} result`,
        meta: it.timestamp,
        kind: 'toolResult',
        toolName,
        toolPayload: JSON.stringify(it, null, 2),
      })
      nextLogs.push({ ts, level: 'tool', text: summarizeToolResult(String(toolName), it) })
      continue
    }
    if (typ === 'error' || it.error || it.err || it.failed) {
      const errText = toText(it.error ?? it.err ?? it.message ?? it)
      nextLogs.push({ ts, level: 'warn', text: errText.slice(0, 240) })
      continue
    }
    if (typ === 'user') {
      nextMsgs.push({ role: 'user', text: toText(it.text ?? it.content ?? it), meta: it.timestamp, kind: 'text' })
      continue
    }
    if (typ === 'assistant') {
      nextMsgs.push({ role: 'assistant', text: toChatText(it.text ?? it.content ?? it), meta: it.timestamp, kind: 'text' })
      continue
    }
    if (typ === 'message' || it.message) {
      const msg = it.message ?? it
      const role = String(msg.role || it.role || 'assistant')
      const content = msg.content ?? msg.text ?? it.content ?? it.text ?? it
      const metaTs = it.timestamp || msg.timestamp

      // assistant toolCall blocks are nested inside message.content[]
      if (role === 'assistant' && Array.isArray(content)) {
        let handledTool = false
        for (const part of content) {
          if (!part || typeof part !== 'object') continue
          if (part.type === 'toolCall') {
            const toolName = part.name || msg.name || it.name || 'tool'
            nextMsgs.push({
              role: 'assistant',
              text: `${toolName} call`,
              meta: metaTs,
              kind: 'toolCall',
              toolName,
              toolPayload: JSON.stringify(part, null, 2),
            })
            nextLogs.push({ ts, level: 'tool', text: summarizeToolCall(String(toolName), part) })
            handledTool = true
          }
        }
        if (handledTool) continue
      }

      // toolResult comes as role=toolResult in message envelope.
      if (role === 'toolResult') {
        const toolName = msg.toolName || msg.name || it.toolName || it.name || 'tool'
        const text = toChatText(content) || toText(content)
        nextMsgs.push({
          role: 'assistant',
          text: `${toolName} result\n${text}`.trim(),
          meta: metaTs,
          kind: 'toolResult',
          toolName,
          toolPayload: JSON.stringify(msg, null, 2),
        })
        nextLogs.push({ ts, level: 'tool', text: summarizeToolResult(String(toolName), msg) })
        continue
      }

      // message-level execution error from provider/gateway.
      if (msg.errorMessage || msg.stopReason === 'error') {
        const errText = String(msg.errorMessage || msg.stopReason || 'message error')
        nextLogs.push({ ts, level: 'warn', text: errText.slice(0, 240) })
        nextMsgs.push({ role: 'assistant', text: errText, meta: metaTs, kind: 'text' })
        continue
      }

      nextMsgs.push({ role, text: toChatText(content), meta: metaTs, kind: 'text' })
      continue
    }

    // More tolerant event->log mapping so Session Log actually shows signal.
    if (typ && typ !== 'message') {
      const name = it.name || it.toolName || it.event || it.kind || typ
      nextLogs.push({ ts, level: 'info', text: String(name) })
      continue
    }
    if (it.toolName || it.name) {
      nextLogs.push({ ts, level: 'tool', text: `${it.toolName || it.name}` })
      continue
    }

    nextMsgs.push({ role: 'assistant', text: toText(it), meta: it.timestamp, kind: 'text' })
  }

  messages.value = nextMsgs
  debugMessages.value = nextDebugMsgs
  logs.value = nextLogs

  await nextTick()
  if (scrollRef.value) {
    scrollRef.value.scrollTop = scrollRef.value.scrollHeight
  }
}

async function selectSession(s: SessionItem) {
  selected.value = s
  selectedModelDraft.value = sessionModelValue(s)
  await refreshHistory()
}

function applySelectedModel() {
  if (!selected.value) return
  const next = selectedModelDraft.value.trim()
  const [provider, ...modelParts] = next.split('/')
  const model = modelParts.join('/').trim()
  const patched: SessionItem = {
    ...selected.value,
    model: model || selected.value.model,
    modelProvider: provider || selected.value.modelProvider,
  }
  selected.value = patched
  sessions.value = sessions.value.map((it) =>
    it.sessionKey === patched.sessionKey
      ? { ...it, model: patched.model, modelProvider: patched.modelProvider }
      : it,
  )
}

async function send() {
  if (!selected.value) return
  const msg = draft.value.trim()
  if (!msg) return

  const t = nowIso()
  messages.value.push({ role: 'user', text: msg, meta: t, kind: 'text' })
  debugMessages.value.push({
    role: 'user',
    text: JSON.stringify({ type: 'send', text: msg }, null, 2),
    meta: t,
  })
  await nextTick()
  if (scrollRef.value) {
    scrollRef.value.scrollTop = scrollRef.value.scrollHeight
  }

  try {
    sending.value = true
    lastError.value = null
    draft.value = ''
    try {
      await sendViaSse(selected.value.sessionKey, selected.value.sessionId, msg)
      refreshSessions().catch(() => {})
    } catch (sseErr: any) {
      const sseText = sseErr?.message ?? String(sseErr)
      debugMessages.value.push({
        role: 'assistant',
        text: JSON.stringify({ type: 'sse_fallback', error: sseText }, null, 2),
        meta: nowIso(),
      })
      await sendMessage(selected.value.sessionKey, msg, selectedGatewayPort.value)
      await refreshHistory()
    }
  } catch (e: any) {
    const errText = e?.message ?? String(e)
    lastError.value = errText
    appendSendError(errText)
  } finally {
    sending.value = false
  }
}

function startPolling() {
  stopPolling()
  pollTimer = setInterval(() => {
    if (!polling.value || activeView.value !== 'sessions') return

    refreshSessions().catch(() => {})
    refreshHistory().catch(() => {})
  }, 1500)
}

function stopPolling() {
  if (pollTimer) clearInterval(pollTimer)
  pollTimer = null
}

function togglePolling() {
  polling.value = !polling.value
  if (polling.value) startPolling()
  else stopPolling()
}

onMounted(() => {
  window.addEventListener('keydown', onWindowKeydown)
  refreshGatewayPorts().catch(() => {})

  // Fast startup: render local view quickly with lightweight status.
  getLocalStatusQuick(selectedGatewayPort.value)
    .then((v) => { status.value = v })
    .catch((e: any) => { lastError.value = e?.message ?? String(e) })

  // Defer heavier calls so first paint stays responsive.
  requestAnimationFrame(() => {
    setTimeout(() => {
      refreshLocalStatus().catch(() => {})
      if (activeView.value === 'local') refreshOpenclawConfig().catch(() => {})
    }, 1200)
  })
})

onBeforeUnmount(() => {
  window.removeEventListener('keydown', onWindowKeydown)
  stopPolling()
})
</script>

<style scoped>
.app-shell {
  background: linear-gradient(180deg, rgba(21, 23, 38, 0.88) 0%, rgba(9, 10, 17, 0.94) 100%);
}

.content-shell {
  height: 100dvh;
  display: flex;
  width: 100%;
  min-width: 0;
  min-height: 0;
  overflow: hidden;
}

.panel-nav {
  width: clamp(180px, 18vw, 260px);
}

.panel-sessions {
  width: clamp(220px, 24vw, 320px);
}

.panel-runtime {
  width: clamp(240px, 26vw, 340px);
}

@media (max-width: 1120px) {
  .panel-runtime {
    display: none;
  }
}

.btn{ @apply text-xs px-3 py-1.5 rounded-lg border border-white/10 bg-white/5 hover:bg-white/10 transition-colors; }
.btn-primary{ @apply text-xs px-3 py-2 rounded-lg bg-purple-600 hover:bg-purple-500 disabled:opacity-50 disabled:cursor-not-allowed; }
.input{ @apply w-full text-sm px-3 py-2 rounded-lg bg-black/25 border border-white/10 outline-none focus:border-purple-500/50; }

.bubble-user {
  background: linear-gradient(180deg, rgba(124, 58, 237, 0.35) 0%, rgba(124, 58, 237, 0.22) 100%);
  border-color: rgba(167, 139, 250, 0.4);
}

.bubble-assistant {
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.09) 0%, rgba(255, 255, 255, 0.04) 100%);
  border-color: rgba(255, 255, 255, 0.14);
}

.bubble-debug {
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.07) 0%, rgba(255, 255, 255, 0.03) 100%);
  border-color: rgba(255, 255, 255, 0.12);
}
</style>
