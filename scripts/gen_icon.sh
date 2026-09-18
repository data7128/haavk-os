#!/usr/bin/env bash
# ============================================================
# HAAVK 徽标生成脚本（ImageMagick）—— 官方几何标志
# 图形：深色圆角底 + 白色三竖条（左窄/中高/右中）+ 斜向箭头菱形
# 配色：深色底 #0f1419 / 白色图形 #ffffff / 青蓝描边 #29c8e8
# ============================================================
set -euo pipefail

cd "$(dirname "$0")/.."
ASSETS=assets
mkdir -p "$ASSETS"

# 主徽标：深色圆角底 + 白色 HAAVK 几何标志（256x256）
# 坐标系：中心 (128,128)，图形区 x 60~196 / y 48~208
convert -size 256x256 xc:none \
  -fill '#0f1419' -draw 'roundrectangle 8,8 248,248 44,44' \
  -fill '#16202a' -draw 'roundrectangle 16,16 240,240 36,36' \
  -stroke '#29c8e8' -strokewidth 5 -fill 'none' -draw 'roundrectangle 20,20 236,236 32,32' \
  -stroke 'none' -fill '#ffffff' \
  -draw 'rectangle 66,78 90,190' \
  -draw 'rectangle 106,50 142,206' \
  -draw 'rectangle 156,86 178,190' \
  -draw 'polygon 156,148 198,108 198,184 156,148' \
  "$ASSETS/haavk_256.png"

# 生成多尺寸（用于 ICO）
for s in 128 64 48 32 16; do
  convert "$ASSETS/haavk_256.png" -resize ${s}x${s} "$ASSETS/haavk_${s}.png"
done

# 合成 ICO（Windows exe 图标，多尺寸内置）
convert "$ASSETS/haavk_256.png" "$ASSETS/haavk_128.png" "$ASSETS/haavk_64.png" \
        "$ASSETS/haavk_48.png" "$ASSETS/haavk_32.png" "$ASSETS/haavk_16.png" \
        "$ASSETS/haavk.ico"

# 复制一份 64px 作为 README 预览
cp "$ASSETS/haavk_64.png" "$ASSETS/haavk_logo_preview.png"

echo "✅ 图标已生成："
ls -la "$ASSETS"/haavk_*.png "$ASSETS/haavk.ico"
