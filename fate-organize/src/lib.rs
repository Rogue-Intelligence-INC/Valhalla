//! 🌌 Fate Organizer - Valhalla 智能文件整理系统
//!
//! 把 Valhalla 的 Fate 系统能力向外输出，整理真实世界的文件系统
//!
//! ## 架构
//!
//! ```text
//!     ┌─────────────────────────────────────────────────────────┐
//!     │              Fate Organizer (Hub 反向应用)               │
//!     │   文件系统 ──→ Fate分析 ──→ 四元分类 ──→ 整理方案        │
//!     └─────────────────────────────────────────────────────────┘
//!                              │
//!          ┌──────────────────┼──────────────────┐
//!          │                  │                  │
//!    ┌─────▼─────┐     ┌─────▼─────┐     ┌─────▼─────┐
//!    │ BlackHole │     │ MeshBrain │     │ MultiNova │
//!    │  压缩/去重  │     │  语义分析  │     │  批量处理  │
//!    └───────────┘     └───────────┘     └───────────┘
//!          │                  │                  │
//!          └──────────────────┼──────────────────┘
//!                             │
//!                    ┌────────▼────────┐
//!                    │    BaseForce    │
//!                    │    协调整理      │
//!                    └─────────────────┘
//! ```
//!
//! ## 使用示例
//!
//! ```rust,no_run
//! use fate_organize::FateOrganizer;
//!
//! let organizer = FateOrganizer::new();
//! let plan = organizer.analyze_directory("/path/to/messy/folder")?;
//! organizer.execute_organization(&plan, false)?; // false = 实际执行
//! ```

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use serde::{Serialize, Deserialize};
use regex::Regex;

/// 文件信号 - 把文件转换为 Fate 可分析的信号
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSignal {
    /// 文件路径
    pub path: PathBuf,
    /// 文件名
    pub name: String,
    /// 扩展名
    pub extension: Option<String>,
    /// 文件大小 (bytes)
    pub size: u64,
    /// 特征向量 (用于模式匹配)
    pub features: FileFeatures,
}

/// 文件特征 - 从文件名中提取的模式
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FileFeatures {
    /// 包含日期模式 (YYYY-MM-DD, YYYYMMDD, etc.)
    pub date_pattern: Option<String>,
    /// 包含序号模式 (001, (1), _1, etc.)
    pub sequence_pattern: Option<String>,
    /// 关键词 (从文件名提取)
    pub keywords: Vec<String>,
    /// 来源标识 (DALL·E, Chatgpt, etc.)
    pub source_tag: Option<String>,
    /// 语言特征 (中文/英文/混合)
    pub language: LanguageType,
    /// 文件类型类别
    pub category: FileCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum LanguageType {
    Chinese,
    English,
    #[default]
    Mixed,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum FileCategory {
    /// AI 生成图片 (DALL·E, Midjourney, etc.)
    AIGeneratedImage,
    /// 截图
    Screenshot,
    /// 照片
    Photo,
    /// 视频
    Video,
    /// 音频
    Audio,
    /// 文档
    Document,
    /// 压缩包
    Archive,
    /// 项目/应用
    Application,
    /// 下载内容
    Download,
    #[default]
    Unknown,
}

/// 整理建议 - 四元系统的协调结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationSuggestion {
    /// 目标文件夹
    pub target_folder: String,
    /// 建议操作
    pub action: SuggestedAction,
    /// 置信度 (0-100)
    pub confidence: u32,
    /// 理由
    pub reason: String,
    /// 相关文件 (可能需要合并/分组)
    pub related_files: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestedAction {
    /// 移动到指定文件夹
    MoveTo(String),
    /// 重命名 (提供新名称模板)
    Rename(String),
    /// 归档 (压缩)
    Archive,
    /// 删除 (重复/临时文件)
    Delete,
    /// 保持原样
    Keep,
    /// 分组 (与相关文件放在一起)
    Group(String),
}

/// 整理方案 - 完整的整理计划
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationPlan {
    /// 源目录
    pub source_dir: PathBuf,
    /// 扫描时间
    pub scan_time: String,
    /// 文件总数
    pub total_files: usize,
    /// 目录总数
    pub total_dirs: usize,
    /// 分类统计
    pub category_stats: HashMap<String, usize>,
    /// 建议的文件夹结构
    pub suggested_structure: Vec<FolderStructure>,
    /// 每个文件的建议
    pub file_suggestions: Vec<(FileSignal, OrganizationSuggestion)>,
    /// 发现的模式
    pub discovered_patterns: Vec<DiscoveredPattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderStructure {
    pub name: String,
    pub description: String,
    pub expected_files: usize,
    pub subcategories: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredPattern {
    pub pattern_type: String,
    pub description: String,
    pub example_files: Vec<String>,
    pub count: usize,
}

/// 执行报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionReport {
    pub total_operations: usize,
    pub successful: usize,
    pub failed: usize,
    pub skipped: usize,
    pub operations: Vec<OperationResult>,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResult {
    pub source: PathBuf,
    pub target: PathBuf,
    pub status: OperationStatus,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationStatus {
    Success,
    Failed,
    Skipped,
    DryRun,
}

// ═══════════════════════════════════════════════════════════════════
// 高级文件夹操作 - 合并、重命名、批量处理
// ═══════════════════════════════════════════════════════════════════

/// 文件夹合并请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeRequest {
    /// 源文件夹列表
    pub sources: Vec<PathBuf>,
    /// 目标文件夹
    pub target: PathBuf,
    /// 冲突处理策略
    pub conflict_strategy: ConflictStrategy,
    /// 合并后删除源文件夹
    pub delete_sources: bool,
}

/// 冲突处理策略
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum ConflictStrategy {
    /// 跳过冲突文件
    #[default]
    Skip,
    /// 覆盖
    Overwrite,
    /// 重命名 (添加序号)
    Rename,
    /// 保留两者 (添加后缀)
    KeepBoth,
}

/// 批量重命名规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenameRule {
    /// 匹配模式 (正则)
    pub pattern: String,
    /// 替换模板
    pub replacement: String,
    /// 是否添加序号
    pub add_sequence: bool,
    /// 起始序号
    pub sequence_start: usize,
}

/// 合并报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeReport {
    pub total_files: usize,
    pub moved: usize,
    pub skipped: usize,
    pub conflicts: usize,
    pub errors: Vec<String>,
}

/// 重命名报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenameReport {
    pub total: usize,
    pub renamed: usize,
    pub skipped: usize,
    pub errors: Vec<String>,
}

/// 智能重命名报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartRenameReport {
    pub total: usize,
    pub renamed: usize,
    pub skipped: usize,
    pub unchanged: usize,
    pub operations: Vec<SmartRenameOp>,
    pub errors: Vec<String>,
}

/// 智能重命名操作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartRenameOp {
    pub old_name: String,
    pub new_name: String,
    pub reason: String,
    pub success: bool,
}

/// 🌌 Fate Organizer - Valhalla 智能文件整理系统
pub struct FateOrganizer {
    /// 特征提取器
    feature_extractor: FeatureExtractor,
    /// 模式数据库 (Fate 学习的模式)
    pattern_db: PatternDatabase,
}

/// 特征提取器 - 从文件名提取信号
pub struct FeatureExtractor {
    /// 日期正则
    date_patterns: Vec<Regex>,
    /// 序号正则
    sequence_patterns: Vec<Regex>,
    /// AI 来源标识
    ai_source_patterns: Vec<(Regex, String)>,
}

impl Default for FeatureExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl FeatureExtractor {
    pub fn new() -> Self {
        Self {
            date_patterns: vec![
                Regex::new(r"(\d{4})-(\d{2})-(\d{2})").unwrap(),           // 2023-12-14
                Regex::new(r"(\d{4})(\d{2})(\d{2})").unwrap(),             // 20231214
                Regex::new(r"(\d{4})\.(\d{2})\.(\d{2})").unwrap(),         // 2023.12.14
                Regex::new(r"(\d{2})-(\d{2})-(\d{4})").unwrap(),           // 12-14-2023
            ],
            sequence_patterns: vec![
                Regex::new(r"\((\d+)\)").unwrap(),                          // (1), (2)
                Regex::new(r"_(\d+)\.").unwrap(),                           // _1., _2.
                Regex::new(r"-(\d+)\.").unwrap(),                           // -1., -2.
                Regex::new(r"(\d{5,})-").unwrap(),                          // 00008-657520491
            ],
            ai_source_patterns: vec![
                (Regex::new(r"(?i)DALL[·\-]?E").unwrap(), "DALL-E".to_string()),
                (Regex::new(r"(?i)midjourney|MJ").unwrap(), "Midjourney".to_string()),
                (Regex::new(r"(?i)stable[\s_-]?diffusion|SD").unwrap(), "StableDiffusion".to_string()),
                (Regex::new(r"(?i)chatgpt|gpt").unwrap(), "ChatGPT".to_string()),
                (Regex::new(r"(?i)claude").unwrap(), "Claude".to_string()),
            ],
        }
    }
    
    /// 从文件提取特征
    pub fn extract(&self, path: &Path, size: u64) -> FileSignal {
        let name = path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        
        let extension = path.extension()
            .map(|e| e.to_string_lossy().to_lowercase());
        
        let features = self.extract_features(&name, &extension);
        
        FileSignal {
            path: path.to_path_buf(),
            name,
            extension,
            size,
            features,
        }
    }
    
    fn extract_features(&self, name: &str, extension: &Option<String>) -> FileFeatures {
        let mut features = FileFeatures::default();
        
        // 提取日期模式
        for pattern in &self.date_patterns {
            if let Some(m) = pattern.find(name) {
                features.date_pattern = Some(m.as_str().to_string());
                break;
            }
        }
        
        // 提取序号模式
        for pattern in &self.sequence_patterns {
            if let Some(caps) = pattern.captures(name) {
                if let Some(m) = caps.get(1) {
                    features.sequence_pattern = Some(m.as_str().to_string());
                    break;
                }
            }
        }
        
        // 识别 AI 来源
        for (pattern, source) in &self.ai_source_patterns {
            if pattern.is_match(name) {
                features.source_tag = Some(source.clone());
                break;
            }
        }
        
        // 判断语言类型
        features.language = self.detect_language(name);
        
        // 判断文件类别
        features.category = self.categorize_file(name, extension, &features);
        
        // 提取关键词
        features.keywords = self.extract_keywords(name);
        
        features
    }
    
    fn detect_language(&self, name: &str) -> LanguageType {
        let has_chinese = name.chars().any(|c| {
            let code = c as u32;
            (0x4E00..=0x9FFF).contains(&code) ||  // CJK 统一汉字
            (0x3400..=0x4DBF).contains(&code)     // CJK 扩展 A
        });
        
        let has_english = name.chars().any(|c| c.is_ascii_alphabetic());
        
        match (has_chinese, has_english) {
            (true, false) => LanguageType::Chinese,
            (false, true) => LanguageType::English,
            (true, true) => LanguageType::Mixed,
            (false, false) => LanguageType::Unknown,
        }
    }
    
    fn categorize_file(&self, name: &str, extension: &Option<String>, features: &FileFeatures) -> FileCategory {
        let ext = extension.as_ref().map(|s| s.as_str()).unwrap_or("");
        
        // 先根据来源判断
        if features.source_tag.is_some() {
            if matches!(ext, "png" | "jpg" | "jpeg" | "webp") {
                return FileCategory::AIGeneratedImage;
            }
        }
        
        // 根据扩展名判断
        match ext {
            "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "tiff" => {
                // 检查是否是截图
                if name.contains("Screenshot") || name.contains("截图") || 
                   Regex::new(r"\d{4}-\d{2}-\d{2}\s+\d{6}").unwrap().is_match(name) {
                    FileCategory::Screenshot
                } else if features.source_tag.is_some() {
                    FileCategory::AIGeneratedImage
                } else {
                    FileCategory::Photo
                }
            }
            "mp4" | "avi" | "mkv" | "mov" | "wmv" | "flv" | "webm" => FileCategory::Video,
            "mp3" | "wav" | "flac" | "aac" | "ogg" | "m4a" => FileCategory::Audio,
            "pdf" | "doc" | "docx" | "txt" | "md" | "ppt" | "pptx" | "xls" | "xlsx" => FileCategory::Document,
            "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" => FileCategory::Archive,
            "exe" | "msi" | "app" | "dmg" => FileCategory::Application,
            _ => FileCategory::Unknown,
        }
    }
    
    fn extract_keywords(&self, name: &str) -> Vec<String> {
        // 移除扩展名
        let name_no_ext = Path::new(name)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| name.to_string());
        
        // 分词：按空格、下划线、连字符分割
        let keywords: Vec<String> = name_no_ext
            .split(|c: char| c.is_whitespace() || c == '_' || c == '-' || c == '.')
            .filter(|s| !s.is_empty() && s.len() > 1)
            .filter(|s| {
                // 过滤纯数字和常见无意义词
                !s.chars().all(|c| c.is_numeric()) &&
                !["the", "a", "an", "of", "to", "in", "for", "on", "with"].contains(&s.to_lowercase().as_str())
            })
            .map(|s| s.to_string())
            .take(10)  // 最多10个关键词
            .collect();
        
        keywords
    }
}

/// 模式数据库 - Fate 学习的模式
#[derive(Default)]
pub struct PatternDatabase {
    /// 文件名前缀模式 -> 类别
    prefix_patterns: HashMap<String, String>,
    /// 关键词 -> 类别
    keyword_categories: HashMap<String, String>,
}

impl PatternDatabase {
    pub fn new() -> Self {
        let mut db = Self::default();
        
        // 预置一些已知模式
        db.prefix_patterns.insert("DALL·E".to_string(), "AI生成/DALL-E".to_string());
        db.prefix_patterns.insert("Chatgpt".to_string(), "AI生成/ChatGPT".to_string());
        db.prefix_patterns.insert("Screenshot".to_string(), "截图".to_string());
        db.prefix_patterns.insert("New folder".to_string(), "待整理".to_string());
        
        db.keyword_categories.insert("PPT".to_string(), "文档/演示".to_string());
        db.keyword_categories.insert("animation".to_string(), "视频/动画".to_string());
        db.keyword_categories.insert("music".to_string(), "音频/音乐".to_string());
        
        db
    }
    
    /// 根据特征推荐文件夹
    pub fn suggest_folder(&self, signal: &FileSignal) -> String {
        // 1. 检查来源标识
        if let Some(source) = &signal.features.source_tag {
            return format!("AI生成/{}", source);
        }
        
        // 2. 根据类别
        let category_folder = match signal.features.category {
            FileCategory::AIGeneratedImage => "AI生成/图片".to_string(),
            FileCategory::Screenshot => "截图".to_string(),
            FileCategory::Photo => "图片/照片".to_string(),
            FileCategory::Video => "视频".to_string(),
            FileCategory::Audio => "音频".to_string(),
            FileCategory::Document => "文档".to_string(),
            FileCategory::Archive => "压缩包".to_string(),
            FileCategory::Application => "应用程序".to_string(),
            FileCategory::Download => "下载".to_string(),
            FileCategory::Unknown => "其他".to_string(),
        };
        
        // 3. 如果有日期，可以按年月分
        if let Some(date) = &signal.features.date_pattern {
            if date.len() >= 7 {
                let year_month = &date[0..7].replace('-', "/");
                return format!("{}/{}", category_folder, year_month);
            }
        }
        
        category_folder
    }
}

impl Default for FateOrganizer {
    fn default() -> Self {
        Self::new()
    }
}

impl FateOrganizer {
    pub fn new() -> Self {
        Self {
            feature_extractor: FeatureExtractor::new(),
            pattern_db: PatternDatabase::new(),
        }
    }
    
    /// 🔥 执行整理 - 实际移动文件
    pub fn execute_organization(&self, plan: &OrganizationPlan, dry_run: bool) -> Result<ExecutionReport, std::io::Error> {
        let mut report = ExecutionReport {
            total_operations: 0,
            successful: 0,
            failed: 0,
            skipped: 0,
            operations: Vec::new(),
            dry_run,
        };
        
        let base_dir = &plan.source_dir;
        
        for (signal, suggestion) in &plan.file_suggestions {
            report.total_operations += 1;
            
            let target_folder = base_dir.join(&suggestion.target_folder);
            let target_path = target_folder.join(&signal.name);
            
            // 如果目标和源相同，跳过
            if signal.path == target_path {
                report.skipped += 1;
                report.operations.push(OperationResult {
                    source: signal.path.clone(),
                    target: target_path,
                    status: OperationStatus::Skipped,
                    message: "已在目标位置".to_string(),
                });
                continue;
            }
            
            if dry_run {
                // 干运行模式：只记录，不执行
                report.successful += 1;
                report.operations.push(OperationResult {
                    source: signal.path.clone(),
                    target: target_path,
                    status: OperationStatus::DryRun,
                    message: format!("将移动到 {}", suggestion.target_folder),
                });
            } else {
                // 实际执行
                match self.move_file(&signal.path, &target_path) {
                    Ok(_) => {
                        report.successful += 1;
                        report.operations.push(OperationResult {
                            source: signal.path.clone(),
                            target: target_path,
                            status: OperationStatus::Success,
                            message: "移动成功".to_string(),
                        });
                    }
                    Err(e) => {
                        report.failed += 1;
                        report.operations.push(OperationResult {
                            source: signal.path.clone(),
                            target: target_path,
                            status: OperationStatus::Failed,
                            message: format!("错误: {}", e),
                        });
                    }
                }
            }
        }
        
        Ok(report)
    }
    
    /// 移动单个文件
    fn move_file(&self, source: &Path, target: &Path) -> Result<(), std::io::Error> {
        // 确保目标目录存在
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // 如果目标文件已存在，添加序号
        let final_target = if target.exists() {
            self.get_unique_path(target)
        } else {
            target.to_path_buf()
        };
        
        // 移动文件
        fs::rename(source, &final_target)?;
        
        Ok(())
    }
    
    /// 获取唯一路径（避免覆盖）
    fn get_unique_path(&self, path: &Path) -> PathBuf {
        let stem = path.file_stem().unwrap_or_default().to_string_lossy();
        let ext = path.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
        let parent = path.parent().unwrap_or(Path::new("."));
        
        let mut counter = 1;
        loop {
            let new_name = format!("{} ({}){}", stem, counter, ext);
            let new_path = parent.join(&new_name);
            if !new_path.exists() {
                return new_path;
            }
            counter += 1;
        }
    }
    
    // ═══════════════════════════════════════════════════════════════════
    // 高级文件夹操作
    // ═══════════════════════════════════════════════════════════════════
    
    /// 合并多个文件夹到目标文件夹
    pub fn merge_folders(&self, request: &MergeRequest) -> Result<MergeReport, std::io::Error> {
        let mut report = MergeReport {
            total_files: 0,
            moved: 0,
            skipped: 0,
            conflicts: 0,
            errors: Vec::new(),
        };
        
        // 确保目标目录存在
        fs::create_dir_all(&request.target)?;
        
        for source in &request.sources {
            if !source.is_dir() {
                report.errors.push(format!("不是目录: {}", source.display()));
                continue;
            }
            
            // 遍历源目录中的所有文件
            for entry in fs::read_dir(source)? {
                let entry = entry?;
                let file_name = entry.file_name();
                let target_path = request.target.join(&file_name);
                
                report.total_files += 1;
                
                if target_path.exists() {
                    report.conflicts += 1;
                    
                    match request.conflict_strategy {
                        ConflictStrategy::Skip => {
                            report.skipped += 1;
                            continue;
                        }
                        ConflictStrategy::Overwrite => {
                            if target_path.is_dir() {
                                fs::remove_dir_all(&target_path)?;
                            } else {
                                fs::remove_file(&target_path)?;
                            }
                        }
                        ConflictStrategy::Rename | ConflictStrategy::KeepBoth => {
                            let unique_path = self.get_unique_path(&target_path);
                            fs::rename(entry.path(), unique_path)?;
                            report.moved += 1;
                            continue;
                        }
                    }
                }
                
                // 移动文件/目录
                fs::rename(entry.path(), &target_path)?;
                report.moved += 1;
            }
            
            // 如果设置了删除源文件夹
            if request.delete_sources {
                if let Err(e) = fs::remove_dir(source) {
                    report.errors.push(format!("无法删除源目录 {}: {}", source.display(), e));
                }
            }
        }
        
        Ok(report)
    }
    
    /// 重命名文件夹
    pub fn rename_folder(&self, source: &Path, new_name: &str) -> Result<PathBuf, std::io::Error> {
        let parent = source.parent().unwrap_or(Path::new("."));
        let target = parent.join(new_name);
        
        if target.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                format!("目标已存在: {}", target.display())
            ));
        }
        
        fs::rename(source, &target)?;
        Ok(target)
    }
    
    /// 批量重命名文件
    pub fn batch_rename(&self, folder: &Path, rule: &RenameRule) -> Result<RenameReport, std::io::Error> {
        let mut report = RenameReport {
            total: 0,
            renamed: 0,
            skipped: 0,
            errors: Vec::new(),
        };
        
        let pattern = match Regex::new(&rule.pattern) {
            Ok(p) => p,
            Err(e) => return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("无效的正则表达式: {}", e)
            )),
        };
        
        let mut sequence = rule.sequence_start;
        
        for entry in fs::read_dir(folder)? {
            let entry = entry?;
            let file_name = entry.file_name().to_string_lossy().to_string();
            
            report.total += 1;
            
            if pattern.is_match(&file_name) {
                let mut new_name = pattern.replace(&file_name, &rule.replacement).to_string();
                
                // 添加序号
                if rule.add_sequence {
                    let stem = Path::new(&new_name).file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or(new_name.clone());
                    let ext = Path::new(&new_name).extension()
                        .map(|e| format!(".{}", e.to_string_lossy()))
                        .unwrap_or_default();
                    new_name = format!("{}_{:04}{}", stem, sequence, ext);
                    sequence += 1;
                }
                
                let source_path = entry.path();
                let target_path = folder.join(&new_name);
                
                if target_path.exists() {
                    report.skipped += 1;
                    report.errors.push(format!("目标已存在: {}", new_name));
                    continue;
                }
                
                match fs::rename(&source_path, &target_path) {
                    Ok(_) => report.renamed += 1,
                    Err(e) => {
                        report.skipped += 1;
                        report.errors.push(format!("{}: {}", file_name, e));
                    }
                }
            } else {
                report.skipped += 1;
            }
        }
        
        Ok(report)
    }
    
    // ═══════════════════════════════════════════════════════════════════
    // 🔥 Fate 智能重命名 - 基于分析自动重命名所有文件
    // ═══════════════════════════════════════════════════════════════════
    
    /// 🔥 基于 Fate 分析智能重命名所有文件
    pub fn smart_rename_all(&self, folder: &Path, recursive: bool, dry_run: bool) -> Result<SmartRenameReport, std::io::Error> {
        let mut report = SmartRenameReport {
            total: 0,
            renamed: 0,
            skipped: 0,
            unchanged: 0,
            operations: Vec::new(),
            errors: Vec::new(),
        };
        
        // 收集所有文件
        let mut files_to_rename = Vec::new();
        self.collect_files_for_rename(folder, &mut files_to_rename, recursive)?;
        
        // 按类别分组计数，用于生成序号
        let mut category_counters: HashMap<String, usize> = HashMap::new();
        
        for file_path in files_to_rename {
            report.total += 1;
            
            let metadata = match fs::metadata(&file_path) {
                Ok(m) => m,
                Err(e) => {
                    report.errors.push(format!("{}: {}", file_path.display(), e));
                    report.skipped += 1;
                    continue;
                }
            };
            
            // 提取特征
            let signal = self.feature_extractor.extract(&file_path, metadata.len());
            
            // 生成新文件名
            let new_name = self.generate_smart_name(&signal, &mut category_counters);
            let old_name = signal.name.clone();
            
            // 如果名字相同，跳过
            if new_name == old_name {
                report.unchanged += 1;
                continue;
            }
            
            let parent = file_path.parent().unwrap_or(Path::new("."));
            let new_path = parent.join(&new_name);
            
            let op = SmartRenameOp {
                old_name: old_name.clone(),
                new_name: new_name.clone(),
                reason: self.get_rename_reason(&signal),
                success: false,
            };
            
            if dry_run {
                report.operations.push(SmartRenameOp { success: true, ..op });
                report.renamed += 1;
            } else {
                // 确保不覆盖
                let final_path = if new_path.exists() {
                    self.get_unique_path(&new_path)
                } else {
                    new_path
                };
                
                match fs::rename(&file_path, &final_path) {
                    Ok(_) => {
                        report.operations.push(SmartRenameOp { 
                            new_name: final_path.file_name().unwrap_or_default().to_string_lossy().to_string(),
                            success: true, 
                            ..op 
                        });
                        report.renamed += 1;
                    }
                    Err(e) => {
                        report.errors.push(format!("{} -> {}: {}", old_name, new_name, e));
                        report.operations.push(op);
                        report.skipped += 1;
                    }
                }
            }
        }
        
        Ok(report)
    }
    
    fn collect_files_for_rename(&self, dir: &Path, files: &mut Vec<PathBuf>, recursive: bool) -> Result<(), std::io::Error> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                if recursive {
                    self.collect_files_for_rename(&path, files, recursive)?;
                }
            } else {
                files.push(path);
            }
        }
        Ok(())
    }
    
    fn generate_smart_name(&self, signal: &FileSignal, counters: &mut HashMap<String, usize>) -> String {
        let ext = signal.extension.as_ref()
            .map(|e| format!(".{}", e))
            .unwrap_or_default();
        
        // 基于类别和特征生成前缀
        let prefix = match &signal.features.category {
            FileCategory::AIGeneratedImage => {
                match &signal.features.source_tag {
                    Some(source) => format!("{}", source),
                    None => "AI_Image".to_string(),
                }
            }
            FileCategory::Screenshot => "Screenshot".to_string(),
            FileCategory::Photo => "Photo".to_string(),
            FileCategory::Video => "Video".to_string(),
            FileCategory::Audio => "Audio".to_string(),
            FileCategory::Document => "Doc".to_string(),
            FileCategory::Archive => "Archive".to_string(),
            FileCategory::Application => "App".to_string(),
            FileCategory::Download => "Download".to_string(),
            FileCategory::Unknown => {
                // 尝试从扩展名推断
                match signal.extension.as_deref() {
                    Some("rs") | Some("py") | Some("js") | Some("ts") | Some("c") | Some("cpp") | Some("java") => "Code".to_string(),
                    Some("html") | Some("css") | Some("scss") => "Web".to_string(),
                    Some("json") | Some("yaml") | Some("toml") | Some("xml") => "Config".to_string(),
                    _ => "File".to_string(),
                }
            }
        };
        
        // 获取日期部分
        let date_part = if let Some(date) = &signal.features.date_pattern {
            format!("_{}", date.replace(['/', '.', ' '], "-"))
        } else {
            String::new()
        };
        
        // 生成序号
        let counter_key = format!("{}{}", prefix, date_part);
        let seq = counters.entry(counter_key.clone()).or_insert(0);
        *seq += 1;
        
        // 生成最终名称
        format!("{}{}_{}{}",
            prefix,
            date_part,
            format!("{:04}", seq),
            ext
        )
    }
    
    fn get_rename_reason(&self, signal: &FileSignal) -> String {
        let mut reasons = Vec::new();
        
        if let Some(source) = &signal.features.source_tag {
            reasons.push(format!("AI来源: {}", source));
        }
        
        reasons.push(format!("类型: {:?}", signal.features.category));
        
        if let Some(date) = &signal.features.date_pattern {
            reasons.push(format!("日期: {}", date));
        }
        
        reasons.join(", ")
    }
    
    /// 创建 Fate 整理目录结构
    pub fn create_fate_organize_structure(&self, base_path: &Path) -> Result<PathBuf, std::io::Error> {
        let fate_dir = base_path.join("fate-organize");
        
        // 创建主目录
        fs::create_dir_all(&fate_dir)?;
        
        // 创建子目录结构
        let subdirs = [
            "AI生成",
            "AI生成/DALL-E",
            "AI生成/ChatGPT",
            "AI生成/Midjourney",
            "图片",
            "图片/照片",
            "图片/截图",
            "视频",
            "音频",
            "文档",
            "压缩包",
            "代码",
            "其他",
            ".preserve",  // 保留区
        ];
        
        for subdir in &subdirs {
            fs::create_dir_all(fate_dir.join(subdir))?;
        }
        
        Ok(fate_dir)
    }
    
    /// 整理到 fate-organize 目录
    pub fn organize_to_fate_dir(&self, source_dir: &Path) -> Result<OrganizationPlan, std::io::Error> {
        // 创建 fate-organize 结构
        let _fate_dir = self.create_fate_organize_structure(source_dir)?;
        
        // 分析源目录
        let mut plan = self.analyze_directory(source_dir)?;
        
        // 更新目标路径为 fate-organize 下的路径
        for (_, suggestion) in &mut plan.file_suggestions {
            suggestion.target_folder = format!("fate-organize/{}", suggestion.target_folder);
        }
        
        Ok(plan)
    }
    
    /// 清理空文件夹
    pub fn cleanup_empty_folders(&self, dir: &Path) -> Result<usize, std::io::Error> {
        let mut removed = 0;
        
        self.cleanup_empty_recursive(dir, &mut removed)?;
        
        Ok(removed)
    }
    
    fn cleanup_empty_recursive(&self, dir: &Path, removed: &mut usize) -> Result<bool, std::io::Error> {
        if !dir.is_dir() {
            return Ok(false);
        }
        
        let mut is_empty = true;
        let mut entries_to_check = Vec::new();
        
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                entries_to_check.push(path);
            } else {
                is_empty = false;
            }
        }
        
        // 递归检查子目录
        for subdir in entries_to_check {
            let subdir_empty = self.cleanup_empty_recursive(&subdir, removed)?;
            if !subdir_empty {
                is_empty = false;
            }
        }
        
        // 如果目录为空，删除它（但不删除根目录）
        if is_empty {
            // 检查是否还有内容（可能在递归中被删除）
            let count = fs::read_dir(dir)?.count();
            if count == 0 {
                fs::remove_dir(dir)?;
                *removed += 1;
                return Ok(true);
            }
        }
        
        Ok(is_empty)
    }
    
    /// 删除文件
    pub fn delete_file(&self, path: &Path) -> Result<(), std::io::Error> {
        if path.is_dir() {
            fs::remove_dir_all(path)
        } else {
            fs::remove_file(path)
        }
    }
    
    /// 批量删除文件
    pub fn batch_delete(&self, paths: &[PathBuf]) -> Result<(usize, Vec<String>), std::io::Error> {
        let mut deleted = 0;
        let mut errors = Vec::new();
        
        for path in paths {
            match self.delete_file(path) {
                Ok(_) => deleted += 1,
                Err(e) => errors.push(format!("{}: {}", path.display(), e)),
            }
        }
        
        Ok((deleted, errors))
    }
    
    /// 扫描目录并生成整理方案
    pub fn analyze_directory(&self, path: &Path) -> Result<OrganizationPlan, std::io::Error> {
        let mut file_signals = Vec::new();
        let mut dir_count = 0;
        let mut category_stats: HashMap<String, usize> = HashMap::new();
        let mut discovered_patterns: HashMap<String, Vec<String>> = HashMap::new();
        
        // 递归扫描
        self.scan_directory(path, &mut file_signals, &mut dir_count)?;
        
        // 统计类别
        for signal in &file_signals {
            let cat = format!("{:?}", signal.features.category);
            *category_stats.entry(cat).or_insert(0) += 1;
            
            // 收集模式
            if let Some(source) = &signal.features.source_tag {
                discovered_patterns
                    .entry(format!("AI来源: {}", source))
                    .or_default()
                    .push(signal.name.clone());
            }
            
            if let Some(date) = &signal.features.date_pattern {
                discovered_patterns
                    .entry(format!("日期模式: {}", date))
                    .or_default()
                    .push(signal.name.clone());
            }
        }
        
        // 生成建议
        let file_suggestions: Vec<_> = file_signals.iter()
            .map(|signal| {
                let suggestion = self.generate_suggestion(signal);
                (signal.clone(), suggestion)
            })
            .collect();
        
        // 生成建议的文件夹结构
        let mut folder_counts: HashMap<String, usize> = HashMap::new();
        for (_, suggestion) in &file_suggestions {
            *folder_counts.entry(suggestion.target_folder.clone()).or_insert(0) += 1;
        }
        
        let suggested_structure: Vec<_> = folder_counts.into_iter()
            .map(|(name, count)| FolderStructure {
                description: format!("包含 {} 个文件", count),
                expected_files: count,
                subcategories: vec![],
                name,
            })
            .collect();
        
        // 转换发现的模式
        let patterns: Vec<_> = discovered_patterns.into_iter()
            .filter(|(_, files)| files.len() >= 2)  // 至少2个文件才算模式
            .map(|(pattern_type, files)| DiscoveredPattern {
                pattern_type: pattern_type.clone(),
                description: format!("发现 {} 个文件匹配此模式", files.len()),
                count: files.len(),
                example_files: files.into_iter().take(5).collect(),
            })
            .collect();
        
        Ok(OrganizationPlan {
            source_dir: path.to_path_buf(),
            scan_time: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            total_files: file_signals.len(),
            total_dirs: dir_count,
            category_stats,
            suggested_structure,
            file_suggestions,
            discovered_patterns: patterns,
        })
    }
    
    fn scan_directory(
        &self,
        path: &Path,
        signals: &mut Vec<FileSignal>,
        dir_count: &mut usize,
    ) -> Result<(), std::io::Error> {
        self.scan_directory_recursive(path, signals, dir_count, false)
    }
    
    fn scan_directory_recursive(
        &self,
        path: &Path,
        signals: &mut Vec<FileSignal>,
        dir_count: &mut usize,
        recursive: bool,
    ) -> Result<(), std::io::Error> {
        if !path.is_dir() {
            return Ok(());
        }
        
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let file_path = entry.path();
            
            if file_path.is_dir() {
                *dir_count += 1;
                // 递归扫描子目录
                if recursive {
                    self.scan_directory_recursive(&file_path, signals, dir_count, recursive)?;
                }
            } else {
                let metadata = entry.metadata()?;
                let signal = self.feature_extractor.extract(&file_path, metadata.len());
                signals.push(signal);
            }
        }
        
        Ok(())
    }
    
    /// 递归分析整个目录树
    pub fn analyze_directory_recursive(&self, path: &Path) -> Result<OrganizationPlan, std::io::Error> {
        let mut file_signals = Vec::new();
        let mut dir_count = 0;
        let mut category_stats: HashMap<String, usize> = HashMap::new();
        let mut discovered_patterns: HashMap<String, Vec<String>> = HashMap::new();
        
        // 递归扫描
        self.scan_directory_recursive(path, &mut file_signals, &mut dir_count, true)?;
        
        // 统计类别
        for signal in &file_signals {
            let cat = format!("{:?}", signal.features.category);
            *category_stats.entry(cat).or_insert(0) += 1;
            
            // 收集模式
            if let Some(source) = &signal.features.source_tag {
                discovered_patterns
                    .entry(format!("AI来源: {}", source))
                    .or_default()
                    .push(signal.name.clone());
            }
            
            if let Some(date) = &signal.features.date_pattern {
                discovered_patterns
                    .entry(format!("日期模式: {}", date))
                    .or_default()
                    .push(signal.name.clone());
            }
        }
        
        // 生成建议
        let file_suggestions: Vec<_> = file_signals.iter()
            .map(|signal| {
                let suggestion = self.generate_suggestion(signal);
                (signal.clone(), suggestion)
            })
            .collect();
        
        // 生成建议的文件夹结构
        let mut folder_counts: HashMap<String, usize> = HashMap::new();
        for (_, suggestion) in &file_suggestions {
            *folder_counts.entry(suggestion.target_folder.clone()).or_insert(0) += 1;
        }
        
        let suggested_structure: Vec<_> = folder_counts.into_iter()
            .map(|(name, count)| FolderStructure {
                description: format!("包含 {} 个文件", count),
                expected_files: count,
                subcategories: vec![],
                name,
            })
            .collect();
        
        // 转换发现的模式
        let patterns: Vec<_> = discovered_patterns.into_iter()
            .filter(|(_, files)| files.len() >= 2)
            .map(|(pattern_type, files)| DiscoveredPattern {
                pattern_type: pattern_type.clone(),
                description: format!("发现 {} 个文件匹配此模式", files.len()),
                count: files.len(),
                example_files: files.into_iter().take(5).collect(),
            })
            .collect();
        
        Ok(OrganizationPlan {
            source_dir: path.to_path_buf(),
            scan_time: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            total_files: file_signals.len(),
            total_dirs: dir_count,
            category_stats,
            suggested_structure,
            file_suggestions,
            discovered_patterns: patterns,
        })
    }
    
    fn generate_suggestion(&self, signal: &FileSignal) -> OrganizationSuggestion {
        let target_folder = self.pattern_db.suggest_folder(signal);
        
        // 根据特征确定置信度
        let confidence = if signal.features.source_tag.is_some() {
            90  // AI 来源明确
        } else if signal.features.category != FileCategory::Unknown {
            75  // 类别明确
        } else {
            50  // 不确定
        };
        
        let reason = match &signal.features.source_tag {
            Some(source) => format!("检测到 {} 生成标识", source),
            None => format!("根据文件类型 {:?} 分类", signal.features.category),
        };
        
        OrganizationSuggestion {
            target_folder,
            action: SuggestedAction::MoveTo(String::new()),
            confidence,
            reason,
            related_files: vec![],
        }
    }
    
    /// 生成整理报告 (Markdown 格式)
    pub fn generate_report(&self, plan: &OrganizationPlan) -> String {
        let mut report = String::new();
        
        report.push_str("# 🌌 Fate Organizer 整理方案\n\n");
        report.push_str(&format!("**扫描目录**: `{}`\n", plan.source_dir.display()));
        report.push_str(&format!("**扫描时间**: {}\n", plan.scan_time));
        report.push_str(&format!("**文件总数**: {} 个文件, {} 个子目录\n\n", plan.total_files, plan.total_dirs));
        
        // 类别统计
        report.push_str("## 📊 文件类型分布\n\n");
        report.push_str("| 类型 | 数量 |\n|------|------|\n");
        let mut sorted_stats: Vec<_> = plan.category_stats.iter().collect();
        sorted_stats.sort_by(|a, b| b.1.cmp(a.1));
        for (cat, count) in sorted_stats {
            report.push_str(&format!("| {} | {} |\n", cat, count));
        }
        report.push_str("\n");
        
        // 发现的模式
        if !plan.discovered_patterns.is_empty() {
            report.push_str("## 🔍 发现的模式\n\n");
            for pattern in &plan.discovered_patterns {
                report.push_str(&format!("### {}\n", pattern.pattern_type));
                report.push_str(&format!("- 匹配文件数: {}\n", pattern.count));
                report.push_str("- 示例:\n");
                for example in &pattern.example_files {
                    report.push_str(&format!("  - `{}`\n", example));
                }
                report.push_str("\n");
            }
        }
        
        // 建议的文件夹结构
        report.push_str("## 📁 建议的文件夹结构\n\n");
        report.push_str("```\n");
        let mut sorted_structure: Vec<_> = plan.suggested_structure.iter().collect();
        sorted_structure.sort_by(|a, b| b.expected_files.cmp(&a.expected_files));
        for folder in sorted_structure {
            report.push_str(&format!("├── {} ({} 个文件)\n", folder.name, folder.expected_files));
        }
        report.push_str("```\n\n");
        
        // 详细文件列表 (按类别分组)
        report.push_str("## 📋 详细整理建议\n\n");
        
        let mut by_folder: HashMap<String, Vec<&(FileSignal, OrganizationSuggestion)>> = HashMap::new();
        for item in &plan.file_suggestions {
            by_folder.entry(item.1.target_folder.clone()).or_default().push(item);
        }
        
        let mut sorted_folders: Vec<_> = by_folder.iter().collect();
        sorted_folders.sort_by(|a, b| b.1.len().cmp(&a.1.len()));
        
        for (folder, files) in sorted_folders.iter().take(10) {  // 只显示前10个分类
            report.push_str(&format!("### 📂 {}\n\n", folder));
            report.push_str(&format!("共 {} 个文件\n\n", files.len()));
            
            // 显示前5个示例
            for (signal, suggestion) in files.iter().take(5) {
                report.push_str(&format!("- `{}` (置信度: {}%)\n", signal.name, suggestion.confidence));
                report.push_str(&format!("  - 理由: {}\n", suggestion.reason));
            }
            
            if files.len() > 5 {
                report.push_str(&format!("  - ... 还有 {} 个文件\n", files.len() - 5));
            }
            report.push_str("\n");
        }
        
        report.push_str("---\n\n");
        report.push_str("*由 Valhalla Fate Organizer 生成*\n");
        
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_feature_extraction() {
        let extractor = FeatureExtractor::new();
        let path = Path::new("DALL·E 2023-12-14 10.32.55 - test image.png");
        let signal = extractor.extract(path, 1024);
        
        assert_eq!(signal.features.source_tag, Some("DALL-E".to_string()));
        assert!(signal.features.date_pattern.is_some());
        assert_eq!(signal.features.category, FileCategory::AIGeneratedImage);
    }
    
    #[test]
    fn test_language_detection() {
        let extractor = FeatureExtractor::new();
        
        assert_eq!(extractor.detect_language("Hello World"), LanguageType::English);
        assert_eq!(extractor.detect_language("你好世界"), LanguageType::Chinese);
        assert_eq!(extractor.detect_language("Hello 你好"), LanguageType::Mixed);
    }
}
