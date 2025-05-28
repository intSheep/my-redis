use std::collections::{BTreeSet, HashMap};
use std::os::linux::raw::stat;
use std::sync::{Arc, Mutex};
use std::time::{Duration};
use bytes::Bytes;
use tokio::sync::{broadcast, Notify};
use tokio::time;
use tokio::time::Instant;
use tracing::debug;

#[derive(Debug)]
pub (crate)struct DbDropGuard{
    db:Db,
}

#[derive(Debug,Clone)]
pub(crate) struct Db{
    shared:Arc<Shared>
}

#[derive(Debug)]
struct Shared{
    state:Mutex<State>,
    background_task:Notify,
}

#[derive(Debug)]
struct State{
    entries:HashMap<String,Entry>,
    pub_sub:HashMap<String,broadcast::Sender<Bytes>>,
    expirations:BTreeSet<(Instant,String)>,
    shutdown:bool,
}

#[derive(Debug)]
struct Entry{
    data:Bytes,
    expires_at:Option<Instant>,
}

impl DbDropGuard {
    pub(crate) fn new()->DbDropGuard{
        DbDropGuard{db: Db::new()}
    }

    pub(crate) fn db(&self) -> Db {
        self.db.clone()
    }
}

impl Drop for DbDropGuard{
    fn drop(&mut self) {
       self.db.shutdown_purge_task()
    }
}

impl Db {
    pub(crate) fn new()->Db{
      let shared = Arc::new(Shared{
          state:Mutex::new(State{
                entries:HashMap::new(),
                pub_sub:HashMap::new(),
                expirations:BTreeSet::new(),
                shutdown:false,
          }),
          background_task:Notify::new(),
      });
        tokio::spawn(purge_expired_tasks(shared.clone()));

        Db{shared}
    }

    pub(crate) fn get(&self,key:&str)->Option<Bytes>{
        let state =self.shared.state.lock().unwrap();
        state.entries.get(key).map(|entry|{
            entry.data.clone()
        })
    }


    pub(crate) fn set(&self,key:String,value: Bytes,expire:Option<Duration>){
        let mut notify =false;
        let mut state = self.shared.state.lock().unwrap();

        let expire_at = expire.map(|duration| {
            let when = Instant::now() +duration;

            notify = state
                .next_expiration()
                .map(|expiration| expiration>when)
                .unwrap_or(true);

            when
        });

        let prev = state.entries.insert(
            key.clone(),
            Entry {
                data: value,
                expires_at: expire_at,
            },
        );

        if let Some(when)=expire_at{
            state.expirations.insert((when, key));
        }

        drop(state);

        if notify{
            self.shared.background_task.notify_one()
        }
    }

    pub(crate)fn subscribe(&self,key:String)->broadcast::Receiver<Bytes>{
        use std::collections::hash_map::Entry;

        let mut state = self.shared.state.lock().unwrap();
        
        match state.pub_sub.entry(key) {
            Entry::Occupied(e)=>e.get().subscribe(),
            Entry::Vacant(e)=>{
                let(tx,rx)=broadcast::channel(1024);
                e.insert(tx);
                rx
            }
        }
    }

    pub(crate) fn public(&self,key:&str,value:Bytes)->usize{
        let state = self.shared.state.lock().unwrap();

        state
            .pub_sub
            .get(key)
            // 没发送成功返回0
            .map(|tx|{tx.send(value).unwrap_or(0)})
            // 没有发送者返回0
            .unwrap_or(0)
    }


    fn shutdown_purge_task(&self){
        let mut state=self.shared.state.lock().unwrap();
        state.shutdown = true;

        drop(state);
        self.shared.background_task.notify_one()
    }
}

impl Shared{
    fn purge_expired_keys(&self) -> Option<Instant>{
        let mut state = self.state.lock().unwrap();

        if state.shutdown{
            return None;
        }
        // 太变态了
        // 将智能指针解引用然后增加mut，让其变成可变引用
        // 不然整个智能指针在被iter借用后，后续不能再借用了
        let state = &mut *state;
        let now = Instant::now();

       while let Some( &(when, ref key))=state.expirations.iter().next(){
            if when>now{
                return Some(when);
            }
           state.entries.remove(key);
           state.expirations.remove(&(when, key.clone()));
        }
        None
    }

    fn is_shutdown(&self)->bool{
        self.state.lock().unwrap().shutdown
    }
}

impl State{
    fn next_expiration(&self)->Option<Instant>{
        self.expirations.iter().next().map(|expiration|expiration.0)
    }
}

async fn purge_expired_tasks(shared: Arc<Shared>) {
    // If the shutdown flag is set, then the task should exit.
    while !shared.is_shutdown() {
        // Purge all keys that are expired. The function returns the instant at
        // which the **next** key will expire. The worker should wait until the
        // instant has passed then purge again.
        if let Some(when) = shared.purge_expired_keys() {
            // Wait until the next key expires **or** until the background task
            // is notified. If the task is notified, then it must reload its
            // state as new keys have been set to expire early. This is done by
            // looping.
            tokio::select! {
                _ = time::sleep_until(when) => {}
                _ = shared.background_task.notified() => {}
            }
        } else {
            // There are no keys expiring in the future. Wait until the task is
            // notified.
            shared.background_task.notified().await;
        }
    }

    debug!("Purge background task shut down")
}
