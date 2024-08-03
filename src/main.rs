use app::App;

mod app;
mod tui;

fn main() -> anyhow::Result<()> {
    let mut terminal = tui::init()?;
    let app_result = App::default().run(&mut terminal);
    tui::restore()?;
    Ok(app_result?)
}
