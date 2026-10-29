#!/usr/bin/env bash
# Builds Mochi's Uzbek voice: Meta's MMS TTS for Uzbek in Latin script
# (https://huggingface.co/facebook/mms-tts, models/uzb-script_latin, CC BY-NC 4.0)
# as model.onnx + tokens.txt for sherpa-onnx, in $1 (default: ./out).
# Needs Python 3.10 (torch 1.13), git and a network. Run by build-windows.yml.
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
out="$(mkdir -p "${1:-out}" && cd "${1:-out}" && pwd)"
work="$(mktemp -d)"
cd "$work"

export language=uzb-script_latin
base=https://huggingface.co/facebook/mms-tts/resolve/main/models/$language
curl -fsSL -o G_100000.pth "$base/G_100000.pth"
curl -fsSL -o config.json "$base/config.json"
curl -fsSL -o vocab.txt "$base/vocab.txt"

python -m pip install -q "numpy<2" scipy Cython "onnx<1.17"
python -m pip install -q torch==1.13.1+cpu -f https://download.pytorch.org/whl/torch_stable.html

git clone -q --depth 1 https://huggingface.co/spaces/mms-meta/MMS
export PYTHONPATH="$PWD/MMS:$PWD/MMS/vits:${PYTHONPATH:-}"
(
  cd MMS/vits/monotonic_align
  python setup.py build > /dev/null
  cp build/lib*/vits/monotonic_align/core*.so .
  sed -i.bak s/.monotonic_align.core/.core/g ./__init__.py
)

python "$here/export.py"

cp model.onnx tokens.txt "$out/"
cat > "$out/LICENSE.txt" <<TXT
O‘zbekcha ovoz: Meta MMS text-to-speech (uzb-script_latin),
https://huggingface.co/facebook/mms-tts — CC BY-NC 4.0 (tijorat maqsadida emas).
sherpa-onnx uchun ONNX ga o‘tkazilgan.
TXT
ls -la "$out"
