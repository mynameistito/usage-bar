import { invoke } from "@tauri-apps/api/core";
import { createUsageGauge } from "./components/UsageGauge";
import { createMcpUsageGauge } from "./components/McpUsageGauge";
import { createSettingsView } from "./components/SettingsView";

const POLL_INTERVAL = 300000; // 5 minutes

let pollingTimer: number | null = null;
let activeTab: "claude" | "zai" | "amp" = "claude";
let claudeLastRefresh: Date | null = null;
let zaiLastRefresh: Date | null = null;
let ampLastRefresh: Date | null = null;
let timestampTimer: number | null = null;

// Cache for API key check to avoid spamming logs
let cachedZaiApiKeyCheck: boolean | null = null;
let zaiApiKeyCacheTime: number = 0;
const ZAI_CACHE_TTL = 5000; // 5 seconds

async function checkZaiApiKey(): Promise<boolean> {
  const now = Date.now();
  if (cachedZaiApiKeyCheck !== null && (now - zaiApiKeyCacheTime) < ZAI_CACHE_TTL) {
    return cachedZaiApiKeyCheck;
  }

  cachedZaiApiKeyCheck = await invoke<boolean>("zai_check_api_key");
  zaiApiKeyCacheTime = now;
  return cachedZaiApiKeyCheck;
}

function invalidateZaiApiKeyCache(): void {
  cachedZaiApiKeyCheck = null;
  zaiApiKeyCacheTime = 0;
}

function updateZaiHeaderState(hasApiKey: boolean): void {
  const zaiView = document.getElementById("zai-view");
  if (zaiView) {
    const header = zaiView.querySelector(".provider-header");
    header?.classList.toggle("empty", !hasApiKey);
  }
}

function createOrUpdateConnectionBadge(
  container: HTMLElement,
  className: string,
  isConnected: boolean
): void {
  // Check if badge exists, create if not
  let badge = container.querySelector(`.${className}`) as HTMLElement;

  if (!badge) {
    badge = document.createElement("span");
    badge.className = className;
    badge.style.cursor = "pointer";

    // Attach click handler directly to the new badge element
    badge.addEventListener("click", (e) => {
      e.preventDefault();
      e.stopPropagation();
      openSettings().catch(console.error);
    });

    container.appendChild(badge);
  }

  // Update badge classes without recreating element
  badge.className = isConnected
    ? `${className} ${className}-connected`
    : `${className} ${className}-disconnected`;

  // Update icon
  let icon = badge.querySelector(`.${className}-icon`) as HTMLElement;
  if (!icon) {
    icon = document.createElement("span");
    icon.className = `${className}-icon`;
    badge.appendChild(icon);
  }

  const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
  svg.setAttribute("width", "12");
  svg.setAttribute("height", "12");
  svg.setAttribute("viewBox", "0 0 24 24");
  svg.setAttribute("fill", "none");
  svg.setAttribute("stroke", "currentColor");
  svg.setAttribute("stroke-linecap", "round");
  svg.setAttribute("stroke-linejoin", "round");

  if (isConnected) {
    svg.setAttribute("stroke-width", "3");
    const polyline = document.createElementNS("http://www.w3.org/2000/svg", "polyline");
    polyline.setAttribute("points", "20 6 9 17 4 12");
    svg.appendChild(polyline);
  } else {
    svg.setAttribute("stroke-width", "2");
    const line1 = document.createElementNS("http://www.w3.org/2000/svg", "line");
    line1.setAttribute("x1", "12");
    line1.setAttribute("y1", "5");
    line1.setAttribute("x2", "12");
    line1.setAttribute("y2", "19");
    const line2 = document.createElementNS("http://www.w3.org/2000/svg", "line");
    line2.setAttribute("x1", "5");
    line2.setAttribute("y1", "12");
    line2.setAttribute("x2", "19");
    line2.setAttribute("y2", "12");
    svg.appendChild(line1);
    svg.appendChild(line2);
  }

  icon.replaceChildren(svg);

  // Update label
  let label = badge.querySelector(`.${className}-label`) as HTMLElement;
  if (!label) {
    label = document.createElement("span");
    label.className = `${className}-label`;
    badge.appendChild(label);
  }
  label.textContent = isConnected ? "Connected" : "Not connected";
}

function updateZaiConnectionBadge(hasApiKey: boolean): void {
  const zaiConnectedStatus = document.getElementById("zai-connected-status");
  if (!zaiConnectedStatus) return;
  createOrUpdateConnectionBadge(zaiConnectedStatus, "zai-header-badge", hasApiKey);
}

function updateAmpConnectionBadge(hasCookie: boolean): void {
  const ampConnectedStatus = document.getElementById("amp-connected-status");
  if (!ampConnectedStatus) return;
  createOrUpdateConnectionBadge(ampConnectedStatus, "amp-header-badge", hasCookie);
}

interface ClaudeUsageData {
  five_hour_utilization: number;
  five_hour_resets_at: string | null;
  seven_day_utilization: number;
  seven_day_resets_at: string | null;
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

interface AmpUsageData {
  quota: number;
  used: number;
  used_percent: number;
  hourly_replenishment: number;
  window_hours: number | null;
  resets_at: number | null;  // epoch millis
}

let settingsOpening = false;

async function openSettings(): Promise<void> {
  const existing = document.getElementById("settings-view");
  if (existing) return;
  if (settingsOpening) return;
  settingsOpening = true;

  try {
    const hasZaiApiKey = await checkZaiApiKey();
    const hasAmpCookie = await invoke<boolean>("amp_check_session_cookie");
    const content = document.getElementById("content");

    const settingsView = createSettingsView({
      checkZaiApiKey,
      validateZaiApiKey: async (apiKey: string) => {
        await invoke("zai_validate_api_key", { apiKey });
      },
      saveZaiApiKey: async (apiKey: string) => {
        await invoke("zai_save_api_key", { apiKey });
      },
      deleteZaiApiKey: async () => {
        await invoke("zai_delete_api_key");
      },
      onZaiKeyChanged: refreshZaiUI,
      checkAmpSessionCookie: async () => invoke<boolean>("amp_check_session_cookie"),
      validateAmpSessionCookie: async (cookie: string) => {
        await invoke("amp_validate_session_cookie", { cookie });
      },
      saveAmpSessionCookie: async (cookie: string) => {
        await invoke("amp_save_session_cookie", { cookie });
      },
      deleteAmpSessionCookie: async () => {
        await invoke("amp_delete_session_cookie");
      },
      onAmpCookieChanged: refreshAmpUI,
      onClose: closeSettings,
    }, hasZaiApiKey, hasAmpCookie);

    const app = document.getElementById("app");

    // Hide content when settings opens
    if (content) {
      content.dataset.originalDisplay = content.style.display || "flex";
      content.style.display = "none";
    }

    app?.appendChild(settingsView);
  } catch (error) {
    console.error("Failed to open settings:", error);
  } finally {
    settingsOpening = false;
  }
}

function closeSettings(): void {
  const settingsView = document.getElementById("settings-view");
  const content = document.getElementById("content");

  if (!settingsView) return;

  // Show content immediately when close starts
  if (content) {
    content.style.display = content.dataset.originalDisplay || "flex";
    delete content.dataset.originalDisplay;
  }

  settingsView.style.animation = "settings-slide-out 0.25s cubic-bezier(0.16, 1, 0.3, 1) forwards";

  const cleanup = () => {
    settingsView.remove();
  };

  const cleanupTimeout = window.setTimeout(cleanup, 250);
  settingsView.addEventListener("animationend", () => {
    clearTimeout(cleanupTimeout);
    cleanup();
  }, { once: true });
}

async function loadContent() {
  const loading = document.getElementById("loading");
  const content = document.getElementById("content");
  const quitButton = document.getElementById("quit-button");

  if (!loading || !content || !quitButton) return;

  content.style.display = "none";

  try {
    await fetchClaudeData();
    await fetchZaiData();

    const hasZaiApiKey = await checkZaiApiKey();
    updateZaiHeaderState(hasZaiApiKey);
    updateZaiConnectionBadge(hasZaiApiKey);

    const hasAmpCookie = await invoke<boolean>("amp_check_session_cookie");
    updateAmpConnectionBadge(hasAmpCookie);

    if (hasAmpCookie) {
      await fetchAmpData();
    }

    loading.style.display = "none";
    content.style.display = "flex";

    setupTabSwitching();

    const savedTab = localStorage.getItem("activeTab");
    if (savedTab === "zai" || savedTab === "amp") {
      switchTab(savedTab as "zai" | "amp");
    }

    startPolling();
    startTimestampUpdater();

    // Show window after content is loaded
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    const window = getCurrentWindow();
    await window.show();
  } catch (error) {
    console.error("Failed to load content:", error);
    loading.innerHTML = "<span>Failed to load</span>";
  }

  quitButton.addEventListener("click", async () => {
    await invoke("quit_app");
  });

  const settingsButton = document.getElementById("settings-button");
  settingsButton?.addEventListener("click", () => openSettings());
}

function setupTabSwitching() {
  const tabClaude = document.getElementById("tab-claude");
  const tabZai = document.getElementById("tab-zai");
  const tabAmp = document.getElementById("tab-amp");

  tabClaude?.addEventListener("click", () => switchTab("claude"));
  tabZai?.addEventListener("click", () => switchTab("zai"));
  tabAmp?.addEventListener("click", () => switchTab("amp"));
}

function switchTab(tab: "claude" | "zai" | "amp") {
  activeTab = tab;
  localStorage.setItem("activeTab", tab);

  const claudeView = document.getElementById("claude-view");
  const zaiView = document.getElementById("zai-view");
  const ampView = document.getElementById("amp-view");
  const tabClaude = document.getElementById("tab-claude");
  const tabZai = document.getElementById("tab-zai");
  const tabAmp = document.getElementById("tab-amp");

  if (claudeView) claudeView.style.display = tab === "claude" ? "block" : "none";
  if (zaiView) zaiView.style.display = tab === "zai" ? "block" : "none";
  if (ampView) ampView.style.display = tab === "amp" ? "block" : "none";

  if (tabClaude) tabClaude.classList.toggle("active", tab === "claude");
  if (tabZai) tabZai.classList.toggle("active", tab === "zai");
  if (tabAmp) tabAmp.classList.toggle("active", tab === "amp");
}

async function fetchClaudeData() {
  try {
    const [usageData, tierData] = await invoke<[ClaudeUsageData, ClaudeTierData]>("claude_get_all");

    const errorContainer = document.getElementById("claude-error");
    const dataContainer = document.getElementById("claude-data");
    const errorMessage = document.getElementById("claude-error-message");

    if (errorContainer) errorContainer.style.display = "none";
    if (dataContainer) {
      dataContainer.style.display = "block";
      dataContainer.innerHTML = "";
    }

    const sessionGauge = createUsageGauge({
      title: "Session",
      utilization: usageData.five_hour_utilization / 100,
      resetsAt: usageData.five_hour_resets_at ?? "",
    });

    if (dataContainer) {
      dataContainer.appendChild(sessionGauge);

      // Only show weekly gauge if there is weekly limit data
      if (usageData.seven_day_resets_at) {
        const weeklyGauge = createUsageGauge({
          title: "Weekly",
          utilization: usageData.seven_day_utilization / 100,
          resetsAt: usageData.seven_day_resets_at,
        });
        dataContainer.appendChild(weeklyGauge);
      }
    }

    const extraUsageLabel = document.getElementById("extra-usage-label");
    const extraUsageValue = document.getElementById("extra-usage-value");
    const extraUsageSection = document.getElementById("extra-usage-section");

    if (extraUsageLabel && extraUsageValue && extraUsageSection) {
      if (usageData.extra_usage_enabled) {
        extraUsageSection.style.display = "block";
        const monthlyLimit = (usageData.extra_usage_monthly_limit ?? 0) / 100;
        const usedCredits = (usageData.extra_usage_used_credits ?? 0) / 100;
        const utilization = usageData.extra_usage_utilization ?? 0;

        extraUsageLabel.textContent = `This month: $${usedCredits.toFixed(2)} / $${monthlyLimit.toFixed(2)}`;
        extraUsageValue.textContent = `${utilization.toFixed(0)}% used`;
        extraUsageValue.style.display = "block";
      } else {
        extraUsageSection.style.display = "none";
        extraUsageLabel.textContent = "Not enabled";
        extraUsageValue.style.display = "none";
      }
    }

    const tierEl = document.getElementById("claude-tier");
    if (tierEl) {
      tierEl.textContent = tierData.plan_name;
      tierEl.title = "";
    }

    claudeLastRefresh = new Date();
    updateTimestamp("claude");
  } catch (error) {
    const errorMsg = String(error);
    const errorContainer = document.getElementById("claude-error");
    const dataContainer = document.getElementById("claude-data");
    const errorMessage = document.getElementById("claude-error-message");

    if (errorContainer && errorMessage) {
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
    }
    if (dataContainer) dataContainer.style.display = "none";

    const tierEl = document.getElementById("claude-tier");
    if (tierEl) {
      tierEl.textContent = "Error";
      tierEl.title = String(error);
    }
  }
}

async function fetchZaiData(forceRefresh = false) {
  const zaiView = document.getElementById("zai-view");
  const errorContainer = document.getElementById("zai-error");
  const dataContainer = document.getElementById("zai-data");
  const errorMessage = document.getElementById("zai-error-message");

  if (!zaiView || !errorContainer || !dataContainer || !errorMessage) return;

  try {
    const command = forceRefresh ? "zai_refresh_all" : "zai_get_all";
    const [usageData, tierData] = await invoke<[ZaiUsageData, ZaiTierData]>(command);

    if (!usageData) return;

    errorContainer.style.display = "none";
    dataContainer.style.display = "block";
    dataContainer.innerHTML = "";

    if (usageData.token_usage) {
      const tokenGauge = createUsageGauge({
        title: "Session",
        utilization: usageData.token_usage.percentage / 100,
        resetsAt: usageData.token_usage.resets_at
          ? new Date(usageData.token_usage.resets_at).toISOString()
          : "",
      });
      dataContainer.appendChild(tokenGauge);
    }

    if (usageData.mcp_usage) {
      const mcpGauge = createMcpUsageGauge({
        title: "MCP Usage (Monthly)",
        percentage: usageData.mcp_usage.percentage,
        used: usageData.mcp_usage.used,
        total: usageData.mcp_usage.total,
      });
      dataContainer.appendChild(mcpGauge);
    }

    const tierEl = document.getElementById("zai-tier");
    if (tierEl) {
      tierEl.textContent = tierData.plan_name;
      tierEl.title = "";
    }

    zaiLastRefresh = new Date();
    updateTimestamp("zai");
  } catch (error) {
    const errorMsg = String(error);
    if (errorMsg.includes("not configured")) {
      dataContainer.style.display = "none";
      errorContainer.style.display = "none";

      const tierEl = document.getElementById("zai-tier");
      if (tierEl) {
        tierEl.textContent = "";
        tierEl.title = "";
      }
    } else {
      errorMessage.textContent = errorMsg;
      errorContainer.style.display = "flex";
      dataContainer.style.display = "none";

      const tierEl = document.getElementById("zai-tier");
      if (tierEl) {
        tierEl.textContent = "Error";
        tierEl.title = String(error);
      }
    }
  }
}

async function refreshZaiUI(): Promise<void> {
  invalidateZaiApiKeyCache();
  const hasZaiApiKey = await checkZaiApiKey();
  updateZaiHeaderState(hasZaiApiKey);
  updateZaiConnectionBadge(hasZaiApiKey);
  await fetchZaiData(true);
}

async function fetchAmpData(forceRefresh = false) {
  const errorContainer = document.getElementById("amp-error");
  const dataContainer = document.getElementById("amp-data");
  const errorMessage = document.getElementById("amp-error-message");

  if (!errorContainer || !dataContainer || !errorMessage) return;

  try {
    const command = forceRefresh ? "amp_refresh_usage" : "amp_get_usage";
    const data = await invoke<AmpUsageData>(command);

    if (!data) return;

    errorContainer.style.display = "none";
    dataContainer.style.display = "block";
    dataContainer.innerHTML = "";

    const usageGauge = createUsageGauge({
      title: "Free Tier Usage",
      utilization: data.used_percent / 100,
      resetsAt: data.resets_at ? new Date(data.resets_at).toISOString() : "",
    });
    dataContainer.appendChild(usageGauge);

    const infoSection = document.createElement("div");
    infoSection.className = "info-section";

    const infoTitle = document.createElement("div");
    infoTitle.className = "info-section-header";
    const titleSpan = document.createElement("span");
    titleSpan.className = "info-section-title";
    titleSpan.textContent = "Balance";
    infoTitle.appendChild(titleSpan);
    infoSection.appendChild(infoTitle);

    const remaining = Math.max(0, data.quota - data.used);
    const total = data.quota;
    const balanceRow = document.createElement("div");
    balanceRow.className = "info-row";
    balanceRow.textContent = `$${remaining.toFixed(2)} / $${total.toFixed(2)} remaining`;
    infoSection.appendChild(balanceRow);

    dataContainer.appendChild(infoSection);

    ampLastRefresh = new Date();
    updateTimestamp("amp");
  } catch (error) {
    const errorMsg = String(error);
    if (errorMsg.includes("not configured")) {
      dataContainer.style.display = "none";
      errorContainer.style.display = "none";
    } else {
      errorMessage.textContent = errorMsg;
      errorContainer.style.display = "flex";
      dataContainer.style.display = "none";

      const tierEl = document.getElementById("amp-tier");
      if (tierEl) {
        tierEl.textContent = "Error";
        tierEl.title = String(error);
      }
    }
  }
}

async function refreshAmpUI(): Promise<void> {
  const hasAmpCookie = await invoke<boolean>("amp_check_session_cookie");
  updateAmpConnectionBadge(hasAmpCookie);
  await fetchAmpData(true);
}

function updateTimestamp(provider: "claude" | "zai" | "amp") {
  const el = document.getElementById(`${provider}-updated`);
  if (!el) return;

  const lastRefresh =
    provider === "claude" ? claudeLastRefresh : provider === "zai" ? zaiLastRefresh : ampLastRefresh;
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
    updateTimestamp("amp");
  }, 30000); // update every 30s
}

async function handleRefresh() {
  const hasAmpCookie = await invoke<boolean>("amp_check_session_cookie");
  const promises: Promise<unknown>[] = [
    fetchClaudeData(),
    fetchZaiData(),
    ...(hasAmpCookie ? [fetchAmpData()] : [])
  ];
  await Promise.allSettled(promises);
}

function startPolling() {
  if (pollingTimer !== null) return;

  pollingTimer = window.setInterval(async () => {
    const hasAmpCookie = await invoke<boolean>("amp_check_session_cookie");
    const promises: Promise<unknown>[] = [fetchClaudeData(), fetchZaiData()];
    if (hasAmpCookie) promises.push(fetchAmpData());
    await Promise.allSettled(promises);
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
