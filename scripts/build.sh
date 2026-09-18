#!/usr/bin/env bash
# HAAVK OS 一键构建脚本
# 用法：bash scripts/build.sh [--release]
set -euo pipefail

cd "$(dirname "$0")/.."

MODE="debug"
if [[ "${1:-}" == "--release" ]]; then
  MODE="release"
fi

echo "==> 生成徽标资源（如已存在则跳过）"
if [[ ! -f assets/haavk.ico ]]; then
  bash scripts/gen_icon.sh
fi

echo "==> 构建 HAAVK（$MODE）"
export PATH="$HOME/.cargo/bin:$PATH"
CARGO_PROFILE_OPT=""
if [[ "$MODE" == "release" ]]; then
  CARGO_PROFILE_OPT="--release"
fi

cargo build $CARGO_PROFILE_OPT -p mandel_core -p haavk_desktop

echo ""
echo "✅ 构建完成"
echo "   mandel-core CLI:  target/$MODE/mandel-core"
echo "   HAAVK 桌面:       target/$MODE/haavk"
echo ""
echo "   快速自检核心: target/$MODE/mandel-core --config config/mandel_core.toml --about"
echo "   启动桌面:    target/$MODE/haavk --config config/mandel_core.toml"
