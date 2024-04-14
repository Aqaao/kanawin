use std::io::{BufRead, BufReader, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::sync::mpsc::Sender;
use std::time::Duration;
use anyhow::Result;
use json_dotpath::DotPaths;
use serde_json::json;

use crate::layer_manager::KanawinState;

fn connect_to_kanata(port: i32) -> Result<TcpStream> {
    Ok(TcpStream::connect_timeout(
        &SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port.try_into().unwrap()),
        Duration::from_secs(5),
    )?)
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
            layers:None,
        })?;
        //请求获取kanata图层名称
        //get actual layers.
        let request_layers = json!({
            "RequestLayerNames": {}
        });
        let _ = stream.write_all(request_layers.to_string().as_bytes());
        //检查服务端消息确认当前Layer
        //Read Kanata message to confirm the current Layer
        loop {
            let mut buffer = String::new();
            let mut reader = BufReader::new(&stream);
            match reader.read_line(&mut buffer){
                Ok(_bytes) => {
                    // let data = String::from_utf8(buffer[0..bytes_read].to_vec())?;
                    // log::debug!("get msg : {:?}",data);
                    let notification: serde_json::Value = serde_json::from_str(&buffer)?;
                    if notification.dot_has("LayerChange.new") {
                        if let Some(new) = notification.dot_get::<String>("LayerChange.new")?{
                            sender.send(KanawinState{
                                window:None,
                                layer:Some(new),
                                stream:None,
                                layers:None,
                            })?;
                        }
                    }
                    else if notification.dot_has("CurrentLayerName.name"){
                        if let Some(name) = notification.dot_get::<String>("CurrentLayerName.name")?{
                            sender.send(KanawinState{
                                window:None,
                                layer:Some(name),
                                stream:None,
                                layers:None,
                            })?;
                        }
                    }
                    else if notification.dot_has("LayerNames.names"){
                        if let Some(names) = notification.dot_get::<Vec<String>>("LayerNames.names")?{
                            sender.send(KanawinState{
                                window:None,
                                layer:None,
                                stream:None,
                                layers:Some(names),
                            })?;
                        }
                    }
                    else if notification.dot_has("ConfigFileReload.new"){
                            let request_layers = json!({
                                "RequestLayerNames": {}
                            });
                            let _ = stream.write_all(request_layers.to_string().as_bytes());
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