// Mochi speaks Uzbek, in a little robot's voice. The words come from the
// offline Uzbek voice next to coucou.exe (src-tauri/src/uzvoice.rs, `uz_say`);
// the robot is made here: a touch higher and quicker, a ring modulator for the
// metal, and a very short echo for the tin can.
//
// Only in Uzbek (the voice reads nothing else), only while sounds are on and
// Settings → General → «Mochi gapiradi» is on. One sentence at a time: a new
// one waits for the current one, and only the latest waiting one is kept.

import { invoke } from "@tauri-apps/api/core";
import { IS_TAURI } from "./bridge";
import { Sound } from "./sound";
import { language } from "../i18n/i18n";

const PREF_KEY = "coucou.uzVoice";

/** Settings → General → «Mochi gapiradi»; on unless turned off. */
export function uzVoiceOn(): boolean {
  try {
    return localStorage.getItem(PREF_KEY) !== "off";
  } catch {
    return true;
  }
}

export function setUzVoiceOn(on: boolean): void {
  try {
    localStorage.setItem(PREF_KEY, on ? "on" : "off");
  } catch {
    /* private storage: the default stays */
  }
}

let ctx: AudioContext | null = null;
let busy = false;
let waiting: string | null = null;
let lastText = "";
let lastAt = 0;

function audio(): AudioContext | null {
  if (ctx) return ctx;
  const Ctor = window.AudioContext ?? (window as unknown as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
  if (!Ctor) return null;
  ctx = new Ctor();
  return ctx;
}

/** The robot: buffer → (dry + ring-modulated) → tin-can echo → out. Resolves when it ends. */
function playRobot(ac: AudioContext, buffer: AudioBuffer, volume: number): Promise<void> {
  return new Promise((resolve) => {
    const src = ac.createBufferSource();
    src.buffer = buffer;
    // Higher and a little quicker: small and cheerful.
    src.playbackRate.value = 1.12;

    const dry = ac.createGain();
    dry.gain.value = 0.55;

    // Ring modulator: the voice times a 55 Hz sine.
    const ring = ac.createGain();
    ring.gain.value = 0;
    const carrier = ac.createOscillator();
    carrier.type = "sine";
    carrier.frequency.value = 55;
    const depth = ac.createGain();
    depth.gain.value = 0.75;
    carrier.connect(depth);
    depth.connect(ring.gain);

    // Tin can: a 7 ms echo fed back into itself.
    const mix = ac.createGain();
    const echo = ac.createDelay(0.05);
    echo.delayTime.value = 0.007;
    const feedback = ac.createGain();
    feedback.gain.value = 0.4;
    mix.connect(echo);
    echo.connect(feedback);
    feedback.connect(echo);

    const lowCut = ac.createBiquadFilter();
    lowCut.type = "highpass";
    lowCut.frequency.value = 140;
    const presence = ac.createBiquadFilter();
    presence.type = "peaking";
    presence.frequency.value = 2400;
    presence.gain.value = 4;
    const squash = ac.createDynamicsCompressor();
    squash.threshold.value = -20;
    squash.ratio.value = 4;
    const out = ac.createGain();
    out.gain.value = volume;

    src.connect(dry);
    src.connect(ring);
    dry.connect(mix);
    ring.connect(mix);
    mix.connect(lowCut);
    echo.connect(lowCut);
    lowCut.connect(presence);
    presence.connect(squash);
    squash.connect(out);
    out.connect(ac.destination);

    const done = () => {
      try {
        carrier.stop();
      } catch {
        /* already stopped */
      }
      // Let the echo ring out before letting go of the graph.
      window.setTimeout(() => {
        out.disconnect();
        resolve();
      }, 120);
    };
    src.onended = done;
    carrier.start();
    src.start();
  });
}

async function speakNow(text: string): Promise<void> {
  const ac = audio();
  if (!ac) return;
  let bytes: ArrayBuffer | null = null;
  try {
    bytes = await invoke<ArrayBuffer>("uz_say", { text });
  } catch (err) {
    console.warn("[coucou] uz_say failed", err);
    return;
  }
  if (!bytes || bytes.byteLength <= 44) return;
  if (ac.state === "suspended") await ac.resume().catch(() => {});
  let buffer: AudioBuffer;
  try {
    buffer = await ac.decodeAudioData(bytes);
  } catch {
    return;
  }
  // The island's sounds are at 0–0.2; a voice needs more to be heard over them.
  const volume = Math.min(1, 0.35 + Sound.volume * 3);
  await playRobot(ac, buffer, volume);
}

async function drain(first: string): Promise<void> {
  busy = true;
  try {
    let next: string | null = first;
    while (next) {
      await speakNow(next);
      next = waiting;
      waiting = null;
    }
  } finally {
    busy = false;
    void ctx?.suspend().catch(() => {});
  }
}

/** Whether Mochi would speak now. */
export function canSpeak(): boolean {
  return IS_TAURI && uzVoiceOn() && Sound.enabled && language() === "uz";
}

/**
 * Mochi says `text` (Uzbek). Said again within 4 s, the same sentence is
 * dropped: two hooks for one event must not make him stutter.
 */
export function speak(text: string): void {
  const clean = text.trim();
  if (!clean || !canSpeak()) return;
  const now = Date.now();
  if (clean === lastText && now - lastAt < 4000) return;
  lastText = clean;
  lastAt = now;
  if (busy) {
    waiting = clean;
    return;
  }
  void drain(clean);
}

/** Mochi says `text` and the promise ends with it (« OK Coucou »'s answers). */
export async function speakAndWait(text: string): Promise<boolean> {
  if (!text.trim() || !IS_TAURI) return false;
  const ok = await invoke<boolean>("uz_voice_ready").catch(() => false);
  if (!ok) return false;
  while (busy) await new Promise((r) => setTimeout(r, 100));
  busy = true;
  try {
    await speakNow(text);
    return true;
  } finally {
    busy = false;
  }
}
