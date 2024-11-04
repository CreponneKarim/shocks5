use std::{io::{self, Read}};

use tokio::{io::{AsyncReadExt, BufReader}, net::TcpStream, sync::watch::error::RecvError, task::JoinHandle};
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

pub trait Readable {
    async fn readable(&self) -> io::Result<()>;
    async fn read(&mut self, buf : &mut [u8]) -> Result<usize,std::io::Error>;
}

pub trait Addr {
    async fn addr(&self) -> io::Result<String>;    
}

pub trait ConnHandler 
    where 
        Self : Readable + Addr + Sized
{

    async fn handle(&mut self) -> Result<(),anyhow::Error> {
        const BUFF_NB_BYTES: usize = 1024;

        let conn_addr = self.addr().await?;
        
        match self.readable().await {
            Ok(()) => {
                loop {
                    let mut buff = [0; BUFF_NB_BYTES];
                    match self.read(&mut buff).await {
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
        Ok(())
    }
}

pub struct Conn {
    conn: TcpStream,
}

impl Addr for Conn {
    async fn addr(&self) -> io::Result<String> {
        self.conn.peer_addr().map(|res| {res.to_string()})
    }
}

impl Readable for Conn {
    async fn read(&mut self, buf : &mut [u8]) -> Result<usize,std::io::Error> {
        self.conn.read(buf).await     
    }
    async fn readable(&self) -> io::Result<()> {
        self.conn.readable().await
    }
}

impl ConnHandler for Conn{
}

impl Conn {
    pub fn new(conn: TcpStream) -> Self {
        Conn {
            conn:conn,
        }
    }
}


/// struct that listens to incoming connections and offloads them to tokio tasks
pub struct Server<T : Readable + Addr>
{
    addr: String,
    handlers: Vec<JoinHandle<T>>
}

impl <T:Readable + Addr + ConnHandler> Server<T>{
    pub fn new (
        addr: String,
    ) -> Self {
        Self {
            addr: addr,
            handlers: Vec::<JoinHandle<T>>::new()
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
                    let conn = Conn::new(stream);
                    let handler = conn.handle();
                    let handle = tokio::spawn(async move {
                        handler.await
                    });
                    self.handlers.push(handle);
                },
                Err(e) => {

                }
            }
        }
    }
}