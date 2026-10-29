#!/usr/bin/env python3
# Says a few sentences with the built Uzbek voice (sherpa-onnx's Python
# package), so the CI run leaves something to listen to: sample.wav in $1.
import sys
import sherpa_onnx
import numpy as np
import wave

d = sys.argv[1]
tokens = {line.rsplit(" ", 1)[0] for line in open(f"{d}/tokens.txt", encoding="utf-8") if " " in line}
apos = next((c for c in ["'", "ʻ", "‘", "’", "ʼ"] if c in tokens), "")
text = "Salom! Xayrli kun! Men Mochi. Sizga qanday yordam bera olaman? Claude Code ishni tugatdi. To‘g‘ri, o‘zbekcha gapiraman."
text = text.lower()
if "а" in tokens and "a" not in tokens:
    # A Cyrillic model: the same rules as to_cyrillic in src-tauri/src/uzvoice.rs.
    for a_, b_ in [("o‘", "ў"), ("g‘", "ғ"), ("sh", "ш"), ("ch", "ч"), ("yo", "ё"), ("yu", "ю"), ("ya", "я"), ("ye", "е")]:
        text = text.replace(a_, b_)
    table = dict(zip("abcdefghijklmnopqrstuvwxyz", "абкдефгҳижклмнопқрстуввхйз"))
    text = "".join(table.get(c, c) for c in text).replace("’", "ъ")
else:
    for c in "ʻ‘’ʼ'":
        text = text.replace(c, apos)
print("text", text)

tts = sherpa_onnx.OfflineTts(sherpa_onnx.OfflineTtsConfig(
    model=sherpa_onnx.OfflineTtsModelConfig(
        vits=sherpa_onnx.OfflineTtsVitsModelConfig(model=f"{d}/model.onnx", tokens=f"{d}/tokens.txt",
                                                   noise_scale=0.6, noise_scale_w=0.7),
        num_threads=2),
    max_num_sentences=1))
audio = tts.generate(text, sid=0, speed=1.05)
samples = np.clip(np.array(audio.samples), -1, 1)
print("sample rate", audio.sample_rate, "seconds", len(samples) / audio.sample_rate)
assert len(samples) > audio.sample_rate, "the voice said almost nothing"
with wave.open(f"{d}/sample.wav", "wb") as w:
    w.setnchannels(1)
    w.setsampwidth(2)
    w.setframerate(audio.sample_rate)
    w.writeframes((samples * 32767).astype("<i2").tobytes())
