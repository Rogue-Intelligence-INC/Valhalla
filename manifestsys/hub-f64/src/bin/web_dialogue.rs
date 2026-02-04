//! 🌌 Valhalla 自主对话系统
//!
//! Valhalla 特性:
//! - 主动输出（不是被动问答）
//! - 不限制响应时间
//! - 不限制回应几句
//! - 不限制回应模态（文字/代码/图像/音频描述等）

use hub_f64::{ValhallaHub, HubConfig, HubRequest, RequestType, Priority};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::time::{Duration, Instant};
use std::thread;
use std::collections::VecDeque;
use serde::{Serialize, Deserialize};
use axum::{
    Router,
    routing::{get, post},
    response::{Html, IntoResponse},
    extract::{State, ws::{WebSocket, WebSocketUpgrade, Message}, Json},
};
use tower_http::cors::CorsLayer;
use rand::Rng;

// ═══════════════════════════════════════════════════════════════════
//  数据结构
// ═══════════════════════════════════════════════════════════════════

#[derive(Clone)]
struct AppState {
    hub: Arc<Mutex<ValhallaHub>>,
    output_queue: Arc<Mutex<VecDeque<ValhallaOutput>>>,
    input_queue: Arc<Mutex<VecDeque<String>>>,
    training_state: Arc<Mutex<TrainingState>>,
    running: Arc<AtomicBool>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
struct TrainingState {
    // Fate 收敛周期
    cycle: u64,              // 完成的收敛轮数
    stable_count: u64,       // 当前稳定计数
    convergence_threshold: f64, // 收敛阈值
    fate_drift: f64,         // 当前 Fate 漂移量
    
    // 信号统计
    total_signals: u64,
    rate: f64,
    elapsed_secs: u64,
    
    // Fate 偏好
    fate: [f64; 4],  // BlackHole, MeshBrain, MultiNova, BaseForce
    
    // 数据聚合
    data_processed: DataAggregation,
    
    is_training: bool,
}

#[derive(Clone, Serialize, Deserialize, Default)]
struct DataAggregation {
    text_items: u64,
    image_items: u64,
    audio_items: u64,
    code_items: u64,
    total_bytes: u64,
}

/// Valhalla 的原始输出 - 她想输出什么就是什么
#[derive(Clone, Serialize, Deserialize)]
struct ValhallaOutput {
    tick: u64,
    raw: serde_json::Value,  // 原始数据，任何格式
}

#[derive(Deserialize)]
struct UserInput {
    content: String,
}

// ═══════════════════════════════════════════════════════════════════
//  Valhalla 自主思维循环
// ═══════════════════════════════════════════════════════════════════

fn valhalla_autonomous_loop(
    hub: Arc<Mutex<ValhallaHub>>,
    output_queue: Arc<Mutex<VecDeque<ValhallaOutput>>>,
    input_queue: Arc<Mutex<VecDeque<String>>>,
    training_state: Arc<Mutex<TrainingState>>,
    running: Arc<AtomicBool>,
) {
    let mut tick = 0u64;
    let start_time = Instant::now();
    
    // Fate 收敛追踪
    let mut prev_fate: [f64; 4] = [0.25, 0.25, 0.25, 0.25];
    let mut stable_count = 0u64;
    let mut cycle = 0u64;
    let convergence_threshold = 0.001;
    let stable_required = 50;
    
    // 当前轮的结果数据（用于下一轮处理）
    let mut cycle_result: Option<serde_json::Value> = None;
    
    // 数据聚合统计
    let mut data_agg = DataAggregation::default();
    
    println!("🌌 Valhalla 运行中...");
    
    while running.load(Ordering::Relaxed) {
        tick += 1;
        
        // 接收外部输入
        let user_input = {
            let mut queue = input_queue.lock().unwrap();
            queue.pop_front()
        };
        
        // 处理数据：优先处理上轮结果，其次外部输入
        let process_data = if let Some(ref result) = cycle_result {
            // 有上轮结果，处理它
            Some(result.clone())
        } else if let Some(ref input) = user_input {
            // 外部输入
            data_agg.text_items += 1;
            data_agg.total_bytes += input.len() as u64;
            Some(serde_json::json!({"input": input}))
        } else {
            None
        };
        
        // 发送到 Hub 处理
        let response = {
            let mut hub = hub.lock().unwrap();
            
            if let Some(data) = process_data {
                let request = HubRequest {
                    request_id: format!("c{}_t{}", cycle, tick),
                    request_type: RequestType::DataProcessing,
                    intensity: 0.8,
                    data: Some(data),
                    priority: Priority::Normal,
                };
                Some(hub.process(request))
            } else {
                // 无数据时的内部演化
                let request = HubRequest {
                    request_id: format!("evolve_{}", tick),
                    request_type: RequestType::Reasoning,
                    intensity: 0.3,
                    data: None,
                    priority: Priority::Low,
                };
                Some(hub.process(request))
            }
        };
        
        // 获取当前 Fate 状态
        let (current_fate, fate_drift) = {
            let hub = hub.lock().unwrap();
            let p = hub.get_fate_preferences();
            let fate: [f64; 4] = [
                *p.affinities.get(0).unwrap_or(&0.25),
                *p.affinities.get(1).unwrap_or(&0.25),
                *p.affinities.get(2).unwrap_or(&0.25),
                *p.affinities.get(3).unwrap_or(&0.25),
            ];
            let drift: f64 = fate.iter()
                .zip(prev_fate.iter())
                .map(|(a, b)| (a - b).abs())
                .sum();
            (fate, drift)
        };
        
        // 检测 Fate 收敛
        if fate_drift < convergence_threshold {
            stable_count += 1;
            if stable_count >= stable_required {
                // 收敛完成！保存当前结果作为下轮输入
                cycle += 1;
                stable_count = 0;
                
                // 保存本轮结果：Fate 状态作为下轮的输入数据
                cycle_result = Some(serde_json::json!({
                    "cycle": cycle,
                    "fate": current_fate,
                    "tick": tick,
                }));
                
                println!("✅ 周期 #{} 收敛完成，结果传递到下一轮", cycle);
                
                // 输出收敛事件
                let output = ValhallaOutput {
                    tick,
                    raw: serde_json::json!({
                        "event": "convergence",
                        "cycle": cycle,
                        "fate": current_fate,
                    }),
                };
                output_queue.lock().unwrap().push_back(output);
            }
        } else {
            stable_count = 0;
            // 正在演化中，清除上轮结果（已被处理）
            if cycle_result.is_some() && tick % 10 == 0 {
                cycle_result = None;
            }
        }
        
        prev_fate = current_fate;
        
        // 更新训练状态
        {
            let elapsed = start_time.elapsed();
            let rate = tick as f64 / elapsed.as_secs_f64().max(0.001);
            
            let mut ts = training_state.lock().unwrap();
            ts.cycle = cycle;
            ts.stable_count = stable_count;
            ts.convergence_threshold = convergence_threshold;
            ts.fate_drift = fate_drift;
            ts.total_signals = tick;
            ts.rate = rate;
            ts.elapsed_secs = elapsed.as_secs();
            ts.fate = current_fate;
            ts.data_processed = data_agg.clone();
            ts.is_training = true;
        }
        
        thread::sleep(Duration::from_millis(100));
    }
}

// ═══════════════════════════════════════════════════════════════════
//  Web 界面
// ═══════════════════════════════════════════════════════════════════

const HTML_PAGE: &str = r##"<!DOCTYPE html>
<html lang="zh">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Valhalla Training</title>
    <link href="https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500&display=swap" rel="stylesheet">
    <style>
        :root {
            --bg: #0a0a0f;
            --surface: #14141f;
            --border: #252535;
            --text: #e0e0e8;
            --dim: #606078;
            --bh: #ef4444;
            --mb: #06b6d4;
            --mn: #f59e0b;
            --bf: #22c55e;
        }
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: 'JetBrains Mono', monospace;
            background: var(--bg);
            color: var(--text);
            height: 100vh;
            overflow-y: auto;
            padding: 20px;
        }
        .main { max-width: 900px; margin: 0 auto; }
        
        /* 标题 */
        h1 { 
            font-size: 1.2rem; 
            color: var(--dim); 
            margin-bottom: 20px;
            font-weight: 400;
        }
        h1 span { color: var(--text); }
        
        /* 卡片 */
        .card {
            background: var(--surface);
            border: 1px solid var(--border);
            border-radius: 8px;
            padding: 16px;
            margin-bottom: 16px;
        }
        .card-title {
            font-size: 0.75rem;
            color: var(--dim);
            margin-bottom: 12px;
            text-transform: uppercase;
            letter-spacing: 0.5px;
        }
        
        /* Fate 收敛 */
        .cycle-row {
            display: flex;
            align-items: baseline;
            gap: 20px;
            margin-bottom: 16px;
        }
        .cycle-num {
            font-size: 3rem;
            font-weight: 500;
            color: var(--text);
        }
        .cycle-label { color: var(--dim); font-size: 0.85rem; }
        .drift {
            font-size: 0.9rem;
            color: var(--dim);
        }
        .drift-value { color: var(--mn); }
        .stable { color: var(--bf); }
        
        /* 收敛进度条 */
        .convergence-bar {
            height: 4px;
            background: var(--border);
            border-radius: 2px;
            overflow: hidden;
            margin-top: 8px;
        }
        .convergence-fill {
            height: 100%;
            background: linear-gradient(90deg, var(--mn), var(--bf));
            transition: width 0.3s;
        }
        
        /* Fate 系统 */
        .fate-grid {
            display: grid;
            grid-template-columns: repeat(4, 1fr);
            gap: 12px;
        }
        .fate-item {
            text-align: center;
            padding: 12px 8px;
            background: var(--bg);
            border-radius: 6px;
            border-left: 3px solid;
        }
        .fate-item.bh { border-color: var(--bh); }
        .fate-item.mb { border-color: var(--mb); }
        .fate-item.mn { border-color: var(--mn); }
        .fate-item.bf { border-color: var(--bf); }
        .fate-name { font-size: 0.7rem; color: var(--dim); }
        .fate-value { font-size: 1.1rem; margin-top: 4px; }
        .fate-item.bh .fate-value { color: var(--bh); }
        .fate-item.mb .fate-value { color: var(--mb); }
        .fate-item.mn .fate-value { color: var(--mn); }
        .fate-item.bf .fate-value { color: var(--bf); }
        
        /* 数据聚合 */
        .data-grid {
            display: grid;
            grid-template-columns: repeat(3, 1fr);
            gap: 12px;
        }
        .data-item {
            background: var(--bg);
            padding: 12px;
            border-radius: 6px;
        }
        .data-label { font-size: 0.7rem; color: var(--dim); }
        .data-value { font-size: 1.2rem; margin-top: 4px; }
        
        /* 统计 */
        .stats-row {
            display: flex;
            gap: 24px;
            font-size: 0.85rem;
            color: var(--dim);
        }
        .stats-row span { color: var(--text); }
        
        /* 可折叠对话 */
        .collapsible {
            margin-top: 16px;
        }
        .collapse-header {
            display: flex;
            align-items: center;
            gap: 8px;
            cursor: pointer;
            padding: 12px 16px;
            background: var(--surface);
            border: 1px solid var(--border);
            border-radius: 8px;
            font-size: 0.8rem;
            color: var(--dim);
        }
        .collapse-header:hover { background: var(--border); }
        .collapse-arrow {
            transition: transform 0.2s;
        }
        .collapse-header.open .collapse-arrow {
            transform: rotate(90deg);
        }
        .collapse-content {
            display: none;
            border: 1px solid var(--border);
            border-top: none;
            border-radius: 0 0 8px 8px;
            max-height: 300px;
            overflow-y: auto;
        }
        .collapse-content.open { display: block; }
        
        /* 对话内容 */
        .dialogue-list {
            padding: 12px;
            font-size: 0.8rem;
        }
        .dialogue-item {
            padding: 8px;
            margin-bottom: 8px;
            background: var(--bg);
            border-radius: 4px;
            border-left: 2px solid var(--border);
        }
        .dialogue-item.convergence {
            border-left-color: var(--bf);
            color: var(--bf);
        }
        
        /* 输入 */
        .input-row {
            display: flex;
            gap: 8px;
            padding: 12px;
            border-top: 1px solid var(--border);
        }
        .input-row input {
            flex: 1;
            background: var(--bg);
            border: 1px solid var(--border);
            border-radius: 4px;
            padding: 8px 12px;
            color: var(--text);
            font-family: inherit;
            font-size: 0.85rem;
        }
        .input-row button {
            background: var(--border);
            border: none;
            border-radius: 4px;
            padding: 8px 16px;
            color: var(--text);
            cursor: pointer;
            font-family: inherit;
        }
        .input-row button:hover { background: var(--dim); }
    </style>
</head>
<body>
    <div class="main">
        <h1>Valhalla <span>Training</span></h1>
        
        <!-- Fate 收敛状态 -->
        <div class="card">
            <div class="card-title">Fate Convergence</div>
            <div class="cycle-row">
                <div>
                    <div class="cycle-num" id="cycleNum">0</div>
                    <div class="cycle-label">cycles completed</div>
                </div>
                <div class="drift">
                    drift: <span class="drift-value" id="driftValue">0.0000</span>
                    &nbsp;|&nbsp;
                    stable: <span class="stable" id="stableCount">0</span>/50
                </div>
            </div>
            <div class="convergence-bar">
                <div class="convergence-fill" id="convergenceFill" style="width: 0%"></div>
            </div>
        </div>
        
        <!-- Fate 系统 -->
        <div class="card">
            <div class="card-title">Fate Systems</div>
            <div class="fate-grid">
                <div class="fate-item bh">
                    <div class="fate-name">BlackHole</div>
                    <div class="fate-value" id="fateBH">25.00%</div>
                </div>
                <div class="fate-item mb">
                    <div class="fate-name">MeshBrain</div>
                    <div class="fate-value" id="fateMB">25.00%</div>
                </div>
                <div class="fate-item mn">
                    <div class="fate-name">MultiNova</div>
                    <div class="fate-value" id="fateMN">25.00%</div>
                </div>
                <div class="fate-item bf">
                    <div class="fate-name">BaseForce</div>
                    <div class="fate-value" id="fateBF">25.00%</div>
                </div>
            </div>
        </div>
        
        <!-- 数据聚合 -->
        <div class="card">
            <div class="card-title">Data Aggregation</div>
            <div class="data-grid">
                <div class="data-item">
                    <div class="data-label">Text Items</div>
                    <div class="data-value" id="dataText">0</div>
                </div>
                <div class="data-item">
                    <div class="data-label">Total Bytes</div>
                    <div class="data-value" id="dataBytes">0</div>
                </div>
                <div class="data-item">
                    <div class="data-label">Signals</div>
                    <div class="data-value" id="dataSignals">0</div>
                </div>
            </div>
        </div>
        
        <!-- 统计 -->
        <div class="card">
            <div class="stats-row">
                <div>Rate: <span id="statRate">0</span>/s</div>
                <div>Elapsed: <span id="statElapsed">0:00:00</span></div>
            </div>
        </div>
        
        <!-- 可折叠对话 -->
        <div class="collapsible">
            <div class="collapse-header" id="dialogueToggle">
                <span class="collapse-arrow">▶</span>
                <span>Dialogue Interface</span>
            </div>
            <div class="collapse-content" id="dialogueContent">
                <div class="dialogue-list" id="dialogueList"></div>
                <div class="input-row">
                    <input type="text" id="inputField" placeholder="Input...">
                    <button id="sendBtn">Send</button>
                </div>
            </div>
        </div>
    </div>
    
    <script>
        const ws = new WebSocket(`ws://${location.host}/ws`);
        
        ws.onmessage = (event) => {
            const data = JSON.parse(event.data);
            if (data.type === 'training') {
                updateUI(data.state);
            } else if (data.type === 'output') {
                addDialogue(data.output);
            }
        };
        
        function updateUI(s) {
            if (!s) return;
            document.getElementById('cycleNum').textContent = s.cycle || 0;
            document.getElementById('driftValue').textContent = (s.fate_drift || 0).toFixed(6);
            document.getElementById('stableCount').textContent = s.stable_count || 0;
            document.getElementById('convergenceFill').style.width = ((s.stable_count || 0) / 50 * 100) + '%';
            
            if (s.fate) {
                document.getElementById('fateBH').textContent = ((s.fate[0] || 0.25) * 100).toFixed(2) + '%';
                document.getElementById('fateMB').textContent = ((s.fate[1] || 0.25) * 100).toFixed(2) + '%';
                document.getElementById('fateMN').textContent = ((s.fate[2] || 0.25) * 100).toFixed(2) + '%';
                document.getElementById('fateBF').textContent = ((s.fate[3] || 0.25) * 100).toFixed(2) + '%';
            }
            
            if (s.data_processed) {
                document.getElementById('dataText').textContent = s.data_processed.text_items || 0;
                document.getElementById('dataBytes').textContent = formatBytes(s.data_processed.total_bytes || 0);
            }
            document.getElementById('dataSignals').textContent = formatNumber(s.total_signals || 0);
            document.getElementById('statRate').textContent = Math.round(s.rate || 0);
            document.getElementById('statElapsed').textContent = formatTime(s.elapsed_secs || 0);
        }
        
        function formatNumber(n) {
            if (n >= 1000000) return (n / 1000000).toFixed(2) + 'M';
            if (n >= 1000) return (n / 1000).toFixed(1) + 'K';
            return n.toString();
        }
        
        function formatTime(secs) {
            const h = Math.floor(secs / 3600);
            const m = Math.floor((secs % 3600) / 60);
            const s = secs % 60;
            return h + ':' + String(m).padStart(2, '0') + ':' + String(s).padStart(2, '0');
        }
        
        function formatBytes(b) {
            if (b >= 1073741824) return (b / 1073741824).toFixed(2) + ' GB';
            if (b >= 1048576) return (b / 1048576).toFixed(1) + ' MB';
            if (b >= 1024) return (b / 1024).toFixed(0) + ' KB';
            return b + ' B';
        }
        
        function addDialogue(output) {
            const list = document.getElementById('dialogueList');
            const item = document.createElement('div');
            item.className = 'dialogue-item';
            if (output.raw && output.raw.event === 'convergence') {
                item.className += ' convergence';
                item.textContent = `Cycle #${output.raw.cycle} converged`;
            } else {
                item.textContent = `#${output.tick}: ${JSON.stringify(output.raw)}`;
            }
            list.appendChild(item);
            list.scrollTop = list.scrollHeight;
            // 限制显示数量
            while (list.children.length > 50) {
                list.removeChild(list.firstChild);
            }
        }
        
        // 可折叠对话
        document.getElementById('dialogueToggle').addEventListener('click', function() {
            this.classList.toggle('open');
            document.getElementById('dialogueContent').classList.toggle('open');
        });
        
        // 发送输入
        const inputField = document.getElementById('inputField');
        const sendBtn = document.getElementById('sendBtn');
        
        function sendMessage() {
            const content = inputField.value.trim();
            if (!content) return;
            fetch('/input', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ content })
            });
            inputField.value = '';
        }
        
        sendBtn.addEventListener('click', sendMessage);
        inputField.addEventListener('keydown', (e) => {
            if (e.key === 'Enter') {
                e.preventDefault();
                sendMessage();
            }
        });
    </script>
</body>
</html>
"##;

// ═══════════════════════════════════════════════════════════════════
//  路由处理
// ═══════════════════════════════════════════════════════════════════

async fn index_handler() -> Html<&'static str> {
    Html(HTML_PAGE)
}

async fn input_handler(
    State(state): State<AppState>,
    Json(input): Json<UserInput>,
) -> impl IntoResponse {
    state.input_queue.lock().unwrap().push_back(input.content);
    "OK"
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    loop {
        // 收集要发送的消息
        let outputs: Vec<ValhallaOutput> = {
            let mut queue = state.output_queue.lock().unwrap();
            let mut collected = Vec::new();
            while let Some(output) = queue.pop_front() {
                collected.push(output);
            }
            collected
        };
        
        // 发送输出
        for output in outputs {
            let json = serde_json::json!({
                "type": "output",
                "output": output
            });
            if socket.send(Message::Text(json.to_string())).await.is_err() {
                return;
            }
        }
        
        // 获取Fate状态
        let prefs = {
            let hub = state.hub.lock().unwrap();
            hub.get_fate_preferences().affinities.clone()
        };
        
        let json = serde_json::json!({
            "type": "fate",
            "preferences": prefs
        });
        if socket.send(Message::Text(json.to_string())).await.is_err() {
            return;
        }
        
        // 发送训练状态
        let training = {
            state.training_state.lock().unwrap().clone()
        };
        let json = serde_json::json!({
            "type": "training",
            "state": training
        });
        if socket.send(Message::Text(json.to_string())).await.is_err() {
            return;
        }
        
        tokio::time::sleep(Duration::from_millis(200)).await;
        
        if !state.running.load(Ordering::Relaxed) {
            break;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
//  主函数
// ═══════════════════════════════════════════════════════════════════

#[tokio::main]
async fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║     🌌 Valhalla 自主对话系统                                   ║");
    println!("║     http://localhost:8766                                      ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");
    
    let running = Arc::new(AtomicBool::new(true));
    
    // 初始化 Hub
    print!("🔧 初始化 Valhalla Hub...");
    let hub_config = HubConfig {
        resource_percent: 50,
        batch_size: 256,
        ..Default::default()
    };
    let hub = Arc::new(Mutex::new(ValhallaHub::with_config(hub_config)));
    println!(" ✅");
    
    let output_queue = Arc::new(Mutex::new(VecDeque::new()));
    let input_queue = Arc::new(Mutex::new(VecDeque::new()));
    let training_state = Arc::new(Mutex::new(TrainingState::default()));
    
    let app_state = AppState {
        hub: hub.clone(),
        output_queue: output_queue.clone(),
        input_queue: input_queue.clone(),
        training_state: training_state.clone(),
        running: running.clone(),
    };
    
    // Ctrl+C 处理
    let running_clone = running.clone();
    ctrlc::set_handler(move || {
        println!("\n⚠️  正在停止...");
        running_clone.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");
    
    // 启动自主思维循环
    let hub_clone = hub.clone();
    let output_clone = output_queue.clone();
    let input_clone = input_queue.clone();
    let training_clone = training_state.clone();
    let running_clone = running.clone();
    thread::spawn(move || {
        valhalla_autonomous_loop(hub_clone, output_clone, input_clone, training_clone, running_clone);
    });
    
    // 启动 Web 服务器
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/ws", get(ws_handler))
        .route("/input", post(input_handler))
        .layer(CorsLayer::permissive())
        .with_state(app_state);
    
    println!("🚀 Web 服务器启动: http://localhost:8766");
    println!("   打开浏览器与 Valhalla 对话");
    println!("   Valhalla 会自主思考和输出");
    println!("   按 Ctrl+C 停止\n");
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8766").await.unwrap();
    
    let running_for_server = running.clone();
    tokio::select! {
        _ = axum::serve(listener, app) => {}
        _ = async {
            while running_for_server.load(Ordering::Relaxed) {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        } => {}
    }
    
    println!("\n🏁 服务已停止");
}
