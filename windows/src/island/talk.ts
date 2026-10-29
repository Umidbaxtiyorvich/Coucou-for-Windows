// What Mochi says out loud, in Uzbek (src/core/uzvoice.ts speaks it). A few
// ways to say each thing, picked at random, so he does not sound like a
// recording.

import { State } from "../core/state";
import { speak } from "../core/uzvoice";

const pick = (lines: readonly string[]) => lines[Math.floor(Math.random() * lines.length)] ?? lines[0] ?? "";

/** The pill's name ("Claude Code", "Codex"…), or a plain "agent". */
function who(agentId: string): string {
  const name = State.tasks.find((t) => t.id === agentId)?.name?.trim();
  return name || "Agent dastur";
}

function partOfDay(hour = new Date().getHours()): string {
  if (hour >= 5 && hour < 11) return "Xayrli tong!";
  if (hour >= 11 && hour < 17) return "Xayrli kun!";
  if (hour >= 17 && hour < 23) return "Xayrli kech!";
  return "Hali ham ishlayapsizmi?";
}

let lastGreeting = 0;
/** Opening the island greets again after this long without a greeting. */
const GREET_AGAIN_MS = 20 * 60 * 1000;

export const Talk = {
  /** Coucou comes up at the top of the screen. */
  launch() {
    lastGreeting = Date.now();
    speak(`Salom! ${partOfDay()} Men Mochi. Sizga qanday yordam bera olaman?`);
  },

  /** The island was opened: hello again, when he has not said it for a while. */
  opened() {
    if (Date.now() - lastGreeting < GREET_AGAIN_MS) return;
    lastGreeting = Date.now();
    speak(pick(["Salom! Qanday yordam beray?", "Salom! Men shu yerdaman. Nima qilamiz?", "Salom! Sizga qanday yordam bera olaman?"]));
  },

  approval(agentId: string) {
    const name = who(agentId);
    speak(pick([`${name} ruxsat so‘ramoqda.`, `${name} sizning ruxsatingizni kutmoqda.`, `Diqqat! ${name} ruxsat so‘rayapti.`]));
  },

  question(agentId: string) {
    const name = who(agentId);
    speak(pick([`${name} sizga savol bermoqda.`, `${name} sizdan javob kutmoqda.`, `${name} bir narsani so‘ramoqchi.`]));
  },

  finished(agentId: string) {
    const name = who(agentId);
    speak(pick([`${name} ishni tugatdi.`, `Tayyor! ${name} ishini yakunladi.`, `${name} vazifani bajarib bo‘ldi.`]));
  },

  failed(agentId: string) {
    const name = who(agentId);
    speak(pick([`Voy! ${name} xatoga uchradi.`, `${name} ishida xatolik yuz berdi.`]));
  },

  rateLimited(agentId: string) {
    speak(`${who(agentId)} so‘rovlar chegarasiga yetdi. Biroz kutish kerak.`);
  },

  started(agentId: string) {
    speak(pick([`${who(agentId)} ishni boshladi.`, `${who(agentId)} ishga kirishdi.`]));
  },

  chatReply() {
    speak(pick(["Javob tayyor!", "Mana, javobim tayyor."]));
  },

  chatFailed() {
    speak("Kechirasiz, javob bera olmadim.");
  },

  integration(name: string, success: boolean) {
    speak(success ? `${name}: muvaffaqiyatli bajarildi.` : `${name}: xatolik yuz berdi.`);
  },

  fileDropped() {
    speak(pick(["Faylni oldim! U bilan nima qilamiz?", "Mazali fayl! Nima qilay?"]));
  },

  poked() {
    speak(pick(["Voy, qitiqlayapsiz!", "Hey, men ishlayapman!", "Hi-hi!"]));
  },
};
