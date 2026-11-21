use crate::app::{App, Screen};
use crate::smoke::ctx::Ctx;

pub async fn run(ctx: &mut Ctx, app: &mut App) {
    ctx.check("fighters registered", !app.reg.chars.is_empty(), format!("{} fighters", app.reg.chars.len()));
    ctx.check("stages registered", !app.reg.stages.is_empty(), format!("{} stages", app.reg.stages.len()));
    ctx.check("no quarantined content", app.reg.quarantined.is_empty(), format!("{} quarantined", app.reg.quarantined.len()));
    ctx.check("no import errors", app.reg.import_errors.is_empty(), app.reg.import_errors.join("; "));
    app.screen = Screen::Title;
    ctx.shot(app, "title", 30).await;
}
