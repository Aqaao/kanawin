use std::{path::PathBuf, sync::OnceLock};
use std::{env,fs};
use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;

//储存配置信息 
//Store configuration information
pub static CONFIG: OnceLock<Configuration> = OnceLock::new();
//储存配置中所有的layer，只有在配置中存在的layer才会被改变 
//Store all layers in configuration, only layers in configuration can be changed
pub static LAYERS: OnceLock<Layers> = OnceLock::new();


pub type Layers = Vec<String>;
pub type Configuration = Vec<Entry>;

#[derive(Debug,Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    pub exe: String,
    pub target_layer: String,
}

pub fn init_configuration(config:&str) -> Result<()> {
    let path = resolve_config_path(config)?;
    let file = &fs::read_to_string(path)?;
    let configuration:Configuration = serde_yaml::from_str(file)?;
    let mut layers:Vec<String>=Vec::new();
    for entry in &configuration{
        layers.push(entry.target_layer.clone());
    }
    let _ = LAYERS.set(layers);
    let _ = CONFIG.set(configuration);
    log::info!("load config finished!");
    log::info!("configuration: {:?}", CONFIG.get().unwrap());
    Ok(())
}

//如果配置文件路径是默认的，就在可执行文件目录中寻找 kanawin.yaml
//If no configuration file path is passed, wiil be searched in current directory.
fn resolve_config_path(raw_path: &str) -> Result<PathBuf> {
    if raw_path == "kanawin.yaml" {
        let path = format!("{}\\{}",env::current_dir().unwrap().to_str().unwrap(),raw_path);
        log::info!("get config path{:?}",path);
        Ok(fs::canonicalize(path)?)
    }
    else {
        log::info!("get config path{:?}",raw_path);
        Ok(fs::canonicalize(raw_path)?)
    }
}