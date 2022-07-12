/// serial pool ：多线程调度者， 提交 event + holder 给 pool处理
/// holder      : 需要串行处理的业务对象，包含数据以及事件队列
/// handler     ：业务逻辑处理者，对holder的进行处理
/// event       : 针对holder的事件
///
/// 对holder的操作，转化成事件队列（串行）
/// 具体的处理逻辑，由handler进行处理
///
/// producer -> event queue -> pool.dispatch( holder , event ) -> handler.process(holder, events)
/// 1）针对某个holder的处理，是串行单线程的，同一个时间没有多线程同时处理这个holder。
/// 将event加入到holder的事件队列中，如果holder在不在运行中，则启动运行（放入到pool中运行）
/// 2) holder的数量可以很多，pool线程的数量是少量或固定的

use std::sync::{Mutex, Arc};
use std::marker::PhantomData;

pub struct HolderState<E: Send>
{
    pub running: bool,
    pub events: Vec<E>,
}

pub struct Holder<T: Send, E: Send>
{
    pub data: Arc<Mutex<T>>,
    pub state: Mutex<HolderState<E>>,
}

pub trait Handler<T: Send, E: Send>: Send + Sync + 'static
{
    fn process(&self, holder_data: Arc<Mutex<T>>, events: Vec<E>);
}

pub struct SerialPool<H, T, E>
    where H: Handler<T, E>,
          T: Send + 'static,
          E: Send + 'static,
{
    pub handler: Arc<H>,
    _t: PhantomData<(T, E)>,
}

//------------------------------
impl<T, E> Holder<T, E>
    where T: Send + 'static,
          E: Send + 'static,
{
    pub fn new(data: T) -> Holder<T, E> {
        Holder {
            data: Arc::new(Mutex::new(data)),
            state: Mutex::new(HolderState {
                running: false,
                events: Vec::<E>::new(),
            }),
        }
    }
}

impl<H, T, E> SerialPool<H, T, E>
    where H: Handler<T, E>,
          T: Send + 'static,
          E: Send + 'static,
{
    pub fn new(h: H) -> SerialPool<H, T, E> {
        SerialPool {
            handler: Arc::new(h),
            _t: PhantomData,
        }
    }

    /// 将holder中的events处理完
    async fn drain_event(handler: Arc<H>, holder: Arc<Holder<T, E>>)
    {
        // println!("enter drain_event .");
        loop {
            let mut guard = holder.state.lock().unwrap();
            let size = guard.events.len();
            // println!("events.len: {}", size);
            if size == 0 {
                guard.running = false;
                break;
            }

            let events_copy: Vec<E> = guard.events.drain(..).collect();

            drop(guard);

            let data = holder.data.clone();
            handler.process(data, events_copy);
        }
    }

    /// 分发/处理 事件
    /// dispatch 密集调用时候，导致 drain_event 没机会运行,
    /// dispatch调用方可以 tokio::task::yield_now().await
    pub fn dispatch(&self, holder: Arc<Holder<T, E>>, event: E) {
        let mut guard = holder.state.lock().unwrap();
        guard.events.push(event);

        // if not running
        if !guard.running {
            let ha = self.handler.clone();
            let ho = holder.clone();

            // println!("will spawn ...");

            tokio::spawn(async move {
                // println!("will drain_event .");
                Self::drain_event(ha, ho).await;
            });
            guard.running = true;
        }
    }
}