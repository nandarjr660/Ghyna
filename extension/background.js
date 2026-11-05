// Posts Chrome's live download state to Ghyna. Chrome keeps the total size of
// a running download in memory, so this is the only way to show a real
// percentage while it is still downloading.

const ENDPOINT = "http://127.0.0.1:47823/downloads";
const HEARTBEAT = "ghyna-download-heartbeat";
const TICK_MS = 1000;

let timer = null;
let active = false;
let inFlight = false;

async function send() {
  if (inFlight) return;
  inFlight = true;
  try {
    const items = await chrome.downloads.search({ state: "in_progress" });
    const downloads = items.map((d) => ({
      id: d.id,
      name: d.filename || "",
      bytes: d.bytesReceived || 0,
      total: d.totalBytes > 0 ? d.totalBytes : 0,
      paused: !!d.paused,
    }));
    setActive(downloads.length > 0);
    try {
      await fetch(ENDPOINT, {
        method: "POST",
        headers: { "Content-Type": "text/plain" },
        body: JSON.stringify({ downloads }),
      });
    } catch {
      // Ghyna is not running; the ticker keeps trying while a download is active.
    }
  } catch {
    // The downloads API hiccupped; the next event or tick tries again.
  } finally {
    inFlight = false;
  }
}

// One post a second while bytes are moving. `onChanged` is not guaranteed to
// fire for every progress update, and relying on it alone lets the notch go
// stale in the middle of a download. Each `search` is also a small keep-alive
// for the service worker.
function setActive(on) {
  if (on === active) return;
  active = on;
  if (on) {
    timer = setInterval(send, TICK_MS);
    chrome.alarms.create(HEARTBEAT, { periodInMinutes: 0.5 });
  } else {
    if (timer) clearInterval(timer);
    timer = null;
    chrome.alarms.clear(HEARTBEAT);
  }
}

// `onChanged` can fire many times a second; one post per 100 ms is plenty.
let queued = false;
function schedule() {
  if (queued) return;
  queued = true;
  setTimeout(() => {
    queued = false;
    send();
  }, 100);
}

chrome.downloads.onCreated.addListener(() => send());
chrome.downloads.onChanged.addListener(schedule);
chrome.downloads.onErased.addListener(() => send());
chrome.alarms.onAlarm.addListener((alarm) => {
  if (alarm.name === HEARTBEAT) send();
});
chrome.runtime.onStartup.addListener(() => send());
chrome.runtime.onInstalled.addListener(() => send());

send();
