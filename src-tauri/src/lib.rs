
mod helper;
mod types;
mod commands;

use crate::commands::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            docker_listen_events,
            docker_ps,
            docker_build_image,
            docker_images,
            docker_volumes,
            docker_networks,
            docker_start,
            docker_stop,
            docker_restart,
            docker_remove_container,
            docker_export_container,
            docker_import_image,
            docker_remove_image,
            docker_create_container,
            docker_logs,
            docker_logs_stream,
            docker_stop_logs_stream,
            docker_inspect,
            docker_exec,
            docker_info,
            docker_pull,
            docker_stop_pull,
            docker_stop_build,
            docker_tag_image,
            docker_image_history,
            docker_create_volume,
            docker_remove_volume,
            docker_create_network,
            docker_remove_network,
            docker_hub_search,
            docker_is_running,
            check_engine_cli_command,
            docker_compose_up,
            docker_compose_down,
            docker_compose_restart,
            docker_prune,
            docker_container_stats,
            docker_stats_stream,
            docker_stop_stats_stream,
            docker_container_ls,
            docker_container_read_file,
            docker_container_write_file,
            get_system_metrics,
            run_elevated_command,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
