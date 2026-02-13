import { invoke } from "@tauri-apps/api/core";
import { createUsageGauge } from "./components/UsageGauge";
import { createMcpUsageGauge } from "./components/McpUsageGauge";
import {
  createZaiSettings,
  checkZaiApiKey,
  createZaiConnectionBadge,
  setZaiCallbacks,
  openZaiModal,
} from "./components/ZaiSettings";

const POLL_INTERVAL = 300000; // 5 minutes

let pollingTimer: number | null = null;
let activeTab: "claude" | "zai" = "claude";
let claudeLastRefresh: Date | null = null;
let zaiLastRefresh: Date | null = null;
let timestampTimer: number | null = null;

function updateZaiHeaderState(hasApiKey: boolean): void {
  const zaiView = document.getElementById("zai-view");
  if (zaiView) {
    const header = zaiView.querySelector(".provider-header");
    header?.classList.toggle("empty", !hasApiKey);
  }
}

interface ClaudeUsageData {
  five_hour_utilization: number;
  five_hour_resets_at: string;
  seven_day_utilization: number;
  seven_day_resets_at: string;
  extra_usage_enabled: boolean;
  extra_usage_monthly_limit: number | null;
  extra_usage_used_credits: number | null;
  extra_usage_utilization: number | null;
}

interface ClaudeTierData {
  plan_name: string;
  rate_limit_tier: string;
}

interface ZaiUsageData {
  token_usage?: {
    percentage: number;
    resets_at?: string;
  };
  mcp_usage?: {
    percentage: number;
    used: number;
    total: number;
  };
  tier_name?: string;
}

interface ZaiTierData {
  plan_name: string;
}

async function loadContent() {
  const loading = document.getElementById("loading");
  const content = document.getElementById("content");
  const quitButton = document.getElementById("quit-button");

  if (!loading || !content || !quitButton) return;

  content.style.display = "none";

  try {
    // Register callbacks with ZaiSettings FIRST, before any Z.ai operations
    setZaiCallbacks({
      checkZaiApiKey: async () => {
        return await invoke<boolean>("check_zai_api_key");
      },
      validateZaiApiKey: async (apiKey: string) => {
        await invoke("validate_zai_api_key", { apiKey });
      },
      saveZaiApiKey: async (apiKey: string) => {
        await invoke("save_zai_api_key", { apiKey });
      },
      deleteZaiApiKey: async () => {
        await invoke("delete_zai_api_key");
      },
      refreshZaiUI: refreshZaiUI,
    });

    // Fire-and-forget tier updates (don't await, don't block loading)
    fetchClaudeTier().catch((err) =>
      console.error("Claude tier fetch failed:", err),
    );
    fetchZaiTier().catch((err) =>
      console.error("Z.ai tier fetch failed:", err),
    );

    await fetchClaudeUsage();
    // fetchZaiUsage handles "not configured" gracefully - it won't throw
    await fetchZaiUsage();

    const hasZaiApiKey = await checkZaiApiKey();
    updateZaiHeaderState(hasZaiApiKey);
    const zaiConnectedStatus = document.getElementById("zai-connected-status");
    if (zaiConnectedStatus) {
      zaiConnectedStatus.appendChild(createZaiConnectionBadge(hasZaiApiKey));
    }

    const zaiSettingsEl = document.getElementById("zai-settings");
    if (zaiSettingsEl) {
      const settingsElement = await createZaiSettings();
      zaiSettingsEl.replaceWith(settingsElement);
    }

    loading.style.display = "none";
    content.style.display = "flex";

    setupTabSwitching();

    const savedTab = localStorage.getItem("activeTab");
    if (savedTab === "zai") {
      switchTab("zai");
    }
    }

    startPolling();
    startTimestampUpdater();
  } catch (error) {
    console.error("Failed to load content:", error);
    loading.innerHTML = "<span>Failed to load</span>";
  }

  quitButton.addEventListener("click", async () => {
    await invoke("quit_app");
  });

  // Setup Settings button to open Z.ai modal when on Z.ai tab
  const settingsButton = document.getElementById("settings-button");
  settingsButton?.addEventListener("click", async () => {
    // Only open Z.ai modal if currently on Z.ai tab
    if (activeTab === "zai") {
      const hasZaiApiKey = await checkZaiApiKey();
      openZaiModal(hasZaiApiKey);
    }
  });
}

function setupTabSwitching() {
  const tabClaude = document.getElementById("tab-claude");
  const tabZai = document.getElementById("tab-zai");

  tabClaude?.addEventListener("click", () => switchTab("claude"));
  tabZai?.addEventListener("click", () => switchTab("zai"));
}

function switchTab(tab: "claude" | "zai") {
  activeTab = tab;
  localStorage.setItem("activeTab", tab);

  const claudeView = document.getElementById("claude-view");
  const zaiView = document.getElementById("zai-view");
  const tabClaude = document.getElementById("tab-claude");
  const tabZai = document.getElementById("tab-zai");

  if (claudeView && zaiView && tabClaude && tabZai) {
    claudeView.style.display = tab === "claude" ? "block" : "none";
    zaiView.style.display = tab === "zai" ? "block" : "none";

    tabClaude.classList.toggle("active", tab === "claude");
    tabZai.classList.toggle("active", tab === "zai");
  }
}

async function fetchClaudeTier() {
  try {
    const data = await invoke<ClaudeTierData>("get_claude_tier");
    const tierEl = document.getElementById("claude-tier");
    if (tierEl) {
      tierEl.textContent = data.plan_name;
      tierEl.title = ""; // Clear any error tooltip
    }
  } catch (error) {
    console.error("Failed to fetch Claude tier:", error);
    const tierEl = document.getElementById("claude-tier");
    if (tierEl) {
      tierEl.textContent = "Error";
      tierEl.title = String(error);
    }
  }
}

async function fetchZaiTier() {
  try {
    const data = await invoke<ZaiTierData>("get_zai_tier");
    const tierEl = document.getElementById("zai-tier");
    if (tierEl) {
      tierEl.textContent = data.plan_name;
      tierEl.title = ""; // Clear any error tooltip
    }
  } catch (error) {
    console.error("Failed to fetch Z.ai tier:", error);
    const tierEl = document.getElementById("zai-tier");
    if (tierEl) {
      tierEl.textContent = "Error";
      tierEl.title = String(error);
    }
  }
}

async function refreshZaiUI(): Promise<void> {
  const hasZaiApiKey = await checkZaiApiKey();
  updateZaiHeaderState(hasZaiApiKey);

  // Update connection badge
  const zaiConnectedStatus = document.getElementById("zai-connected-status");
  if (zaiConnectedStatus) {
    zaiConnectedStatus.innerHTML = "";
    zaiConnectedStatus.appendChild(createZaiConnectionBadge(hasZaiApiKey));
  }

  // Update settings
  const zaiSettingsEl = document.getElementById("zai-settings");
  if (zaiSettingsEl) {
    const settingsElement = await createZaiSettings();
    zaiSettingsEl.replaceWith(settingsElement);
  }

  // Fetch usage data with force refresh to bypass cache
  await fetchZaiUsage(true);
  await fetchZaiTier();
}

async function fetchClaudeUsage() {
  const errorContainer = document.getElementById("claude-error");
  const dataContainer = document.getElementById("claude-data");
  const errorMessage = document.getElementById("claude-error-message");

  if (!errorContainer || !dataContainer || !errorMessage) return;

  try {
    const data = await invoke<ClaudeUsageData>("get_claude_usage");

    errorContainer.style.display = "none";
    dataContainer.style.display = "block";
    dataContainer.innerHTML = "";

    const sessionGauge = createUsageGauge({
      title: "Session",
      utilization: data.five_hour_utilization / 100, // Convert percentage to 0-1 ratio
      resetsAt: data.five_hour_resets_at,
    });

    const weeklyGauge = createUsageGauge({
      title: "Weekly",
      utilization: data.seven_day_utilization / 100, // Convert percentage to 0-1 ratio
      resetsAt: data.seven_day_resets_at,
    });

    dataContainer.appendChild(sessionGauge);
    dataContainer.appendChild(weeklyGauge);

    // Update extra usage section
    const extraUsageLabel = document.getElementById("extra-usage-label");
    const extraUsageValue = document.getElementById("extra-usage-value");
    const extraUsageSection = document.getElementById("extra-usage-section");

    if (extraUsageLabel && extraUsageValue && extraUsageSection) {
      if (data.extra_usage_enabled) {
        extraUsageSection.style.display = "block";
        const monthlyLimit = data.extra_usage_monthly_limit ?? 0;
        const usedCredits = data.extra_usage_used_credits ?? 0;
        const utilization = data.extra_usage_utilization ?? 0;

        extraUsageLabel.textContent = `This month: $${usedCredits.toFixed(2)} / $${monthlyLimit.toFixed(2)}`;
        extraUsageValue.textContent = `${(utilization * 100).toFixed(0)}% used`;
      } else {
        extraUsageSection.style.display = "none";
      }
    }

    claudeLastRefresh = new Date();
    updateTimestamp("claude");
  } catch (error) {
    const errorMsg = String(error);

    if (
      errorMsg.includes("Credential not found") ||
      errorMsg.includes("not found")
    ) {
      errorMessage.textContent =
        "Claude credentials not found. Please sign in to Claude Code first.";
    } else if (errorMsg.includes("Access denied")) {
      errorMessage.textContent = "Access denied -- check your permissions";
    } else if (errorMsg.includes("Rate limited")) {
      errorMessage.textContent = "Rate limited -- please wait and try again";
    } else {
      errorMessage.textContent = errorMsg;
    }

    errorContainer.style.display = "flex";
    dataContainer.style.display = "none";
  }
}

async function fetchZaiUsage(forceRefresh = false): Promise<void> {
  const zaiView = document.getElementById("zai-view");
  const errorContainer = document.getElementById("zai-error");
  const dataContainer = document.getElementById("zai-data");
  const errorMessage = document.getElementById("zai-error-message");

  if (!zaiView || !errorContainer || !dataContainer || !errorMessage) return;

  try {
    const command = forceRefresh ? "refresh_zai_usage" : "get_zai_usage";
    const data = await invoke<ZaiUsageData>(command);

    if (!data) return;

    errorContainer.style.display = "none";
    dataContainer.style.display = "block";
    dataContainer.innerHTML = "";

    if (data.token_usage) {
      const tokenGauge = createUsageGauge({
        title: "Session",
        utilization: data.token_usage.percentage / 100, // Convert percentage to 0-1 ratio
        resetsAt: data.token_usage.resets_at
          ? new Date(data.token_usage.resets_at).toISOString()
          : "",
      });
      dataContainer.appendChild(tokenGauge);
    }

    if (data.mcp_usage) {
      const mcpGauge = createMcpUsageGauge({
        title: "MCP Usage (Monthly)",
        percentage: data.mcp_usage.percentage,
        used: data.mcp_usage.used,
        total: data.mcp_usage.total,
      });
      dataContainer.appendChild(mcpGauge);
    }

    zaiLastRefresh = new Date();
    updateTimestamp("zai");
  } catch (error) {
    const errorMsg = String(error);
    if (errorMsg.includes("not configured")) {
      // Hide data, show nothing — settings will handle it
      dataContainer.style.display = "none";
      errorContainer.style.display = "none";
    } else {
      errorMessage.textContent = errorMsg;
      errorContainer.style.display = "flex";
      dataContainer.style.display = "none";
    }
  }
}

function updateTimestamp(provider: "claude" | "zai") {
  const el = document.getElementById(`${provider}-updated`);
  if (!el) return;

  const lastRefresh =
    provider === "claude" ? claudeLastRefresh : zaiLastRefresh;
  if (!lastRefresh) {
    el.textContent = "Updated just now";
    return;
  }

  const now = new Date();
  const diffMs = now.getTime() - lastRefresh.getTime();
  const diffMin = Math.floor(diffMs / 60000);

  if (diffMin < 1) {
    el.textContent = "Updated just now";
  } else if (diffMin < 60) {
    el.textContent = `Updated ${diffMin}m ago`;
  } else {
    const diffHours = Math.floor(diffMin / 60);
    el.textContent = `Updated ${diffHours}h ago`;
  }
}

function startTimestampUpdater() {
  if (timestampTimer !== null) return;
  timestampTimer = window.setInterval(() => {
    updateTimestamp("claude");
    updateTimestamp("zai");
  }, 30000); // update every 30s
}

async function handleRefresh() {
  try {
    await fetchClaudeUsage();
    await fetchZaiUsage();
  } catch (error) {
    console.error("Failed to refresh:", error);
  }
}

function startPolling() {
  if (pollingTimer !== null) return;

  pollingTimer = window.setInterval(async () => {
    try {
      fetchClaudeTier(); // fire-and-forget
      fetchZaiTier(); // fire-and-forget
      await fetchClaudeUsage();
      await fetchZaiUsage();
    } catch (error) {
      console.error("Polling error:", error);
    }
  }, POLL_INTERVAL);
}

function stopPolling() {
  if (pollingTimer !== null) {
    clearInterval(pollingTimer);
    pollingTimer = null;
  }
  if (timestampTimer !== null) {
    clearInterval(timestampTimer);
    timestampTimer = null;
  }
}

document.addEventListener("DOMContentLoaded", loadContent);

window.addEventListener("beforeunload", stopPolling);
