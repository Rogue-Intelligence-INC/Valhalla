//! Valhalla 系统存储
//! 
//! 简单的目录+文件操作权限
//! 每个系统有自己的目录，想怎么改就怎么改

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write, BufReader, BufWriter};
use std::path::{Path, PathBuf};
use serde::{Serialize, de::DeserializeOwned};

/// 系统存储 - 每个系统一个实例
#[derive(Debug, Clone)]
pub struct SystemStorage {
    /// 系统根目录
    pub root: PathBuf,
    /// 系统名称
    pub name: String,
}

impl SystemStorage {
    /// 创建系统存储（自动创建目录）
    pub fn new(base_path: &Path, system_name: &str) -> io::Result<Self> {
        let root = base_path.join(system_name);
        fs::create_dir_all(&root)?;
        
        Ok(Self {
            root,
            name: system_name.to_string(),
        })
    }
    
    /// 获取完整路径
    pub fn path(&self, relative: &str) -> PathBuf {
        self.root.join(relative)
    }
    
    // ========== 基础文件操作 ==========
    
    /// 创建/写入文件（覆盖）
    pub fn write(&self, relative: &str, content: &str) -> io::Result<()> {
        let path = self.path(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, content)
    }
    
    /// 追加写入
    pub fn append(&self, relative: &str, content: &str) -> io::Result<()> {
        let path = self.path(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        file.write_all(content.as_bytes())
    }
    
    /// 读取文件
    pub fn read(&self, relative: &str) -> io::Result<String> {
        fs::read_to_string(self.path(relative))
    }
    
    /// 读取为字节
    pub fn read_bytes(&self, relative: &str) -> io::Result<Vec<u8>> {
        fs::read(self.path(relative))
    }
    
    /// 写入字节
    pub fn write_bytes(&self, relative: &str, content: &[u8]) -> io::Result<()> {
        let path = self.path(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, content)
    }
    
    /// 删除文件
    pub fn delete(&self, relative: &str) -> io::Result<()> {
        let path = self.path(relative);
        if path.is_file() {
            fs::remove_file(&path)
        } else if path.is_dir() {
            fs::remove_dir_all(&path)
        } else {
            Ok(()) // 不存在就算成功
        }
    }
    
    /// 文件是否存在
    pub fn exists(&self, relative: &str) -> bool {
        self.path(relative).exists()
    }
    
    /// 是否是文件
    pub fn is_file(&self, relative: &str) -> bool {
        self.path(relative).is_file()
    }
    
    /// 是否是目录
    pub fn is_dir(&self, relative: &str) -> bool {
        self.path(relative).is_dir()
    }
    
    // ========== 目录操作 ==========
    
    /// 创建目录
    pub fn mkdir(&self, relative: &str) -> io::Result<()> {
        fs::create_dir_all(self.path(relative))
    }
    
    /// 列出目录内容
    pub fn list(&self, relative: &str) -> io::Result<Vec<String>> {
        let path = self.path(relative);
        let mut items = Vec::new();
        
        if path.is_dir() {
            for entry in fs::read_dir(&path)? {
                let entry = entry?;
                if let Some(name) = entry.file_name().to_str() {
                    items.push(name.to_string());
                }
            }
        }
        
        Ok(items)
    }
    
    /// 列出所有文件（递归）
    pub fn list_all(&self) -> io::Result<Vec<String>> {
        self.list_recursive("")
    }
    
    fn list_recursive(&self, relative: &str) -> io::Result<Vec<String>> {
        let path = self.path(relative);
        let mut items = Vec::new();
        
        if path.is_dir() {
            for entry in fs::read_dir(&path)? {
                let entry = entry?;
                let name = entry.file_name().to_string_lossy().to_string();
                let child_path = if relative.is_empty() {
                    name.clone()
                } else {
                    format!("{}/{}", relative, name)
                };
                
                if entry.path().is_dir() {
                    items.extend(self.list_recursive(&child_path)?);
                } else {
                    items.push(child_path);
                }
            }
        }
        
        Ok(items)
    }
    
    // ========== 重组操作 ==========
    
    /// 移动/重命名
    pub fn rename(&self, from: &str, to: &str) -> io::Result<()> {
        let from_path = self.path(from);
        let to_path = self.path(to);
        
        if let Some(parent) = to_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        fs::rename(from_path, to_path)
    }
    
    /// 复制文件
    pub fn copy(&self, from: &str, to: &str) -> io::Result<()> {
        let from_path = self.path(from);
        let to_path = self.path(to);
        
        if let Some(parent) = to_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        fs::copy(from_path, to_path)?;
        Ok(())
    }
    
    /// 合并多个文件内容到一个文件
    pub fn merge(&self, sources: &[&str], target: &str) -> io::Result<()> {
        let mut content = String::new();
        for src in sources {
            if self.exists(src) {
                content.push_str(&self.read(src)?);
                content.push('\n');
            }
        }
        self.write(target, &content)
    }
    
    // ========== JSON 序列化 ==========
    
    /// 保存 JSON
    pub fn save_json<T: Serialize>(&self, relative: &str, data: &T) -> io::Result<()> {
        let json = serde_json::to_string_pretty(data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        self.write(relative, &json)
    }
    
    /// 加载 JSON
    pub fn load_json<T: DeserializeOwned>(&self, relative: &str) -> io::Result<T> {
        let content = self.read(relative)?;
        serde_json::from_str(&content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
    
    // ========== 统计 ==========
    
    /// 获取文件大小
    pub fn size(&self, relative: &str) -> io::Result<u64> {
        Ok(fs::metadata(self.path(relative))?.len())
    }
    
    /// 获取目录总大小
    pub fn total_size(&self) -> io::Result<u64> {
        self.dir_size(&self.root)
    }
    
    fn dir_size(&self, path: &Path) -> io::Result<u64> {
        let mut total = 0;
        if path.is_dir() {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let p = entry.path();
                if p.is_dir() {
                    total += self.dir_size(&p)?;
                } else {
                    total += fs::metadata(&p)?.len();
                }
            }
        }
        Ok(total)
    }
    
    /// 文件数量
    pub fn file_count(&self) -> io::Result<usize> {
        Ok(self.list_all()?.len())
    }
    
    // ========== 清理 ==========
    
    /// 清空整个存储
    pub fn clear(&self) -> io::Result<()> {
        if self.root.exists() {
            fs::remove_dir_all(&self.root)?;
            fs::create_dir_all(&self.root)?;
        }
        Ok(())
    }
}

/// 创建所有系统的存储目录
pub fn init_all_storages(base_path: &Path) -> io::Result<AllStorages> {
    Ok(AllStorages {
        blackhole: SystemStorage::new(base_path, "blackhole")?,
        meshbrain: SystemStorage::new(base_path, "meshbrain")?,
        multinova: SystemStorage::new(base_path, "multinova")?,
        baseforce: SystemStorage::new(base_path, "baseforce")?,
        stemcell: SystemStorage::new(base_path, "stemcell")?,
        tile: SystemStorage::new(base_path, "tile")?,
        chamber: SystemStorage::new(base_path, "chamber")?,
    })
}

/// 所有系统存储
#[derive(Debug, Clone)]
pub struct AllStorages {
    pub blackhole: SystemStorage,
    pub meshbrain: SystemStorage,
    pub multinova: SystemStorage,
    pub baseforce: SystemStorage,
    pub stemcell: SystemStorage,
    pub tile: SystemStorage,
    pub chamber: SystemStorage,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;
    
    #[test]
    fn test_basic_operations() {
        let base = temp_dir().join("valhalla_test");
        let storage = SystemStorage::new(&base, "test_system").unwrap();
        
        // 写入
        storage.write("test.txt", "Hello World").unwrap();
        assert!(storage.exists("test.txt"));
        
        // 读取
        let content = storage.read("test.txt").unwrap();
        assert_eq!(content, "Hello World");
        
        // 追加
        storage.append("test.txt", "\nMore content").unwrap();
        let content = storage.read("test.txt").unwrap();
        assert!(content.contains("More content"));
        
        // 删除
        storage.delete("test.txt").unwrap();
        assert!(!storage.exists("test.txt"));
        
        // 清理
        storage.clear().unwrap();
    }
    
    #[test]
    fn test_directory_operations() {
        let base = temp_dir().join("valhalla_test2");
        let storage = SystemStorage::new(&base, "test_system").unwrap();
        
        // 创建子目录和文件
        storage.write("sub/a.txt", "A").unwrap();
        storage.write("sub/b.txt", "B").unwrap();
        storage.write("sub/deep/c.txt", "C").unwrap();
        
        // 列出
        let items = storage.list("sub").unwrap();
        assert!(items.contains(&"a.txt".to_string()));
        assert!(items.contains(&"b.txt".to_string()));
        
        // 递归列出
        let all = storage.list_all().unwrap();
        assert!(all.len() >= 3);
        
        // 清理
        storage.clear().unwrap();
    }
}
