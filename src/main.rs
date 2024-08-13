use app::App;

mod app;
mod sysinfo_thread;
mod tui;
mod utils;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut terminal = tui::init()?;
    let thread_rx = sysinfo_thread::start_thread()?;
    let app_result = App::default().run(&mut terminal, thread_rx).await;
    tui::restore()?;
    Ok(app_result?)
}
