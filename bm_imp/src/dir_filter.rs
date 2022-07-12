use std::collections::VecDeque;
use std::ffi::OsString;

use regex::Regex;
use tokio::fs::{self, DirEntry};
use tokio::stream::StreamExt;

use deadqueue::unlimited::Queue;

#[derive(Debug)]
pub struct FileItem {
    pub index: u32,
    pub file_name: OsString,
}

pub struct DirWalkFilter {
    /// 文件大小限制
    pub size: u64,

    /// 文件扩展名
    pub ext: Vec<String>,

    /// 目录
    pub dir: String,

    /// 文件名正则要求
    pub regex: Regex,

    /// 目录下文件个数（包含不符合要求的文件）
    pub count: u64,

    /// 符合要求的文件名集合
    pub targets: Queue<FileItem>,

    // 存储一些样例
    pub good_samples: VecDeque<OsString>,
    pub bad_samples: VecDeque<OsString>,

}

impl DirWalkFilter {
    pub fn new(dir: &str, ext: Vec<String>, regex: Regex) -> Self {
        DirWalkFilter {
            size: 0,
            ext,
            dir: dir.to_string(),
            regex,
            count: 0,
            targets: Queue::new(),
            good_samples: VecDeque::new(),
            bad_samples: VecDeque::new(),
        }
    }

    /// 检查后缀
    fn check_extension(&self, entry: &DirEntry) -> bool {

        // 后缀不限制
        if self.ext.is_empty() {
            return true;
        }

        let path = entry.path();
        let ext = match path.extension() {
            Some(v) => v,
            None => {
                return false;
            }
        };
        let ext = match ext.to_str() {
            Some(v) => v,
            None => {
                return false;
            }
        };

        // self.ext.iter().fold(false, |a, b| a || ext.eq_ignore_ascii_case(b))
        self.ext.iter().any(|b| ext.eq_ignore_ascii_case(b))
    }

    /// 检查文件名是否符合正则
    fn check_filestem(&self, entry: &DirEntry) -> bool {
        let path = entry.path();
        let name = match path.file_stem() {
            Some(v) => v,
            None => {
                return false;
            }
        };
        let name = match name.to_str() {
            Some(v) => v,
            None => {
                return false;
            }
        };

        self.regex.is_match(name)
    }

    pub async fn list_dir(&mut self) -> std::io::Result<()> {
        self.count = 0;

        self.good_samples.clear();
        self.bad_samples.clear();

        let sample_count = 10;
        let mut gs_full = false;
        let mut bs_full = false;
        let mut index = 0_u32;

        let mut entry = fs::read_dir(&self.dir).await?;

        while let Some(item) = entry.next().await {
            let item = item?;
            let meta = item.metadata().await?;

            if meta.is_file() {
                self.count += 1;

                //检查文件大小, 后缀，正则
                if meta.len() > self.size
                    && self.check_extension(&item)
                    && self.check_filestem(&item) {
                    self.targets.push(FileItem {
                        index,
                        file_name: item.file_name(),
                    });
                    index += 1;

                    if !gs_full {
                        self.good_samples.push_front(item.file_name());
                        if self.good_samples.len() == sample_count {
                            gs_full = true;
                        }
                    }
                } else if !bs_full {
                    self.bad_samples.push_front(item.file_name());
                    if self.bad_samples.len() == sample_count {
                        bs_full = true;
                    }
                }
            }
        }

        Ok(())
    }
}
