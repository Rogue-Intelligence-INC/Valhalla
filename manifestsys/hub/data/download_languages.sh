#!/bin/bash
# 🌍 多语言数据下载脚本
# 下载主流语言的词汇、语法、文本数据

set -e

DATA_DIR="/home/jouly/AI/Valhalla/manifestsys/hub/data/languages"
mkdir -p "$DATA_DIR"

echo "╔════════════════════════════════════════════════════════════╗"
echo "║     🌍 多语言数据下载 - Valhalla 语言核提炼实验            ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

# 1. 下载 Universal Dependencies (语法树库)
echo "📥 [1/4] 下载 Universal Dependencies 语法数据..."
UD_DIR="$DATA_DIR/universal_dependencies"
mkdir -p "$UD_DIR"

# 主要语言的 UD 数据
LANGUAGES=(
    "en:English-EWT"
    "zh:Chinese-GSD"
    "ja:Japanese-GSD"
    "de:German-GSD"
    "fr:French-GSD"
    "es:Spanish-GSD"
    "ru:Russian-GSD"
    "ar:Arabic-PADT"
    "ko:Korean-GSD"
    "hi:Hindi-HDTB"
    "pt:Portuguese-GSD"
    "it:Italian-ISDT"
    "nl:Dutch-Alpino"
    "pl:Polish-PDB"
    "tr:Turkish-IMST"
)

for lang_data in "${LANGUAGES[@]}"; do
    IFS=':' read -r code name <<< "$lang_data"
    echo "   → $name ($code)..."
    UD_URL="https://raw.githubusercontent.com/UniversalDependencies/UD_${name}/master/${code}_${name,,}-ud-train.conllu"
    # 尝试下载，失败则跳过
    curl -sL --max-time 30 "$UD_URL" -o "$UD_DIR/${code}.conllu" 2>/dev/null || echo "     (跳过: $name)"
done

# 2. 下载基础词汇表
echo ""
echo "📥 [2/4] 创建多语言基础词汇表..."
VOCAB_DIR="$DATA_DIR/vocabulary"
mkdir -p "$VOCAB_DIR"

# 创建常用词汇文件
cat > "$VOCAB_DIR/common_words.json" << 'EOF'
{
  "en": {
    "greetings": ["hello", "hi", "good morning", "good evening", "goodbye", "see you"],
    "numbers": ["one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten"],
    "colors": ["red", "blue", "green", "yellow", "black", "white", "orange", "purple", "pink", "brown"],
    "common_verbs": ["be", "have", "do", "say", "get", "make", "go", "know", "take", "see", "come", "think", "look", "want", "give", "use", "find", "tell", "ask", "work", "seem", "feel", "try", "leave", "call"],
    "common_nouns": ["time", "year", "people", "way", "day", "man", "thing", "woman", "life", "child", "world", "school", "state", "family", "student", "group", "country", "problem", "hand", "part"]
  },
  "zh": {
    "greetings": ["你好", "早上好", "晚上好", "再见", "谢谢", "不客气"],
    "numbers": ["一", "二", "三", "四", "五", "六", "七", "八", "九", "十"],
    "colors": ["红色", "蓝色", "绿色", "黄色", "黑色", "白色", "橙色", "紫色", "粉色", "棕色"],
    "common_verbs": ["是", "有", "做", "说", "去", "来", "看", "想", "知道", "给", "用", "找", "问", "工作", "学习", "吃", "喝", "走", "跑", "写"],
    "common_nouns": ["时间", "人", "年", "天", "事", "东西", "地方", "问题", "工作", "生活", "世界", "国家", "学校", "家", "朋友", "孩子", "水", "书", "钱", "话"]
  },
  "ja": {
    "greetings": ["こんにちは", "おはようございます", "こんばんは", "さようなら", "ありがとう", "すみません"],
    "numbers": ["一", "二", "三", "四", "五", "六", "七", "八", "九", "十"],
    "colors": ["赤", "青", "緑", "黄色", "黒", "白", "オレンジ", "紫", "ピンク", "茶色"],
    "common_verbs": ["する", "ある", "いる", "なる", "言う", "行く", "来る", "見る", "思う", "知る", "取る", "使う", "出る", "入る", "持つ"],
    "common_nouns": ["時間", "人", "年", "日", "事", "物", "所", "問題", "仕事", "生活", "世界", "国", "学校", "家", "友達"]
  },
  "de": {
    "greetings": ["Hallo", "Guten Morgen", "Guten Abend", "Auf Wiedersehen", "Danke", "Bitte"],
    "numbers": ["eins", "zwei", "drei", "vier", "fünf", "sechs", "sieben", "acht", "neun", "zehn"],
    "colors": ["rot", "blau", "grün", "gelb", "schwarz", "weiß", "orange", "lila", "rosa", "braun"],
    "common_verbs": ["sein", "haben", "werden", "können", "müssen", "sagen", "machen", "gehen", "wissen", "sehen", "lassen", "stehen", "finden", "bleiben", "liegen"],
    "common_nouns": ["Zeit", "Jahr", "Mensch", "Tag", "Frau", "Mann", "Kind", "Land", "Teil", "Haus", "Welt", "Hand", "Leben", "Kopf", "Arbeit"]
  },
  "fr": {
    "greetings": ["Bonjour", "Bonsoir", "Salut", "Au revoir", "Merci", "De rien"],
    "numbers": ["un", "deux", "trois", "quatre", "cinq", "six", "sept", "huit", "neuf", "dix"],
    "colors": ["rouge", "bleu", "vert", "jaune", "noir", "blanc", "orange", "violet", "rose", "marron"],
    "common_verbs": ["être", "avoir", "faire", "dire", "pouvoir", "aller", "voir", "savoir", "vouloir", "venir", "devoir", "prendre", "trouver", "donner", "falloir"],
    "common_nouns": ["temps", "année", "jour", "homme", "femme", "vie", "monde", "main", "chose", "fois", "pays", "lieu", "travail", "cas", "fait"]
  },
  "es": {
    "greetings": ["Hola", "Buenos días", "Buenas noches", "Adiós", "Gracias", "De nada"],
    "numbers": ["uno", "dos", "tres", "cuatro", "cinco", "seis", "siete", "ocho", "nueve", "diez"],
    "colors": ["rojo", "azul", "verde", "amarillo", "negro", "blanco", "naranja", "morado", "rosa", "marrón"],
    "common_verbs": ["ser", "estar", "haber", "tener", "hacer", "poder", "decir", "ir", "ver", "dar", "saber", "querer", "llegar", "pasar", "deber"],
    "common_nouns": ["tiempo", "año", "día", "hombre", "mujer", "vida", "mundo", "mano", "cosa", "vez", "país", "lugar", "trabajo", "caso", "parte"]
  },
  "ru": {
    "greetings": ["Привет", "Доброе утро", "Добрый вечер", "До свидания", "Спасибо", "Пожалуйста"],
    "numbers": ["один", "два", "три", "четыре", "пять", "шесть", "семь", "восемь", "девять", "десять"],
    "colors": ["красный", "синий", "зелёный", "жёлтый", "чёрный", "белый", "оранжевый", "фиолетовый", "розовый", "коричневый"],
    "common_verbs": ["быть", "мочь", "сказать", "знать", "стать", "хотеть", "говорить", "думать", "видеть", "идти", "стоять", "иметь", "взять", "дать", "понять"],
    "common_nouns": ["время", "год", "человек", "день", "рука", "дело", "жизнь", "слово", "место", "лицо", "друг", "глаз", "вопрос", "дом", "сторона"]
  },
  "ar": {
    "greetings": ["مرحبا", "صباح الخير", "مساء الخير", "مع السلامة", "شكرا", "عفوا"],
    "numbers": ["واحد", "اثنان", "ثلاثة", "أربعة", "خمسة", "ستة", "سبعة", "ثمانية", "تسعة", "عشرة"],
    "colors": ["أحمر", "أزرق", "أخضر", "أصفر", "أسود", "أبيض", "برتقالي", "بنفسجي", "وردي", "بني"],
    "common_verbs": ["كان", "قال", "كون", "أخذ", "جاء", "علم", "أعطى", "وجد", "ذهب", "رأى", "أراد", "فعل", "وقف", "جعل", "عرف"],
    "common_nouns": ["يوم", "سنة", "وقت", "رجل", "شيء", "بيت", "عمل", "حياة", "عالم", "امرأة", "ولد", "كلمة", "ماء", "مكان", "يد"]
  },
  "ko": {
    "greetings": ["안녕하세요", "좋은 아침", "좋은 저녁", "안녕히 가세요", "감사합니다", "천만에요"],
    "numbers": ["하나", "둘", "셋", "넷", "다섯", "여섯", "일곱", "여덟", "아홉", "열"],
    "colors": ["빨강", "파랑", "초록", "노랑", "검정", "하양", "주황", "보라", "분홍", "갈색"],
    "common_verbs": ["하다", "있다", "되다", "가다", "오다", "보다", "주다", "알다", "받다", "만들다", "먹다", "살다", "쓰다", "나가다", "들어오다"],
    "common_nouns": ["사람", "것", "때", "말", "일", "집", "눈", "손", "곳", "날", "년", "물", "몸", "나라", "생각"]
  },
  "hi": {
    "greetings": ["नमस्ते", "सुप्रभात", "शुभ संध्या", "अलविदा", "धन्यवाद", "कृपया"],
    "numbers": ["एक", "दो", "तीन", "चार", "पाँच", "छह", "सात", "आठ", "नौ", "दस"],
    "colors": ["लाल", "नीला", "हरा", "पीला", "काला", "सफेद", "नारंगी", "बैंगनी", "गुलाबी", "भूरा"],
    "common_verbs": ["होना", "करना", "जाना", "आना", "देखना", "लेना", "देना", "कहना", "मिलना", "रहना", "चाहना", "सोचना", "बनाना", "पड़ना", "लगना"],
    "common_nouns": ["समय", "साल", "आदमी", "दिन", "बात", "घर", "काम", "जगह", "हाथ", "आँख", "सिर", "दुनिया", "देश", "पानी", "जीवन"]
  }
}
EOF

# 3. 创建俗语/成语数据
echo ""
echo "📥 [3/4] 创建多语言俗语/成语库..."
IDIOM_DIR="$DATA_DIR/idioms"
mkdir -p "$IDIOM_DIR"

cat > "$IDIOM_DIR/idioms.json" << 'EOF'
{
  "en": [
    {"idiom": "break a leg", "meaning": "good luck", "literal": "打断一条腿"},
    {"idiom": "piece of cake", "meaning": "very easy", "literal": "一块蛋糕"},
    {"idiom": "bite the bullet", "meaning": "face difficulty bravely", "literal": "咬住子弹"},
    {"idiom": "cost an arm and a leg", "meaning": "very expensive", "literal": "花费一只胳膊和一条腿"},
    {"idiom": "let the cat out of the bag", "meaning": "reveal a secret", "literal": "把猫从袋子里放出来"},
    {"idiom": "hit the nail on the head", "meaning": "exactly right", "literal": "击中钉子头"},
    {"idiom": "once in a blue moon", "meaning": "very rarely", "literal": "蓝月亮出现时"},
    {"idiom": "the ball is in your court", "meaning": "it's your decision", "literal": "球在你的场上"},
    {"idiom": "spill the beans", "meaning": "reveal secret information", "literal": "洒出豆子"},
    {"idiom": "kill two birds with one stone", "meaning": "accomplish two things at once", "literal": "一石二鸟"}
  ],
  "zh": [
    {"idiom": "画蛇添足", "pinyin": "huà shé tiān zú", "meaning": "做多余的事"},
    {"idiom": "守株待兔", "pinyin": "shǒu zhū dài tù", "meaning": "不努力而想获得成功"},
    {"idiom": "对牛弹琴", "pinyin": "duì niú tán qín", "meaning": "对不懂的人讲道理"},
    {"idiom": "井底之蛙", "pinyin": "jǐng dǐ zhī wā", "meaning": "见识短浅的人"},
    {"idiom": "班门弄斧", "pinyin": "bān mén nòng fǔ", "meaning": "在行家面前卖弄"},
    {"idiom": "杯水车薪", "pinyin": "bēi shuǐ chē xīn", "meaning": "力量太小，解决不了问题"},
    {"idiom": "破釜沉舟", "pinyin": "pò fǔ chén zhōu", "meaning": "下定决心，不留退路"},
    {"idiom": "望梅止渴", "pinyin": "wàng méi zhǐ kě", "meaning": "用空想安慰自己"},
    {"idiom": "掩耳盗铃", "pinyin": "yǎn ěr dào líng", "meaning": "自欺欺人"},
    {"idiom": "叶公好龙", "pinyin": "yè gōng hào lóng", "meaning": "表面喜欢实际害怕"}
  ],
  "ja": [
    {"idiom": "猿も木から落ちる", "romaji": "saru mo ki kara ochiru", "meaning": "専門家も失敗する"},
    {"idiom": "石の上にも三年", "romaji": "ishi no ue ni mo sannen", "meaning": "忍耐すれば成功する"},
    {"idiom": "七転び八起き", "romaji": "nana korobi ya oki", "meaning": "何度失敗しても諦めない"},
    {"idiom": "花より団子", "romaji": "hana yori dango", "meaning": "実用的なものを好む"},
    {"idiom": "猫に小判", "romaji": "neko ni koban", "meaning": "価値がわからない人に貴重品"},
    {"idiom": "井の中の蛙", "romaji": "i no naka no kawazu", "meaning": "視野が狭い"},
    {"idiom": "馬の耳に念仏", "romaji": "uma no mimi ni nenbutsu", "meaning": "聞いても理解しない"},
    {"idiom": "案ずるより産むが易し", "romaji": "anzuru yori umu ga yasushi", "meaning": "心配より実行"},
    {"idiom": "鬼に金棒", "romaji": "oni ni kanabou", "meaning": "強いものがさらに強くなる"},
    {"idiom": "能ある鷹は爪を隠す", "romaji": "nou aru taka wa tsume wo kakusu", "meaning": "優れた人は才能を見せびらかさない"}
  ],
  "de": [
    {"idiom": "Da liegt der Hund begraben", "meaning": "das ist der Kern des Problems", "literal": "那里埋着狗"},
    {"idiom": "Tomaten auf den Augen haben", "meaning": "etwas Offensichtliches nicht sehen", "literal": "眼睛上有番茄"},
    {"idiom": "Die Daumen drücken", "meaning": "Glück wünschen", "literal": "按住大拇指"},
    {"idiom": "Ich verstehe nur Bahnhof", "meaning": "ich verstehe nichts", "literal": "我只理解火车站"},
    {"idiom": "Das ist mir Wurst", "meaning": "das ist mir egal", "literal": "这对我来说是香肠"}
  ],
  "fr": [
    {"idiom": "Avoir le cafard", "meaning": "être triste", "literal": "有蟑螂"},
    {"idiom": "Coûter les yeux de la tête", "meaning": "très cher", "literal": "花费头上的眼睛"},
    {"idiom": "Poser un lapin", "meaning": "ne pas venir à un rendez-vous", "literal": "放一只兔子"},
    {"idiom": "Avoir un chat dans la gorge", "meaning": "avoir la voix enrouée", "literal": "喉咙里有只猫"},
    {"idiom": "Il pleut des cordes", "meaning": "il pleut très fort", "literal": "下绳子雨"}
  ],
  "es": [
    {"idiom": "Estar en las nubes", "meaning": "estar distraído", "literal": "在云中"},
    {"idiom": "Costar un ojo de la cara", "meaning": "muy caro", "literal": "花费脸上的一只眼睛"},
    {"idiom": "Meter la pata", "meaning": "cometer un error", "literal": "把爪子伸进去"},
    {"idiom": "Tomar el pelo", "meaning": "burlarse", "literal": "拿头发"},
    {"idiom": "No hay mal que por bien no venga", "meaning": "algo bueno viene de algo malo", "literal": "没有不带来好处的坏事"}
  ],
  "ru": [
    {"idiom": "Вешать лапшу на уши", "meaning": "обманывать", "literal": "往耳朵上挂面条"},
    {"idiom": "Как с гуся вода", "meaning": "ничего не действует", "literal": "像鹅身上的水"},
    {"idiom": "Делать из мухи слона", "meaning": "преувеличивать", "literal": "把苍蝇变成大象"},
    {"idiom": "Медведь на ухо наступил", "meaning": "нет музыкального слуха", "literal": "熊踩了耳朵"},
    {"idiom": "Когда рак на горе свистнет", "meaning": "никогда", "literal": "当螃蟹在山上吹口哨时"}
  ]
}
EOF

# 4. 创建语法规则摘要
echo ""
echo "📥 [4/4] 创建语法规则摘要..."
GRAMMAR_DIR="$DATA_DIR/grammar"
mkdir -p "$GRAMMAR_DIR"

cat > "$GRAMMAR_DIR/grammar_patterns.json" << 'EOF'
{
  "en": {
    "word_order": "SVO",
    "patterns": ["Subject + Verb + Object", "Adjective + Noun", "Noun + of + Noun"],
    "tenses": ["present", "past", "future", "present_perfect", "past_perfect", "future_perfect"],
    "features": ["articles", "prepositions", "auxiliary_verbs", "modal_verbs"]
  },
  "zh": {
    "word_order": "SVO",
    "patterns": ["主语 + 动词 + 宾语", "定语 + 的 + 名词", "状语 + 地 + 动词"],
    "tenses": ["时态通过时间词表示"],
    "features": ["量词", "语气词", "补语", "把字句", "被字句"]
  },
  "ja": {
    "word_order": "SOV",
    "patterns": ["主語 + 目的語 + 動詞", "名詞 + の + 名詞", "形容詞 + 名詞"],
    "tenses": ["非過去形", "過去形", "進行形", "完了形"],
    "features": ["助詞", "敬語", "受身", "使役"]
  },
  "de": {
    "word_order": "SOV/V2",
    "patterns": ["Subjekt + Verb + Objekt (Hauptsatz)", "Verb am Ende (Nebensatz)"],
    "tenses": ["Präsens", "Präteritum", "Perfekt", "Plusquamperfekt", "Futur I", "Futur II"],
    "features": ["cases", "gender", "separable_verbs", "modal_particles"]
  },
  "fr": {
    "word_order": "SVO",
    "patterns": ["Sujet + Verbe + Objet", "Nom + Adjectif", "Article + Nom"],
    "tenses": ["présent", "passé_composé", "imparfait", "plus-que-parfait", "futur_simple", "conditionnel"],
    "features": ["gender", "liaisons", "subjunctive", "reflexive_verbs"]
  },
  "es": {
    "word_order": "SVO",
    "patterns": ["Sujeto + Verbo + Objeto", "Nombre + Adjetivo", "Artículo + Nombre"],
    "tenses": ["presente", "pretérito", "imperfecto", "futuro", "condicional", "subjuntivo"],
    "features": ["gender", "ser_estar", "reflexive_verbs", "personal_a"]
  },
  "ru": {
    "word_order": "SVO (flexible)",
    "patterns": ["Субъект + Глагол + Объект", "Прилагательное + Существительное"],
    "tenses": ["настоящее", "прошедшее", "будущее"],
    "features": ["cases", "aspect", "gender", "animate_inanimate"]
  },
  "ar": {
    "word_order": "VSO",
    "patterns": ["فعل + فاعل + مفعول", "اسم + صفة", "مضاف + مضاف إليه"],
    "tenses": ["الماضي", "المضارع", "الأمر"],
    "features": ["root_system", "gender", "dual_number", "case_endings"]
  },
  "ko": {
    "word_order": "SOV",
    "patterns": ["주어 + 목적어 + 동사", "관형어 + 명사", "부사어 + 동사"],
    "tenses": ["현재", "과거", "미래"],
    "features": ["particles", "honorifics", "verb_endings", "topic_marker"]
  },
  "hi": {
    "word_order": "SOV",
    "patterns": ["कर्ता + कर्म + क्रिया", "विशेषण + संज्ञा"],
    "tenses": ["वर्तमान", "भूत", "भविष्य"],
    "features": ["postpositions", "gender", "honorifics", "compound_verbs"]
  }
}
EOF

# 统计下载结果
echo ""
echo "╔════════════════════════════════════════════════════════════╗"
echo "║                    📊 下载完成统计                         ║"
echo "╠════════════════════════════════════════════════════════════╣"

VOCAB_COUNT=$(cat "$VOCAB_DIR/common_words.json" | grep -o '"[a-z][a-z]":' | wc -l)
IDIOM_COUNT=$(cat "$IDIOM_DIR/idioms.json" | grep -o '"idiom"' | wc -l)
UD_COUNT=$(ls -1 "$UD_DIR"/*.conllu 2>/dev/null | wc -l)

echo "║ 词汇语言数: $VOCAB_COUNT                                           ║"
echo "║ 俗语条目数: $IDIOM_COUNT                                          ║"
echo "║ UD语法文件: $UD_COUNT                                            ║"
echo "║ 语法规则数: 10                                           ║"
echo "╚════════════════════════════════════════════════════════════╝"

echo ""
echo "✅ 数据目录: $DATA_DIR"
echo "✅ 准备就绪，可以运行语言核提炼实验！"
