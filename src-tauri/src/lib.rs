mod automation;
mod commands;
mod design;
mod engines;
mod image_server;
mod jobs;
mod models;
mod state;
mod text_model;

use tauri::Manager;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let state = AppState::initialize(app.handle()).map_err(std::io::Error::other)?;
            let should_start_automation = state.config().automation.enabled;
            app.manage(state.clone());

            if should_start_automation {
                tauri::async_runtime::spawn(async move {
                    if let Err(error) = automation::start(state).await {
                        eprintln!("Unable to start automation API: {error}");
                    }
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::save_config,
            commands::get_capabilities,
            commands::list_jobs,
            commands::get_job,
            commands::enqueue_generation,
            commands::cancel_job,
            commands::probe_engine,
            commands::probe_text_model,
            commands::generate_creative_plan,
            commands::render_design,
            commands::read_asset_preview,
            commands::start_automation_server,
            commands::start_image_server,
            commands::stop_image_server,
            commands::get_image_server_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Imgen Pro");
}
