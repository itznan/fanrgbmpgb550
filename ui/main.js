// ==========================================================================
// FanRGB Controller - Vercel UI & Transitions.dev Orchestration
// ==========================================================================

// IPC Bridge
async function invokeCommand(cmd, args = {}) {
  try {
    if (window.__TAURI__ && window.__TAURI__.core) {
      return await window.__TAURI__.core.invoke(cmd, args);
    } else if (window.__TAURI__ && window.__TAURI__.tauri) {
      return await window.__TAURI__.tauri.invoke(cmd, args);
    } else {
      console.log(`[Simulated IPC] ${cmd}`, args);
      return { status: "simulated" };
    }
  } catch (err) {
    showToast(`Error: ${err}`);
    throw err;
  }
}

// --------------------------------------------------------------------------
// Transitions.dev: Toast Open / Close Orchestration
// --------------------------------------------------------------------------
let toastTimer = null;
function showToast(msg) {
  const toast = document.getElementById("toast-container");
  const text = document.getElementById("toast-text");
  text.innerText = msg;

  toast.classList.add("is-open");
  toast.setAttribute("data-open", "true");

  if (toastTimer) clearTimeout(toastTimer);
  toastTimer = setTimeout(() => {
    toast.classList.remove("is-open");
    toast.setAttribute("data-open", "false");
  }, 2600);
}

// --------------------------------------------------------------------------
// Transitions.dev: Sliding Tabs Orchestration
// --------------------------------------------------------------------------
const tabsBar = document.querySelector(".t-tabs");
const tabsPill = tabsBar.querySelector(".t-tabs-pill");
const tabButtons = [...tabsBar.querySelectorAll(".t-tab")];

function movePillTo(tab, animate = true) {
  if (!animate) {
    tabsPill.style.transition = "none";
  }
  tabsPill.style.transform = `translateX(${tab.offsetLeft}px)`;
  tabsPill.style.width = `${tab.offsetWidth}px`;
  if (!animate) {
    tabsPill.offsetHeight; // Force reflow
    tabsPill.style.transition = "";
  }
}

tabButtons.forEach((tab) => {
  tab.addEventListener("click", () => {
    tabButtons.forEach((t) => t.setAttribute("aria-selected", "false"));
    tab.setAttribute("aria-selected", "true");
    movePillTo(tab, true);

    const targetId = tab.getAttribute("data-tab");
    document.querySelectorAll(".panel-section").forEach((p) => p.classList.remove("active"));
    const targetPanel = document.getElementById(targetId);
    if (targetPanel) targetPanel.classList.add("active");
  });
});

// Snap pill on page mount
window.addEventListener("DOMContentLoaded", () => {
  const activeTab = tabsBar.querySelector('.t-tab[aria-selected="true"]') || tabButtons[0];
  if (activeTab) {
    movePillTo(activeTab, false);
  }
  checkHardwareStatus();
});

window.addEventListener("resize", () => {
  const activeTab = tabsBar.querySelector('.t-tab[aria-selected="true"]');
  if (activeTab) movePillTo(activeTab, false);
});

// --------------------------------------------------------------------------
// Transitions.dev: Toggle Switch Orchestration
// --------------------------------------------------------------------------
const gpuToggle = document.getElementById("toggle-gpu-sync");
gpuToggle.addEventListener("click", () => {
  gpuToggle.classList.add("is-init");
  const isOn = gpuToggle.getAttribute("data-on") === "true";
  gpuToggle.setAttribute("data-on", isOn ? "false" : "true");
});

// --------------------------------------------------------------------------
// Color Utilities & Inputs Sync
// --------------------------------------------------------------------------
function hexToRgb(hex) {
  const clean = hex.replace("#", "");
  const num = parseInt(clean, 16);
  return {
    r: (num >> 16) & 255,
    g: (num >> 8) & 255,
    b: num & 255,
  };
}

// Sync color picker with text hex input
const mbColor = document.getElementById("mb-color");
const mbHex = document.getElementById("mb-hex-input");
mbColor.addEventListener("input", (e) => (mbHex.value = e.target.value));
mbHex.addEventListener("input", (e) => {
  if (/^#[0-9A-Fa-f]{6}$/.test(e.target.value)) {
    mbColor.value = e.target.value;
  }
});

const gpuColor = document.getElementById("gpu-color");
const gpuHex = document.getElementById("gpu-hex-input");
gpuColor.addEventListener("input", (e) => (gpuHex.value = e.target.value));
gpuHex.addEventListener("input", (e) => {
  if (/^#[0-9A-Fa-f]{6}$/.test(e.target.value)) {
    gpuColor.value = e.target.value;
  }
});

function setPresetColor(name, hex) {
  mbColor.value = hex;
  mbHex.value = hex;
}

function setGpuPresetColor(name, hex) {
  gpuColor.value = hex;
  gpuHex.value = hex;
}

// --------------------------------------------------------------------------
// Hardware Action Triggers
// --------------------------------------------------------------------------

// 1. Motherboard Update
document.getElementById("btn-apply-mb").addEventListener("click", async () => {
  const hex = mbHex.value;
  const mode = document.getElementById("mb-mode").value;
  const rgb = hexToRgb(hex);

  showToast("Dispatching HID Report 0x52...");
  if (mode === "static") {
    await invokeCommand("set_mb_color", { r: rgb.r, g: rgb.g, b: rgb.b });
  } else {
    await invokeCommand("set_mb_mode", { mode, r: rgb.r, g: rgb.g, b: rgb.b });
  }
  showToast(`Motherboard updated: ${mode.toUpperCase()} (${rgb.r}, ${rgb.g}, ${rgb.b})`);
});

// 2. GPU Update
document.getElementById("btn-apply-gpu").addEventListener("click", async () => {
  const hex = gpuHex.value;
  const mode = document.getElementById("gpu-mode").value;
  const rgb = hexToRgb(hex);

  showToast("Sending NVAPI I2C block...");
  if (mode === "static") {
    await invokeCommand("set_gpu_color", { r: rgb.r, g: rgb.g, b: rgb.b });
  } else {
    await invokeCommand("set_gpu_mode", { mode });
  }
  showToast(`GPU updated: ${mode.toUpperCase()}`);
});

// 3. System Sync
async function syncSystemColor(r, g, b, name) {
  showToast(`Synchronizing system to ${name}...`);
  await invokeCommand("set_sync_color", { r, g, b });
  showToast(`Rig synchronized to ${name}`);
}

async function syncSystemOff() {
  showToast("Powering down all RGB...");
  await invokeCommand("turn_off_all");
  showToast("All system lighting powered down");
}

// 4. Pure Red Bass Visualizer
let visMeterTimer = null;

document.getElementById("btn-start-vis").addEventListener("click", async () => {
  const syncGpu = gpuToggle.getAttribute("data-on") === "true";
  showToast("Starting WASAPI audio loopback visualizer...");
  await invokeCommand("start_visualizer", { syncGpu });

  document.getElementById("btn-start-vis").disabled = true;
  document.getElementById("btn-stop-vis").disabled = false;

  const shimmer = document.getElementById("vis-shimmer-label");
  shimmer.setAttribute("data-text", "Live: 28-90 Hz Bass Capture");
  shimmer.innerText = "Live: 28-90 Hz Bass Capture";

  // Pulse animation for UI meter
  if (!visMeterTimer) {
    visMeterTimer = setInterval(() => {
      const val = Math.floor(Math.random() * 70) + 20;
      document.getElementById("vis-meter-fill").style.width = `${val}%`;
      document.getElementById("vis-pct").innerText = `${val}%`;
    }, 90);
  }
  showToast("Bass Visualizer active");
});

document.getElementById("btn-stop-vis").addEventListener("click", async () => {
  showToast("Halting visualizer...");
  await invokeCommand("stop_visualizer");

  document.getElementById("btn-start-vis").disabled = false;
  document.getElementById("btn-stop-vis").disabled = true;

  const shimmer = document.getElementById("vis-shimmer-label");
  shimmer.setAttribute("data-text", "Standby — Ready");
  shimmer.innerText = "Standby — Ready";

  document.getElementById("vis-meter-fill").style.width = "0%";
  document.getElementById("vis-pct").innerText = "0%";

  if (visMeterTimer) {
    clearInterval(visMeterTimer);
    visMeterTimer = null;
  }
  showToast("Visualizer stopped");
});

// 5. Initial hardware check
async function checkHardwareStatus() {
  const mbInd = document.getElementById("mb-indicator");
  const gpuInd = document.getElementById("gpu-indicator");

  try {
    await invokeCommand("get_mb_status");
    mbInd.className = "status-indicator active";
    mbInd.querySelector(".status-label").innerText = "B550 HID Active";
  } catch (e) {
    mbInd.className = "status-indicator error";
    mbInd.querySelector(".status-label").innerText = "B550 Offline";
  }

  try {
    const gpu = await invokeCommand("get_gpu_status");
    gpuInd.className = "status-indicator active";
    gpuInd.querySelector(".status-label").innerText = `RTX 3060 Ti (${gpu.address})`;
  } catch (e) {
    gpuInd.className = "status-indicator";
    gpuInd.querySelector(".status-label").innerText = "GPU Standby";
  }
}
