mod boot;
mod fight;
mod modes;
mod screens;

use super::ctx::Ctx;
use crate::app::App;
use std::future::Future;
use std::pin::Pin;

pub type Run = for<'a> fn(&'a mut Ctx, &'a mut App) -> Pin<Box<dyn Future<Output = ()> + 'a>>;

pub struct Feature {
    pub id: &'static str,
    pub title: &'static str,
    pub run: Run,
}

macro_rules! feature {
    ($id:literal, $title:literal, $path:path) => {
        Feature { id: $id, title: $title, run: |ctx, app| Box::pin($path(ctx, app)) }
    };
}

pub const ALL: &[Feature] = &[
    feature!("boot", "Boot and registry", boot::run),
    feature!("screens", "Menu and utility screens", screens::run),
    feature!("select", "Fighter and stage selection", modes::select),
    feature!("fight_1v1", "Fight 1v1 to results", fight::run),
    feature!("training", "Training mode", modes::training),
    feature!("team_5v5", "Team 5v5", modes::team),
    feature!("story", "Story mode", modes::story),
];
