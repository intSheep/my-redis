use std::collections::{BTreeSet, HashMap};
use std::os::linux::raw::stat;
use std::sync::{Arc, Mutex};
use bytes::Bytes;
use tokio::sync::{broadcast, Notify};
use tokio::time::Instant;

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

impl Drop for DbDropGuard{
    fn drop(&mut self) {
        // TODO
    }
}

impl Db {
    //TODO
}

impl Shared{
    fn purge_expired_key(&self) -> Option<Instant>{
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