#!/bin/bash
# Val-OS GUI Demo - 在真实终端中运行

cd "$(dirname "$0")/.."

# 编译
./wave/bin/wave val-os/real_test.wave -o real_test 2>/dev/null

# 运行并转换转义序列
echo -e "\033[2J\033[H"
./real_test 2>&1 | while IFS= read -r line; do
    # 转换 \x1b 到真实的 ESC
    echo -e "$(echo "$line" | sed 's/\\x1b/\x1b/g')"
done
