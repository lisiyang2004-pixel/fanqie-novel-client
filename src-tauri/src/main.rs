// 防止 Windows 发布构建时出现控制台窗口
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    fanqie_novel_client_lib::run()
}
