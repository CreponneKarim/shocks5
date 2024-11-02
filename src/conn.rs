use std::{os::unix::net::SocketAddr, thread::JoinHandle};

use anyhow::Error;
use tokio::{io::{AsyncReadExt, BufReader}, net::TcpStream, sync::watch::error::RecvError};
use tracing::{info, warn, error};

// pub enum DisplayMessages {
//     Error(ErrorMessages),
//     WarningMessages(WarningMessages),
//     OkMessages(OkMessages)
// }

// pub enum ErrorMessages {

// }
// pub enum WarningMessages {
//     UnreadableStream{message : }
// }
// pub enum OkMessages {

// }

pub struct Conn {
    conn: TcpStream,
}

impl Conn {
    pub const BUFF_NB_BYTES: usize = 1024;

    pub async fn handle(conn: TcpStream) -> Result<Self,anyhow::Error>{
        let conn_addr = conn.peer_addr().unwrap();
        let mut connection_obj = Conn {
            conn:conn
        };
        match connection_obj.conn.readable().await {
            Ok(()) => {
                loop {
                    let mut buff = [0; Self::BUFF_NB_BYTES];
                    match connection_obj.conn.read(&mut buff).await {
                        Ok(0) => {
                            info!("EOF reached on {}",conn_addr);
                            break;
                        },
                        Ok(n) => {
                            //  do smth with buffer
                            if n <1024 {
                                //  break since its last loop
                                break;
                            }
                        },
                        Err(e) => {
                            //  capture error and print it
                            warn!("encountered error while reading from connection : {} error :{}", conn_addr, e)
                        }
                    }
                }
            },
            Err(e) => {
                warn!("unable to read from connection : {} error: {}", conn_addr,e);
            }
        }

        Ok(connection_obj)
    }
}

/// struct that listens to incoming connections and offloads them to tokio tasks
pub struct Server<T> where
    T: Result
{
    addr: String,
    task_list: Vec<JoinHandle<T>>
}

impl Server{
    pub fn new(
        addr: String,
    ) -> Self {
        Self {
            addr: addr,
        }
    }
    
    pub async fn listen(&self) {
        info!("listening to incoming connections on {}",self.addr);
        let listener = match tokio::net::TcpListener::bind(self.addr.clone()).await {
            Ok(conn) => {
                conn
            },
            Err(e) => {
                error!("could not listen on {} error : {}", self.addr,e);
                return
            }
        };

        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let conn = Conn::handle(stream);
                    let handle = tokio::spawn(async move {
                        conn.await
                    });
                },
                Err(e) => {

                }
            }
        }
    }
}