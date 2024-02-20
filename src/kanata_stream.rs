use std::io::Read;
use std::net::TcpStream;
use std::sync::mpsc::Sender;
use std::time::Duration;
use anyhow::Result;
use json_dotpath::DotPaths;

use crate::layer_manager::KanawinState;

fn connect_to_kanata(port: i32) -> Result<TcpStream> {
    Ok(TcpStream::connect(format!("localhost:{port}"))?)
}

fn establish_connection(port: i32) -> Result<TcpStream> {
    let mut result = connect_to_kanata(port);
    while result.is_err() {
        log::warn!("cant connect to kanata tcp server, retrying connection in 5 seconds");
        std::thread::sleep(Duration::from_secs(5));
        result = connect_to_kanata(port);
    }
    log::info!("suceesed establish TCP connection!");
    result
}

//运行TCP循环
//run TCP loop
pub fn run_tcp_client (sender:Sender<KanawinState>,port: i32) -> Result<()> {
    loop {
        let mut stream = establish_connection(port)?;
        //将stream发送给管理线程
        //Send stream to manager thread
        sender.send(KanawinState{
            window:None,
            layer:None,
            stream:Some(stream.try_clone()?),
        })?;
        //检查服务端消息确认当前Layer
        //Read Kanata message to confirm the current Layer
        loop {
            let mut buffer = vec![0; 1024];
            match stream.read(&mut buffer){
                Ok(bytes_read) => {
                    let data = String::from_utf8(buffer[0..bytes_read].to_vec())?;
                    // log::debug!("get msg : {:?}",data);
                    let notification: serde_json::Value = serde_json::from_str(&data)?;
                    if notification.dot_has("LayerChange.new") {
                        if let Some(new) = notification.dot_get::<String>("LayerChange.new")?{
                            sender.send(KanawinState{
                                window:None,
                                layer:Some(new),
                                stream:None,
                            })?;
                        }
                    }
                },
                Err(_) => {
                    log::warn!("Could not get msg from server");
                    break;
                },
            }
        }
    }
}