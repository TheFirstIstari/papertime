<script lang="ts">
  interface PatternService {
    id: string;
    operator: string;
    headcode: string;
    dep: number | null;
    arr: number | null;
    days: string[];
  }
  
  interface PatternBranch {
    next_stop: string | null;
    next_stop_name: string;
    destination: string;
    destination_tiploc: string;
    frequency: number;
    operators: string[];
    operator_color: string;
    services: PatternService[];
  }
  
  interface StationPattern {
    station: string;
    station_name: string;
    n_services: number;
    branches: PatternBranch[];
  }

  let { patternData = null, loading = true, error = '' }: {
    patternData: StationPattern | null;
    loading: boolean;
    error: string;
  } = $props();

  const OP_COLORS: Record<string, string> = {
    'CC': '#009E73', 'XC': '#009E73', 'SE': '#009E73', 'LE': '#009E73',
    'EM': '#CC79A7', 'GR': '#CC79A7', 'AW': '#CC79A7',
    'LO': '#E86A10', 'ME': '#E86A10',
    'VT': '#E32636', 'HX': '#E32636', 'HT': '#E32636',
    'GW': '#56B4E9', 'SR': '#56B4E9',
    'TP': '#D55E00', 'TL': '#D55E00', 'LM': '#D55E00',
    'NT': '#0072B2', 'SW': '#0072B2', 'CH': '#0072B2',
    'SN': '#F0E442', 'GN': '#F0E442',
    'GC': '#882255', 'GX': '#56B4E9', 'LF': '#E86A10',
    'XR': '#D55E00', 'CS': '#E32636',
  };

  function formatTime(val: number | string | null): string {
    if (val === null || val === undefined) return '---';
    let minutes: number;
    if (typeof val === 'string') {
      const parts = val.split(':');
      if (parts.length < 2) return '---';
      minutes = parseInt(parts[0], 10) * 60 + parseInt(parts[1], 10);
    } else {
      minutes = val;
    }
    if (isNaN(minutes)) return '---';
    const h = Math.floor(minutes / 60) % 24;
    const m = minutes % 60;
    return `${h.toString().padStart(2, '0')}:${m.toString().padStart(2, '0')}`;
  }

  let expandedBranch: string | null = null;

  function toggleBranch(branchKey: string) {
    expandedBranch = expandedBranch === branchKey ? null : branchKey;
  }
</script>

<div class="space-y-4">
  {#if loading}
    <div class="text-center py-8 text-slate-400">Loading pattern diagram...</div>
  {:else if error}
    <div class="text-center py-8 text-red-400">{error}</div>
  {:else if patternData}
    <div class="mb-4">
      <h2 class="text-xl font-bold">{patternData.station_name}</h2>
      <p class="text-slate-400 text-sm">{patternData.n_services} services, {patternData.branches.length} branches</p>
    </div>

    <!-- Branch overview -->
    <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-2 mb-6">
      {#each patternData.branches as branch}
        <div class="bg-slate-800 rounded-lg p-3 border border-slate-700">
          <div class="flex items-center gap-2 mb-1">
            <div class="w-3 h-3 rounded-full" style="background: {branch.operator_color}"></div>
            <span class="text-sm font-medium truncate">{branch.next_stop_name || 'Terminating'}</span>
          </div>
          <div class="text-xs text-slate-400">→ {branch.destination}</div>
          <div class="text-lg font-bold mt-1">{branch.frequency}</div>
          <div class="text-xs text-slate-500">services</div>
        </div>
      {/each}
    </div>

    <!-- Detailed branch view -->
    <div class="space-y-2">
      {#each patternData.branches as branch, idx}
        {@const branchKey = `${branch.next_stop}-${idx}`}
        <button
          class="w-full text-left bg-slate-800 border border-slate-700 rounded-lg p-4 hover:border-blue-500 transition-colors"
          onclick={() => toggleBranch(branchKey)}
        >
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-3">
              <div class="w-4 h-4 rounded-full" style="background: {branch.operator_color}"></div>
              <div>
                <div class="font-medium">
                  {branch.next_stop_name || 'Terminating services'}
                  <span class="text-slate-400"> → {branch.destination}</span>
                </div>
                <div class="text-xs text-slate-500">
                  {branch.operators.join(', ')} · {branch.frequency} services
                </div>
              </div>
            </div>
            <div class="text-slate-400">
              {expandedBranch === branchKey ? '▲' : '▼'}
            </div>
          </div>
          
          {#if expandedBranch === branchKey}
            <div class="mt-4 border-t border-slate-700 pt-4">
              <div class="grid grid-cols-4 gap-2 text-xs text-slate-500 mb-2">
                <div>Headcode</div>
                <div>Operator</div>
                <div>Departs</div>
                <div>Days</div>
              </div>
              {#each branch.services.slice(0, 20) as svc}
                <div class="grid grid-cols-4 gap-2 text-sm py-1 border-t border-slate-700/50">
                  <div class="font-mono">{svc.headcode || '—'}</div>
                  <div>{svc.operator}</div>
                  <div>{formatTime(svc.dep)}</div>
                  <div class="text-slate-400">{svc.days.join(', ')}</div>
                </div>
              {/each}
              {#if branch.services.length > 20}
                <div class="text-sm text-slate-400 mt-2">
                  ... and {branch.services.length - 20} more services
                </div>
              {/if}
            </div>
          {/if}
        </button>
      {/each}
    </div>
  {/if}
</div>
