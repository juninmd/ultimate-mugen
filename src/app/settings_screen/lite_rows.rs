use super::{cycle, on_off, row, Act, Row};
use crate::i18n::tr3;
use crate::lite::{CUSTOM, PRESET_COUNT, SCALES};
use crate::settings::Settings;

#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) enum LiteAct {
    Preset,
    Scale,
    Particles,
    Afterimage,
    Glass,
    Shadows,
    Backdrop,
    Crt,
    Auto,
}

fn preset_name(p: usize) -> &'static str {
    match p {
        0 => tr3("Alta", "High", "Alta"),
        1 => tr3("Média", "Medium", "Media"),
        2 => tr3("Baixa", "Low", "Baja"),
        3 => tr3("Mínima", "Minimum", "Mínima"),
        _ => tr3("Personalizada", "Custom", "Personalizada"),
    }
}

fn particles_name(p: u8) -> &'static str {
    match p {
        0 => tr3("Mínimas", "Minimal", "Mínimas"),
        1 => tr3("Reduzidas", "Reduced", "Reducidas"),
        _ => tr3("Todas", "Full", "Todas"),
    }
}

fn scale_label(k: f32) -> String {
    let (w, h) = ume_shell::quad::window_size();
    format!("{}% · {}×{}", (k * 100.0).round() as u32, (w * k).round() as u32, (h * k).round() as u32)
}

pub(super) fn rows(s: &Settings) -> Vec<Row> {
    let l = &s.lite;
    let t = tr3;
    vec![
        row(Act::Lite(LiteAct::Preset), t("Predefinição gráfica", "Graphics preset", "Preajuste gráfico"), preset_name(s.preset()), t("Ajusta todas as opções abaixo de uma vez", "Sets every option below at once", "Ajusta todas las opciones de abajo")),
        row(Act::Lite(LiteAct::Scale), t("Escala de renderização", "Render scale", "Escala de renderizado"), scale_label(l.render_scale()), t("Resolução interna da luta; menor é mais rápido", "Internal fight resolution; lower is faster", "Resolución interna del combate; menor es más rápido")),
        row(Act::Fx, t("Pós-processamento", "Post-processing", "Posprocesado"), crate::settings_labels::fx_level(s.fx), t("Bloom, vinheta e correção de cor", "Bloom, vignette and colour grading", "Bloom, viñeta y corrección de color")),
        row(Act::Quality, t("Qualidade da arte", "Art quality", "Calidad del arte"), crate::settings_labels::art_quality(s.quality), t("Menos memória e GPU em máquinas fracas", "Less memory and GPU on weak machines", "Menos memoria y GPU en equipos lentos")),
        row(Act::Lite(LiteAct::Particles), t("Partículas e efeitos", "Particles and effects", "Partículas y efectos"), particles_name(l.particles), t("Faíscas, poeira e brilhos de impacto", "Sparks, dust and impact flashes", "Chispas, polvo y destellos de impacto")),
        row(Act::Lite(LiteAct::Afterimage), t("Rastro de imagem", "After image", "Estela de imagen"), on_off(l.afterimage), t("Cópias fantasma dos lutadores", "Ghost copies of fighters", "Copias fantasma de los luchadores")),
        row(Act::Lite(LiteAct::Glass), t("HUD translúcido", "Glass HUD", "HUD translúcido"), on_off(l.glass), t("Desligado usa painéis simples sem blur", "Off uses simple panels without blur", "Apagado usa paneles simples sin blur")),
        row(Act::Lite(LiteAct::Shadows), t("Sombras", "Shadows", "Sombras"), on_off(l.shadows), t("Sombras de contato sob os lutadores", "Contact shadows under fighters", "Sombras de contacto bajo los luchadores")),
        row(Act::Lite(LiteAct::Backdrop), t("Fundo animado", "Animated backdrop", "Fondo animado"), on_off(l.backdrop), t("Detalhes vivos do cenário", "Living stage details", "Detalles vivos del escenario")),
        row(Act::Lite(LiteAct::Crt), t("Filtro CRT", "CRT filter", "Filtro CRT"), on_off(l.crt), t("Linhas de varredura nas telas de título", "Scanlines on title screens", "Líneas de barrido en la pantalla de título")),
        row(Act::FpsCap, t("Limite de FPS", "FPS cap", "Límite de FPS"), match s.fps_cap_value() { 0 => t("Sem limite", "Unlimited", "Sin límite").to_string(), n => format!("{n} FPS") }, t("Limitar poupa CPU e bateria", "Capping saves CPU and battery", "Limitar ahorra CPU y batería")),
        row(Act::Lite(LiteAct::Auto), t("Ajuste automático", "Auto adjust", "Ajuste automático"), on_off(l.auto), t("Reduz os gráficos se o FPS cair nas lutas", "Lowers graphics if FPS drops in fights", "Reduce los gráficos si baja el FPS")),
    ]
}

pub(super) fn adjust(s: &mut Settings, act: LiteAct, step: i32) {
    let flip = |b: &mut bool| *b = !*b;
    let l = &mut s.lite;
    match act {
        LiteAct::Preset => {
            let current = s.preset();
            let next = match (current == CUSTOM, step > 0) {
                (true, true) => 0,
                (true, false) => PRESET_COUNT - 1,
                _ => cycle(current, step, PRESET_COUNT),
            };
            s.set_preset(next);
        }
        LiteAct::Scale => l.scale = cycle(l.scale as usize, step, SCALES.len()) as u8,
        LiteAct::Particles => l.particles = cycle(l.particles as usize, step, 3) as u8,
        LiteAct::Afterimage => flip(&mut l.afterimage),
        LiteAct::Glass => flip(&mut l.glass),
        LiteAct::Shadows => flip(&mut l.shadows),
        LiteAct::Backdrop => flip(&mut l.backdrop),
        LiteAct::Crt => flip(&mut l.crt),
        LiteAct::Auto => flip(&mut l.auto),
    }
}
