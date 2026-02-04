import re

# 读取原始对话
with open('/tmp/valhalla_dialogues_raw.txt', 'r', encoding='utf-8') as f:
    lines = f.readlines()

# 解析对话
dialogues = []
for i, line in enumerate(lines):
    # 提取语言和内容
    match = re.search(r'\[([a-z]+)\]\s*(.+)', line)
    if match:
        lang = match.group(1)
        content = match.group(2).strip()
        # 检查是否有语法标注
        grammar_match = re.search(r'\(([A-Z\s_]+)\)$', content)
        if grammar_match:
            grammar = grammar_match.group(1)
            content_clean = content[:content.rfind('(')].strip()
            item_type = 'grammar'
        else:
            grammar = ''
            content_clean = content
            # 判断类型
            if len(content_clean) < 20 and ' ' not in content_clean:
                item_type = 'vocabulary'
            else:
                item_type = 'sentence'
        
        dialogues.append({
            'id': i + 1,
            'language': lang,
            'type': item_type,
            'content': content_clean,
            'grammar': grammar,
            'raw': line.strip()
        })

# 输出为 CSV
with open('valhalla_all_dialogues.csv', 'w', encoding='utf-8') as f:
    f.write('id,language,type,content\n')
    for d in dialogues:
        content_escaped = d['content'].replace('"', '""')
        f.write(f'{d["id"]},{d["language"]},{d["type"]},"{content_escaped}"\n')

# 输出为详细文本
with open('valhalla_all_dialogues.txt', 'w', encoding='utf-8') as f:
    f.write('# Valhalla 对话记录 - 深度分析用\n')
    f.write(f'# 总条数: {len(dialogues)}\n')
    f.write('# 格式: [序号] [语言] [类型] 内容\n')
    f.write('=' * 80 + '\n\n')
    
    for d in dialogues:
        f.write(f'[{d["id"]:03d}] [{d["language"]:10s}] [{d["type"]:10s}] {d["content"]}\n')
        if d['grammar']:
            f.write(f'      语法: {d["grammar"]}\n')
        f.write('\n')

# 统计
stats = {}
for d in dialogues:
    lang = d['language']
    dtype = d['type']
    if lang not in stats:
        stats[lang] = {'sentence': 0, 'vocabulary': 0, 'grammar': 0}
    stats[lang][dtype] += 1

print('=' * 60)
print('语言统计:')
print('=' * 60)
print(f'{"语言":<12} {"句子":<8} {"词汇":<8} {"语法":<8} {"总计":<8}')
print('-' * 60)
for lang, counts in sorted(stats.items(), key=lambda x: -sum(x[1].values())):
    total = sum(counts.values())
    print(f'{lang:<12} {counts["sentence"]:<8} {counts["vocabulary"]:<8} {counts["grammar"]:<8} {total:<8}')
print('-' * 60)
print(f'{"总计":<12} {sum(s["sentence"] for s in stats.values()):<8} {sum(s["vocabulary"] for s in stats.values()):<8} {sum(s["grammar"] for s in stats.values()):<8} {len(dialogues):<8}')

print('\n✅ 已生成文件:')
print('   - valhalla_all_dialogues.csv (CSV格式)')
print('   - valhalla_all_dialogues.txt (详细文本格式)')
