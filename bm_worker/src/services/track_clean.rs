use crate::app_ctx::AppCtx;
use std::sync::Arc;
use std::time::Duration;
use super::Service;

use tokio::task::{JoinHandle as TkJoinHandle};
use tokio::sync::watch::{Receiver};
use tokio::stream::StreamExt;
use tokio::time;

use log::{debug, warn, error, info};
use cffc_base::util::utils;
use cffc_base::model::img_file;
use crate::error::AppResult;


pub struct TrackCleanSvc {
    ctx: Arc<AppCtx>,
    dur: Duration,
    times: i64,
}

impl TrackCleanSvc {
    pub fn new(ctx: Arc<AppCtx>) -> Self {
        let dur = Duration::from_secs(ctx.cfg.disk_clean.interval_minute as u64 * 60);
        TrackCleanSvc {
            ctx,
            dur,
            times: 0,
        }
    }

    /// 取出最旧的N条记录，（id,ft_sid)
    /// 删除 id <= x，删除文件系统中 ft_sid对应的目录
    async fn do_clean_ft(&mut self) -> AppResult<()> {
        let limit = self.ctx.cfg.disk_clean.clean_ft_batch as i64;
        let ctx = self.ctx.clone();
        let df_imgs = self.ctx.cfg.df_imgs.as_str();

        let ids = tokio::task::spawn_blocking(move || {
            ctx.dao.load_eldest_ft(limit)
        }).await??;

        if ids.is_empty() {
            warn!("warn, TrackCleanSvc, do clean, but facetracks not found");
            return Ok(());
        }

        let mut max_id: i64 = 0;

        for v in ids.iter() {
            if v.0 > max_id {
                max_id = v.0;
            }

            let dir = img_file::get_facetrack_imgdir(df_imgs, v.1.as_str());
            match tokio::fs::remove_dir_all(&dir).await {
                Ok(_) => {
                    debug!("TrackCleanSvc, remove ft dir ok, {:?}", dir);
                }
                Err(e) => {
                    error!("error, TrackCleanSvc remove ft dir:{:?}, {:?}", dir, e);
                }
            }
        }
        // 删除数据库记录
        let ctx = self.ctx.clone();
        let affect = tokio::task::spawn_blocking(move || {
            ctx.dao.delete_eldest_ft(max_id)
        }).await??;
        info!("TrackCleanSvc, delete {} facetracks", affect);

        Ok(())
    }

    /// 取出最旧的N条记录，（id,ct_sid)
    /// 删除 id <= x，删除文件系统中 ct_sid对应的目录
    async fn do_clean_ct(&mut self) -> AppResult<()> {
        let limit = self.ctx.cfg.disk_clean.clean_ct_batch as i64;
        let ctx = self.ctx.clone();
        let df_imgs = self.ctx.cfg.df_imgs.as_str();

        let ids = tokio::task::spawn_blocking(move || {
            ctx.dao.load_eldest_ct(limit)
        }).await??;

        if ids.is_empty() {
            warn!("warn, TrackCleanSvc, do clean, but cartracks not found");
            return Ok(());
        }

        let mut max_id: i64 = 0;

        for v in ids.iter() {
            if v.0 > max_id {
                max_id = v.0;
            }

            let dir = img_file::get_cartrack_imgdir(df_imgs, v.1.as_str());
            match tokio::fs::remove_dir_all(&dir).await {
                Ok(_) => {
                    debug!("TrackCleanSvc, remove ct dir ok, {:?}", dir);
                }
                Err(e) => {
                    error!("error, TrackCleanSvc remove ct dir:{:?}, {:?}", dir, e);
                }
            }
        }
        // 删除数据库记录
        let ctx = self.ctx.clone();
        let affect = tokio::task::spawn_blocking(move || {
            ctx.dao.delete_eldest_ct(max_id)
        }).await??;
        info!("TrackCleanSvc, delete {} cartracks", affect);

        Ok(())
    }

    /// 检查图片目录所在分区，剩余磁盘空间
    /// 少于指定值，删除track记录和图片文件
    async fn do_work(&mut self) {
        debug!("TrackCleanSvc do work. times:{}", self.times);
        let dir = self.ctx.cfg.df_imgs.as_str();
        let available_size = match utils::get_disk_available(dir) {
            Ok(v) => v,
            Err(e) => {
                error!("error, get_disk_available, {}, {:?}", dir, e);
                return;
            }
        };

        if available_size < self.ctx.cfg.disk_clean.avail_size_m * 1024 * 1024 {
            // 空间不够
            debug!("TrackCleanSvc, do clean, available_size: {}", available_size);
            if let Err(e) = self.do_clean_ft().await {
                debug!("error, TrackCleanSvc, do ft clean, {:?}", e);
            } else {
                debug!("TrackCleanSvc, do ft clean, ok");
            }

            if let Err(e) = self.do_clean_ct().await {
                debug!("error, TrackCleanSvc, do ct clean, {:?}", e);
            } else {
                debug!("TrackCleanSvc, do ct clean, ok");
            }
        } else {
            debug!("TrackCleanSvc, skip it, available_size: {}", available_size);
        }

        self.times += 1;
    }
}

impl Service for TrackCleanSvc {
    fn run(self, rx: Receiver<i64>) -> TkJoinHandle<()> {
        let mut interval = time::interval(self.dur);
        let mut svc = self;
        let mut exit_rx = rx;

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                            info!("TrackCleanSvc wakup, do homework ...");
                            svc.do_work().await;
                    }
                    quit = exit_rx.next() => {
                        if let Some(100) = quit {
                            info!("TrackCleanSvc recv exit");
                            break;
                        }
                    }
                }
            }
            info!("TrackCleanSvc exit.");
        })
    }
}


