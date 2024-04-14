use std::io::Write;
use std::net::TcpStream;
use std::sync::mpsc::Receiver;
use serde_json::json;

use crate::configuration::CONFIG;
use crate::configuration::LAYERS;

pub struct KanawinState {
    pub window:Option<String>,
    pub layer:Option<String>,
    pub stream:Option<TcpStream>,
    pub layers:Option<Vec<String>>,
}

//运行层控制循环，接受其他线程的消息，判断是否向 Kanata 发送指令
//run LayerManager loop, accept messages from other threads, determine whether to send request to Kanata
pub fn run_layer_manager(receiver:Receiver<KanawinState>, force_mode:bool) {
    let mut state = KanawinState{window:None, layer:None, stream:None, layers:None};
    loop {
        //recv 会阻塞线程直到收到新消息
        match receiver.recv() {
            Ok(msg) => {
                if let Some(get_stream) = msg.stream {
                    state.stream = Some(get_stream);
                    log::info!("get TCPstream, start working......");
                }
                if let Some(get_layer) = msg.layer {
                    state.layer = Some(get_layer);
                    log::info!("current layer: {}",state.layer.as_ref().unwrap());
                    if force_mode {check_layer(&mut state);}
                }
                if let Some(get_window) = msg.window {
                    state.window = Some(get_window);
                    log::debug!("current window: {}",state.window.as_ref().unwrap());
                    check_layer(&mut state);
                }
                if let Some(get_layers) = msg.layers {
                    state.layers = Some(get_layers);
                    log::info!("get actual layers: {:}",state.layers.as_ref().unwrap().join(","));
                }
            },
            Err(_err) => {
                log::error!("error when receive messeage from other threads!");
            },
        }
    }
}

//检查当前是否需要改变层，如果需要则向kanata发送请求
//Check if current layer needs to be changed, send requests to kanata if needed.
fn check_layer( state: &mut KanawinState,) {
    if state.stream.is_none() {
        log::warn!("stream is not load!");
        return;
    };
    if state.layer.is_none() {
        log::warn!("layer is not load!");
        return;
    };
    if state.window.is_none(){
        log::warn!("window is not load!");
        return;
    }
    if state.layers.is_none() {
        log::warn!("actual layers not acquired!");
        return;
    }
    if !LAYERS.get().unwrap().contains(state.layer.as_ref().unwrap()){
        log::debug!("current layers is not exist in configuration file");
        return;
    };


    let mut layer:Option<&str>= None;
    let configuration = CONFIG.get().unwrap();
    for entry in configuration {
        if state.window.as_ref().unwrap().contains(&entry.exe) {
            layer = Some(entry.target_layer.as_ref());
            break;
        }
        if entry.exe == "*" {
            layer =  Some(entry.target_layer.as_ref());
            break;
        }
    }
    match layer{
        Some(target_layer) => {
            if !state.layers.as_ref().unwrap().contains(&target_layer.to_string()){
                log::warn!("Failed to change layer, Kanata have not this layer: {target_layer}");
                return
            }
            if state.layer.as_ref().unwrap() != &target_layer{
                let stream = state.stream.as_mut().unwrap();
                let request_changelayer = json!({
                    "ChangeLayer": {
                        "new": target_layer,
                    }
                });
                if let Err(_)  = stream.write_all(request_changelayer.to_string().as_bytes()){
                    log::error!("send ChangeLayer request error!");
                    return;
                }
                let request_layername = json!({
                    "RequestCurrentLayerName": {}
                });
                let _ = stream.write_all(request_layername.to_string().as_bytes());
                log::debug!("Successfully send changed layer request: {}",target_layer);
            }
            else {
                log::debug!("Current layer is target layer");
            }
        },
        None => {
            log::debug!("No need to change layers");
        },
    }

}