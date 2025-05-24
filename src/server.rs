use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::time;
use tracing::{debug, error, info, instrument};


struct Listener{
    listener: TcpListener,
    limit_connections:Arc<Semaphore>
}

struct Handler{

}

impl Listener {

    async fn run(&mut self)->crate::Result<()>{
        info!("accepting inbound connections");

        loop {

            let permit = self.
                limit_connections.
                clone().
                acquire_owned().
                await.
                unwrap();
            let socket = self.accept().await;
            let handler = Handler{};
            // spawn a new task to handle the connection
        }
        Ok(())
    }

    async fn accept(&mut self)->crate::Result<(TcpStream)>{
        let mut backoff =1;
        loop {
            match self.listener.accept() {
                Ok((socket,_))=>return Ok(socket),
                Err(err)=>{
                    if backoff > 64{
                        return Err(err.into());
                    }
                }
            }
            time::sleep(Duration::from_secs(backoff)).await;
            backoff *=2;
        }
    }
}