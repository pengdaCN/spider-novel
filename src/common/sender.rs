use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::{OwnedPermit, Sender};
use tokio::sync::{Mutex, Notify};

pub struct WrapSender<T> {
    notify: Arc<Notify>,
    sender: Sender<T>,
    tx_seq: Arc<Mutex<usize>>,
    permit_seq: AtomicUsize,
}

impl<T> WrapSender<T> {
    pub fn wrap(tx: Sender<T>) -> Self {
        Self {
            notify: Arc::default(),
            sender: tx,
            tx_seq: Arc::default(),
            permit_seq: AtomicUsize::default(),
        }
    }

    pub async fn permit_owned(&self) -> Result<OrderPermit<T>, SendError<()>> {
        let seq = self.permit_seq.fetch_add(1, Ordering::Relaxed);
        let permit = self.sender.clone().reserve_owned().await?;

        Ok(OrderPermit {
            seq,
            tx: self.tx_seq.clone(),
            permit,
            notify: self.notify.clone(),
        })
    }
}

pub struct OrderPermit<T> {
    seq: usize,
    tx: Arc<Mutex<usize>>,
    permit: OwnedPermit<T>,
    notify: Arc<Notify>,
}

impl<T> OrderPermit<T> {
    pub async fn send(self, val: T) {
        loop {
            let mut x = self.tx.lock().await;
            let tx_seq = *x;

            // 判断发送的序列号是否等于当前的序列号
            if tx_seq == self.seq {
                // 发送数据
                self.permit.send(val);

                // 更新下一个序列号
                *x = tx_seq + 1;

                // 释放锁
                drop(x);

                // 通知其他任务
                self.notify.notify_waiters();

                return;
            } else {
                // 若不等于则释放锁，然后等待唤醒
                drop(x);

                self.notify.notified().await;
            }
        }
    }
}
