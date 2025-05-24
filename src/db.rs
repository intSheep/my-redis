use std::sync::{Arc, Mutex};

#[derive(Debug,Clone)]
pub(crate) struct Db{
    shared:Arc<Shared>
}

#[derive(Debug)]
struct Shared{

}