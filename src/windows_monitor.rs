use std::sync::mpsc::Sender;
use std::sync::OnceLock;
use std::ptr;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::winbase::QueryFullProcessImageNameW;
use winapi::um::winuser::{DispatchMessageW, GetMessageW, GetWindowThreadProcessId, SetWinEventHook, TranslateMessage, EVENT_SYSTEM_FOREGROUND, MSG, WINEVENT_OUTOFCONTEXT};
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::handleapi::CloseHandle;
use winapi::um::winnt::{PROCESS_QUERY_INFORMATION, PROCESS_VM_READ, LONG};
use winapi::shared::windef::{HWINEVENTHOOK,HWND};

use crate::layer_manager::KanawinState;

static SENDER: OnceLock<Sender<KanawinState>> = OnceLock::new();

//运行活动窗口变更检测循环
//run active window change detection loop
pub fn run_windows_monitor(sender:Sender<KanawinState>) {
    //在回调函数中使用SENDER发送消息
    //Use the SENDER in callback function to send messages
    let _ = SENDER.set(sender);

    unsafe {
        // 设置钩子 
        // set hook
        let hook = SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_FOREGROUND,
            ptr::null_mut(),
            Some(event_callback),
            0,
            0,
            WINEVENT_OUTOFCONTEXT | WINEVENT_OUTOFCONTEXT,
        );

        // 检测钩子是否设置成功 
        // check whether hook is set successfully
        if hook.is_null() {
            let error_code = GetLastError();
            log::error!("Failed to set hook! Error code: {}", error_code);
            return;
        }
        log::info!("WinEventHook set successfully!, {:?}", hook);
        
        //创建消息循环，否则回调函数不会执行。
        //Create a message loop, otherwise callback function will not execute
        let mut msg_buf: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg_buf, ptr::null_mut(), 0, 0) != -1 {
            TranslateMessage(&msg_buf);
            DispatchMessageW(&msg_buf);
        }
    }
}

//使用Sender向管理线程发送窗口改变信息
//send window change information to the manager thread
fn send_message(msg:String) {
    let _ = SENDER.get().unwrap().send(
        KanawinState{
            window:Some(msg),
            layer:None,
            stream:None,
        }
    );
}

//事件回调函数
//callback
unsafe extern "system" fn event_callback(
    _hwin_event_hook: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    _id_object: LONG,
    _id_child: LONG,
    _id_event_thread: u32,
    _dwms_event_time: u32,
) {
    match event {
        EVENT_SYSTEM_FOREGROUND => {
            match get_process_path(hwnd){
                Some(msg) => send_message(msg),
                None => send_message(String::from("None")),
            }
        }
        _ => (),
    }
}

//从 hwnd 获取进程可执行文件路径
//Get process executable path from hwnd
unsafe fn get_process_path(hwnd: HWND) -> Option<String> {
    let mut pid = 0;
    GetWindowThreadProcessId(hwnd, &mut pid);

    let process_handle = OpenProcess(
        PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
        false as i32,
        pid
    );
    if process_handle.is_null() {
        return None;
    }

    let mut path: [u16; 512] = [0; 512];
    QueryFullProcessImageNameW(
        process_handle,
        0,
        path.as_mut_ptr(),
        &mut (path.len() as u32)
    );

    CloseHandle(process_handle);

    if path.len() == 0{
        return None;
    }

    let path_str = String::from_utf16_lossy(&path);
    Some(path_str)
}