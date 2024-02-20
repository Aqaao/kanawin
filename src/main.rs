mod windows_monitor;
mod kanata_stream;
mod layer_manager;
mod configuration;

extern crate winapi;

use std::sync::mpsc;
use std::time::Duration;
use std::thread;
use anyhow::Result;
use clap::Parser;

use configuration::init_configuration;
use windows_monitor::run_windows_monitor;
use kanata_stream::run_tcp_client;
use layer_manager::run_layer_manager;

#[derive(Debug, Parser)]
#[clap(author, about, version, arg_required_else_help = true)]
struct Cli {
    /// Force change layer, detecting layer every time you switch it.
    /// Otherwise it will only be detected if the window focus changes.
    #[clap(short = 'f', long)]
    force_mode: bool,
    /// The port on which kanata's TCP server is running
    #[clap(short = 'p', long)]
    kanata_port: i32,
    /// Path to your configuration file
    #[clap(short, long, default_value = "kanawin.yaml")]
    configuration: String,
}

fn main() -> Result<()> {
    //设置日志等级
    //Set log level
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    //初始化日志功能，不打印时间戳
    //initialize log，dont print timestamps
    env_logger::builder().format_timestamp(None).init();

    //解析命令行参数 
    //Parse command line arguments
    let cli: Cli = Cli::parse();

    //读取配置 
    //Load config file
    init_configuration(&cli.configuration)?;

    //创建线程通信通道 
    //Creating thread communication channel
    let (sender1, receiver) = mpsc::channel();
    let sender2 = sender1.clone();
    //运行程序 
    //start run
    thread::spawn(move || run_layer_manager(receiver,cli.force_mode));
    thread::spawn(move || run_tcp_client(sender1, cli.kanata_port));
    thread::spawn(move || run_windows_monitor(sender2));

    loop {
        thread::sleep(Duration::from_secs(60));
    }
}
