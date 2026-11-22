use super::*;
use super::market_text::mt;
use crate::market::{ask_text, typing_supported, SUGGESTED};
use ume_market::Kind;

mod actions;
mod badges;
mod confirm;
mod grid;
mod items;
mod list;
mod portrait_tex;
mod rows;
mod toolbar;
mod ui_state;
pub use ui_state::*;
use rows::Row;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Act {
    Add,
    Refresh,
    Token,
    ToggleView,
    CycleFilter,
    MarkAll,
    ClearMarks,
    Install,
    Enable,
    Disable,
    Remove,
    RemoveRepo,
    Back,
    StopDownloads,
    SetFormat(Option<ume_market::view::Format>),
    ToggleOutdated,
    ToggleLayout,
    ToggleMissing,
    InstallAll,
    UpdateAll,
    AskRemoveAll,
    ConfirmRemove,
    CancelRemove,
}

const REPO_ROWS: usize = 9;
const FILTERS: [(Option<Kind>, &str); 5] = [
    (None, "Tudo"),
    (Some(Kind::Character), "Personagens"),
    (Some(Kind::Stage), "Cenários"),
    (Some(Kind::Music), "Músicas"),
    (Some(Kind::Pack), "Pacotes"),
];

fn kind_label(k: Kind) -> &'static str {
    match k {
        Kind::Character => mt("PERSONAGEM"),
        Kind::Stage => mt("CENÁRIO"),
        Kind::Music => mt("MÚSICA"),
        Kind::Pack => mt("PACOTE"),
    }
}

fn kind_color(k: Kind) -> Color {
    match k {
        Kind::Character => hex("#ff4d6d"),
        Kind::Stage => hex("#22e8ff"),
        Kind::Music => hex("#ffcc33"),
        Kind::Pack => hex("#b388ff"),
    }
}

fn fmt_bytes(b: u64) -> String {
    if b >= 1 << 20 {
        format!("{:.1} MB", b as f64 / (1u64 << 20) as f64)
    } else if b >= 1 << 10 {
        format!("{} KB", b >> 10)
    } else {
        format!("{b} B")
    }
}

fn short(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        format!(
            "{}…",
            s.chars().take(n.saturating_sub(1)).collect::<String>()
        )
    }
}

fn ago(secs: u64) -> String {
    let d = ume_market::now_secs().saturating_sub(secs);
    if secs == 0 {
        mt("não listado").into()
    } else if d < 90 {
        mt("agora há pouco").into()
    } else if d < 5400 {
        crate::i18n::fill(crate::i18n::tr3("há {n} min", "{n} min ago", "hace {n} min"), &[("n", &(d / 60).to_string())])
    } else if d < 172_800 {
        crate::i18n::fill(crate::i18n::tr3("há {n} h", "{n} h ago", "hace {n} h"), &[("n", &(d / 3600).to_string())])
    } else {
        crate::i18n::fill(crate::i18n::tr3("há {n} dias", "{n} days ago", "hace {n} días"), &[("n", &(d / 86_400).to_string())])
    }
}

impl App {
    pub(crate) fn market_ui_mark(&mut self, repo_key: &str, id: &str) {
        self.market_ui
            .marked
            .insert((repo_key.to_string(), id.to_string()));
    }
    pub(crate) fn market_ui_filter(&mut self, format: Option<ume_market::view::Format>, outdated_only: bool) {
        self.market_ui.format = format;
        self.market_ui.outdated_only = outdated_only;
        self.market_ui.cur = 0;
        self.market_ui.scroll = 0;
    }
    fn market_remove(&mut self, pairs: Vec<(String, String)>) {
        for (a, b) in &pairs {
            self.market.uninstall(a, b);
        }
        self.market.notice = format!("{} item(ns) removido(s) deste aparelho.", pairs.len());
        self.market_ui.marked.clear();
    }
    pub(crate) fn market_ui_view_installed(&mut self) {
        self.market_ui.view = View::Installed;
        self.market_ui.cur = 0;
    }

    fn market_entry_modal(&mut self) {
        let Some(mut e) = self.market_ui.entry.take() else {
            return;
        };
        let typed = typed_text();
        e.buf.push_str(&typed);
        if is_key_pressed(KeyCode::Backspace) {
            e.buf.pop();
            self.market_ui.backspace_hold = -0.4;
        } else if is_key_down(KeyCode::Backspace) {
            self.market_ui.backspace_hold += get_frame_time();
            if self.market_ui.backspace_hold > 0.05 {
                self.market_ui.backspace_hold = 0.0;
                e.buf.pop();
            }
        } else {
            self.market_ui.backspace_hold = 0.0;
        }
        let (x, y, w, h) = (190.0, 170.0, 900.0, 380.0);
        self.ui.adapted_modal(x, y, w, h, VW, VH);
        self.ui.text(
            if e.token {
                mt("TOKEN DO GITHUB (opcional)")
            } else {
                mt("ADICIONAR REPOSITÓRIO DO GITHUB")
            },
            x + 30.0,
            y + 48.0,
            26.0,
            hex("#ffcc33"),
            Align::Left,
            true,
        );
        self.ui.text(if e.token { mt("Aumenta o limite de requisições da API. Fica salvo só neste aparelho.") } else { mt("Cole ou digite o link (ex.: https://github.com/dono/repositorio) ou apenas dono/repositorio.") }, x + 30.0, y + 82.0, 17.0, hex("#a49cc0"), Align::Left, false);
        self.ui.adapted_field(x + 30.0, y + 104.0, w - 60.0, 46.0);
        let shown = if e.token {
            "•".repeat(e.buf.chars().count())
        } else {
            e.buf.clone()
        };
        let caret = if (self.ui.t * 2.0) as i32 % 2 == 0 {
            "|"
        } else {
            ""
        };
        self.ui.text(
            &format!("{}{caret}", short(&shown, 70)),
            x + 44.0,
            y + 136.0,
            24.0,
            WHITE,
            Align::Left,
            false,
        );
        if !typing_supported() && !e.token {
            self.ui.text(
                mt("Este aparelho não tem teclado em jogo: escolha uma sugestão abaixo."),
                x + 30.0,
                y + 176.0,
                17.0,
                hex("#ff9e6d"),
                Align::Left,
                false,
            );
        }
        if !e.token {
            self.ui.text(
                mt("Sugestões"),
                x + 30.0,
                y + 206.0,
                18.0,
                hex("#a49cc0"),
                Align::Left,
                true,
            );
            for (i, (repo, desc)) in SUGGESTED.iter().enumerate() {
                let ry = y + 218.0 + i as f32 * 46.0;
                let sel = i == e.suggestion;
                self.ui.adapted_row(x + 30.0, ry, w - 60.0, 40.0, true);
                if sel {
                    self.ui.adapted_highlight(("mkt.sug.x", "mkt.sug.y"), x + 30.0, ry, w - 60.0, 40.0);
                }
                self.ui
                    .text(repo, x + 44.0, ry + 26.0, 20.0, WHITE, Align::Left, true);
                self.ui.text(
                    desc,
                    x + 330.0,
                    ry + 26.0,
                    16.0,
                    hex("#a49cc0"),
                    Align::Left,
                    false,
                );
                if self.clicked_rect(x + 30.0, ry, w - 60.0, 40.0) {
                    e.suggestion = i;
                    e.buf = repo.to_string();
                }
            }
            if self.c.pressed(nav::DOWN) {
                e.suggestion = (e.suggestion + 1) % SUGGESTED.len();
            }
            if self.c.pressed(nav::UP) {
                e.suggestion = (e.suggestion + SUGGESTED.len() - 1) % SUGGESTED.len();
            }
        }
        let ok = is_key_pressed(KeyCode::Enter)
            || is_key_pressed(KeyCode::KpEnter)
            || self.market_button(
                x + w - 330.0,
                y + h - 70.0,
                150.0,
                "OK",
                "Enter",
                hex("#2ecc71"),
                true,
            );
        let cancel = is_key_pressed(KeyCode::Escape)
            || self.market_button(
                x + w - 170.0,
                y + h - 70.0,
                140.0,
                mt("Cancelar"),
                "Esc",
                hex("#ff4d6d"),
                true,
            );
        if ok {
            let text = if e.buf.trim().is_empty() && !e.token {
                SUGGESTED[e.suggestion].0.to_string()
            } else {
                e.buf.clone()
            };
            if e.token {
                self.market.set_token(&text);
            } else {
                match self.market.add_repo(&text) {
                    Ok(key) => {
                        if let Some(i) = self.market.state.repos.iter().position(|r| r.key == key) {
                            self.market_ui.repo = i;
                            self.market_ui.view = View::Browse;
                            self.market_ui.cur = 0;
                            self.market_ui.scroll = 0;
                        }
                    }
                    Err(m) => self.msg = m,
                }
            }
            return;
        }
        if cancel {
            return;
        }
        self.market_ui.entry = Some(e);
    }

    fn market_open_entry(&mut self, token: bool) {
        if let Some(v) = ask_text(
            if token {
                mt("Token do GitHub (deixe vazio para remover)")
            } else {
                mt("Link do repositório no GitHub (dono/repositorio)")
            },
            "",
        ) {
            if token {
                self.market.set_token(&v);
            } else if let Err(m) = self.market.add_repo(&v) {
                self.msg = m;
            }
            return;
        }
        if cfg!(target_arch = "wasm32") {
            return;
        }
        self.market_ui.entry = Some(TextEntry {
            token,
            buf: String::new(),
            suggestion: 0,
        });
    }

    pub(super) fn market_screen(&mut self) {
        crate::screen_art::ScreenArt::Market.background(&self.ui);
        self.ui.adapted_header(mt("MERCADO · REPOSITÓRIOS GITHUB"), "", VW);
        self.ui.text(
            mt("Importe personagens, cenários e músicas de qualquer repositório público do GitHub"),
            74.0,
            96.0,
            18.0,
            hex("#a49cc0"),
            Align::Left,
            false,
        );
        if !self.market.ready {
            self.ui.text(
                mt("Carregando dados locais…"),
                VW / 2.0,
                360.0,
                26.0,
                WHITE,
                Align::Center,
                true,
            );
            if self.c.pressed(nav::BACK) {
                self.screen = Screen::Menu { cur: 6 };
            }
            return;
        }
        if !self.market_ui.opened {
            self.market_ui.opened = true;
            if let Some(k) = self
                .market
                .state
                .repos
                .get(self.market_ui.repo)
                .map(|r| r.key.clone())
            {
                if self.market.needs_refresh(&k) {
                    self.market.refresh(&k);
                }
            }
        }
        let modal = self.market_ui.entry.is_some() || self.market_ui.confirm.is_some();
        let n_repos = self.market.state.repos.len();
        self.market_ui.repo = self.market_ui.repo.min(n_repos.saturating_sub(1));
        let rows = self.market_rows();
        let total = rows.len();
        self.market_ui.cur = self.market_ui.cur.min(total.saturating_sub(1));
        let mut acts: Vec<Act> = vec![];
        let browse = self.market_ui.view == View::Browse;

        let (lx, ly, lw) = (40.0, 112.0, 330.0);
        crate::menu_art::glass(&self.ui, lx, ly, lw, 484.0, 1.0);
        self.ui.text(
            mt("REPOSITÓRIOS"),
            lx + 16.0,
            ly + 30.0,
            20.0,
            hex("#a49cc0"),
            Align::Left,
            true,
        );
        let add_h = self.hover_rect(lx + lw - 130.0, ly + 8.0, 120.0, 32.0);
        self.ui.adapted_plate(lx + lw - 130.0, ly + 8.0, 120.0, 32.0, Some(alpha(hex("#2ecc71"), if add_h { 1.0 } else { 0.7 })));
        self.ui.text(
            mt("+ Adicionar"),
            lx + lw - 70.0,
            ly + 31.0,
            17.0,
            WHITE,
            Align::Center,
            true,
        );
        if self.clicked_rect(lx + lw - 130.0, ly + 8.0, 120.0, 32.0) && !modal {
            acts.push(Act::Add);
        }
        if n_repos == 0 {
            self.ui.text(
                mt("Nenhum repositório ainda."),
                lx + 16.0,
                ly + 90.0,
                18.0,
                WHITE,
                Align::Left,
                false,
            );
            self.ui.text(
                mt("Pressione U ou toque em Adicionar."),
                lx + 16.0,
                ly + 116.0,
                16.0,
                hex("#a49cc0"),
                Align::Left,
                false,
            );
        }
        if self.market_ui.repo < self.market_ui.repo_scroll {
            self.market_ui.repo_scroll = self.market_ui.repo;
        }
        if self.market_ui.repo >= self.market_ui.repo_scroll + REPO_ROWS {
            self.market_ui.repo_scroll = self.market_ui.repo + 1 - REPO_ROWS;
        }
        for (vi, ri) in (self.market_ui.repo_scroll..n_repos)
            .take(REPO_ROWS)
            .enumerate()
        {
            let (key, name, count, listed) = {
                let r = &self.market.state.repos[ri];
                (r.key.clone(), r.repo.display(), r.item_count, r.listed_at)
            };
            let y = ly + 52.0 + vi as f32 * 48.0;
            let sel = ri == self.market_ui.repo && browse;
            let hov = self.hover_rect(lx + 8.0, y, lw - 16.0, 44.0);
            self.ui.adapted_row(lx + 8.0, y, lw - 16.0, 44.0, hov || sel);
            if sel && self.market_ui.pane == Pane::Repos {
                self.ui.adapted_highlight(("mkt.repo.x", "mkt.repo.y"), lx + 8.0, y, lw - 16.0, 44.0);
            } else if sel {
                self.ui.adapted_plate(lx + 8.0, y, lw - 16.0, 44.0, Some(crate::tweaks::accent(0.4)));
            }
            self.ui.text(
                &short(&name, 28),
                lx + 20.0,
                y + 20.0,
                19.0,
                WHITE,
                Align::Left,
                true,
            );
            let sub = if self.market.listing.contains(&key) {
                mt("listando…").to_string()
            } else {
                format!("{count} itens · {}", ago(listed))
            };
            self.ui.text(
                &sub,
                lx + 20.0,
                y + 38.0,
                14.0,
                hex("#a49cc0"),
                Align::Left,
                false,
            );
            if self.clicked_rect(lx + 8.0, y, lw - 16.0, 44.0) && !modal {
                self.market_ui.repo = ri;
                self.market_ui.pane = Pane::Repos;
                self.market_ui.view = View::Browse;
                self.market_ui.cur = 0;
                self.market_ui.scroll = 0;
                if self.market.needs_refresh(&key) {
                    self.market.refresh(&key);
                }
            }
        }
        let tok = self.market.token().is_some();
        self.ui.text(
            if tok {
                mt("Token do GitHub: ativo (T altera)")
            } else {
                mt("Sem token: limite de 60 requisições/h (T define)")
            },
            lx + 16.0,
            ly + 476.0,
            14.0,
            hex("#a49cc0"),
            Align::Left,
            false,
        );

        let (rx, ry, rw) = (390.0, 112.0, 850.0);
        crate::menu_art::glass(&self.ui, rx, ry, rw, 484.0, 1.0);
        for (i, (v, label)) in [(View::Browse, mt("Explorar")), (View::Installed, mt("Instalados"))]
            .iter()
            .enumerate()
        {
            let x = rx + 12.0 + i as f32 * 150.0;
            let on = self.market_ui.view == *v;
            self.ui.adapted_tab(x, ry + 8.0, 140.0, 34.0, on);
            let n = if *v == View::Installed {
                format!(" ({})", self.market.state.installed.len())
            } else {
                String::new()
            };
            self.ui.text(
                &format!("{label}{n}"),
                x + 70.0,
                ry + 32.0,
                18.0,
                WHITE,
                Align::Center,
                true,
            );
            if self.clicked_rect(x, ry + 8.0, 140.0, 34.0) && !modal && self.market_ui.view != *v {
                acts.push(Act::ToggleView);
            }
        }
        self.market_toolbar(rx, ry, rw, modal, &mut acts);
        for (i, (f, label)) in FILTERS.iter().map(|(f, l)| (f, mt(l))).enumerate() {
            let x = rx + 330.0 + i as f32 * 100.0;
            let on = self.market_ui.filter == *f;
            self.ui.adapted_plate(x, ry + 10.0, 94.0, 30.0, on.then(|| alpha(hex("#22e8ff"), 0.6)));
            self.ui.text(
                label,
                x + 47.0,
                ry + 31.0,
                15.0,
                WHITE,
                Align::Center,
                false,
            );
            if self.clicked_rect(x, ry + 10.0, 94.0, 30.0) && !modal {
                self.market_ui.filter = *f;
                self.market_ui.cur = 0;
                self.market_ui.scroll = 0;
            }
        }
        let cur_repo = self.market.state.repos.get(self.market_ui.repo).map(|r| r.key.clone());
        let listing_now = cur_repo.as_ref().is_some_and(|k| self.market.listing.contains(k));
        if total == 0 && self.market_ui.unfiltered > 0 {
            self.ui.text(mt("Nenhum item com estes filtros. F, J e P limpam os filtros."), rx + rw / 2.0, ry + 220.0, 20.0, WHITE, Align::Center, false);
        } else if total == 0 {
            self.market_empty_state(browse, cur_repo.is_some(), listing_now, rx, ry, rw);
        }
        self.market_items(&rows, rx, ry, rw, modal);

        self.market_holo_overlay(rx, ry, rw);
        let (px, py) = (rx, ry + 492.0);
        if self.market.busy() {
            let p = &self.market.progress;
            let frac = if p.total_bytes > 0 {
                p.done_bytes as f32 / p.total_bytes as f32
            } else if p.total_files > 0 {
                p.done_files as f32 / p.total_files as f32
            } else {
                0.0
            };
            self.ui.adapted_meter(px, py, rw, frac);
            let q = self.market.queue.len();
            self.ui.text(
                &crate::i18n::fill(
                    crate::i18n::tr3("Baixando {name} · {done}/{total} arquivos · {bytes}{queue}", "Downloading {name} · {done}/{total} files · {bytes}{queue}", "Descargando {name} · {done}/{total} archivos · {bytes}{queue}"),
                    &[("name", &self.market.current_name().unwrap_or_default()), ("done", &p.done_files.to_string()), ("total", &p.total_files.to_string()), ("bytes", &fmt_bytes(p.done_bytes)), ("queue", &if q > 0 { format!(" · {q} {}", crate::i18n::tr3("na fila", "queued", "en cola")) } else { String::new() })],
                ),
                px,
                py + 32.0,
                16.0,
                WHITE,
                Align::Left,
                false,
            );
            if self.clicked_rect(px + rw - 110.0, py + 12.0, 110.0, 26.0) {
                acts.push(Act::StopDownloads);
            }
            self.ui.adapted_plate(px + rw - 110.0, py + 12.0, 110.0, 26.0, Some(alpha(hex("#ff4d6d"), 0.8)));
            self.ui.text(
                mt("Cancelar"),
                px + rw - 55.0,
                py + 31.0,
                15.0,
                WHITE,
                Align::Center,
                true,
            );
        } else if !self.market.notice.is_empty() {
            self.ui.text(
                &short(&self.market.notice, 110),
                px,
                py + 20.0,
                17.0,
                if self.market.notice_error {
                    hex("#ff9e6d")
                } else {
                    hex("#6dff9e")
                },
                Align::Left,
                false,
            );
        } else if let Some(line) = self.market_contents_line(&rows) {
            self.ui.text(&short(&line, 110), px, py + 20.0, 16.0, hex("#c9c3dd"), Align::Left, false);
        } else {
            self.ui.text(
                &crate::i18n::fill(
                    crate::i18n::tr3("{marked} marcados · {installed} instalados · {disk} no disco · {tracks} músicas ativas", "{marked} marked · {installed} installed · {disk} on disk · {tracks} active tracks", "{marked} marcados · {installed} instalados · {disk} en disco · {tracks} pistas activas"),
                    &[("marked", &self.market_ui.marked.len().to_string()), ("installed", &self.market.state.installed.len().to_string()), ("disk", &fmt_bytes(self.market.state.total_bytes())), ("tracks", &self.market.active_tracks().to_string())],
                ),
                px,
                py + 20.0,
                16.0,
                hex("#a49cc0"),
                Align::Left,
                false,
            );
        }
        let by = 640.0;
        self.market_action_bar(&rows, rx, rw, modal, &mut acts);
        self.market_links(rows.get(self.market_ui.cur).map(|r| (r.repo_key.clone(), r.id.clone())), modal);
        self.ui.adapted_plate(40.0, by, 330.0, 48.0, Some(crate::tweaks::accent(0.9)));
        self.ui.text(mt("Voltar"), 205.0, by + 32.0, 26.0, WHITE, Align::Center, true);
        if self.clicked_rect(40.0, by, 330.0, 48.0) && !modal {
            acts.push(Act::Back);
        }
        self.ui.text(mt("U adicionar · T token · Tab explorar/instalados · F tipo · J formato · P desatualizados · K não instalados · V lista/grade · Espaço marca · Enter/Start aplica"), 40.0, 712.0, 14.0, hex("#a79fc4"), Align::Left, false);

        self.market_entry_modal();
        self.market_confirm_modal(&mut acts);
        if !modal {
            let geo = self.market_geo();
            let last = total.saturating_sub(1);
            if self.c.pressed(nav::DOWN) {
                if self.market_ui.pane == Pane::Items {
                    let next = self.market_ui.cur + geo.cols;
                    let line_end = (self.market_ui.cur / geo.cols + 1) * geo.cols;
                    self.market_ui.cur = if next <= last { next } else if line_end <= last { last } else { self.market_ui.cur };
                } else if n_repos > 0 {
                    self.market_ui.repo = (self.market_ui.repo + 1).min(n_repos - 1);
                    self.market_ui.cur = 0;
                    self.market_ui.scroll = 0;
                }
            }
            if self.c.pressed(nav::UP) {
                if self.market_ui.pane == Pane::Items {
                    self.market_ui.cur = self.market_ui.cur.saturating_sub(geo.cols);
                } else {
                    self.market_ui.repo = self.market_ui.repo.saturating_sub(1);
                    self.market_ui.cur = 0;
                    self.market_ui.scroll = 0;
                }
            }
            if is_key_pressed(KeyCode::PageDown) {
                self.market_ui.cur = (self.market_ui.cur + geo.vis * geo.cols).min(last);
            }
            if is_key_pressed(KeyCode::PageUp) {
                self.market_ui.cur = self.market_ui.cur.saturating_sub(geo.vis * geo.cols);
            }
            let in_grid_row = geo.cols > 1 && self.market_ui.pane == Pane::Items;
            if self.c.pressed(nav::LEFT) {
                if in_grid_row && self.market_ui.cur % geo.cols > 0 {
                    self.market_ui.cur -= 1;
                } else {
                    self.market_ui.pane = Pane::Repos;
                    self.market_ui.view = View::Browse;
                }
            }
            if self.c.pressed(nav::RIGHT) {
                if in_grid_row && self.market_ui.cur % geo.cols < geo.cols - 1 && self.market_ui.cur < last {
                    self.market_ui.cur += 1;
                }
                self.market_ui.pane = Pane::Items;
            }
            if self.c.pressed(nav::OK) {
                if self.market_ui.pane == Pane::Items {
                    if let Some(r) = rows.get(self.market_ui.cur) {
                        let k = (r.repo_key.clone(), r.id.clone());
                        if !self.market_ui.marked.remove(&k) {
                            self.market_ui.marked.insert(k);
                        }
                    }
                } else {
                    self.market_ui.pane = Pane::Items;
                    if let Some(k) = cur_repo.clone() {
                        if self.market.needs_refresh(&k) {
                            self.market.refresh(&k);
                        }
                    }
                }
            }
            if is_key_pressed(KeyCode::U) {
                acts.push(Act::Add);
            }
            if is_key_pressed(KeyCode::T) {
                acts.push(Act::Token);
            }
            if is_key_pressed(KeyCode::R) {
                acts.push(Act::Refresh);
            }
            if is_key_pressed(KeyCode::Tab) {
                acts.push(Act::ToggleView);
            }
            if is_key_pressed(KeyCode::F) {
                acts.push(Act::CycleFilter);
            }
            if is_key_pressed(KeyCode::J) {
                acts.push(Act::SetFormat(ume_market::view::next_format(self.market_ui.format)));
            }
            if is_key_pressed(KeyCode::P) {
                acts.push(Act::ToggleOutdated);
            }
            for (code, act) in [(KeyCode::K, Act::ToggleMissing), (KeyCode::Z, Act::InstallAll), (KeyCode::Y, Act::UpdateAll), (KeyCode::B, Act::AskRemoveAll)] {
                if is_key_pressed(code) {
                    acts.push(act);
                }
            }
            if is_key_pressed(KeyCode::V) || self.c.pressed(nav::AUX) {
                acts.push(Act::ToggleLayout);
            }
            if is_key_pressed(KeyCode::M) {
                acts.push(Act::MarkAll);
            }
            if is_key_pressed(KeyCode::N) {
                acts.push(Act::ClearMarks);
            }
            if is_key_pressed(KeyCode::I) {
                acts.push(Act::Install);
            }
            if is_key_pressed(KeyCode::E) {
                acts.push(Act::Enable);
            }
            if is_key_pressed(KeyCode::O) {
                acts.push(Act::Disable);
            }
            if is_key_pressed(KeyCode::X) || is_key_pressed(KeyCode::Delete) {
                acts.push(if self.market_ui.pane == Pane::Repos {
                    Act::RemoveRepo
                } else {
                    Act::Remove
                });
            }
            if self.c.pressed(nav::START) {
                acts.push(if browse {
                    Act::Install
                } else if rows.iter().any(|r| r.installed == Some(false)) {
                    Act::Enable
                } else {
                    Act::Disable
                });
            }
            if self.c.pressed(nav::BACK) {
                acts.push(Act::Back);
            }
        }

        let targets: Vec<Row> = {
            let marked: Vec<Row> = rows
                .iter()
                .filter(|r| {
                    self.market_ui
                        .marked
                        .contains(&(r.repo_key.clone(), r.id.clone()))
                })
                .cloned()
                .collect();
            if marked.is_empty() {
                rows.get(self.market_ui.cur).cloned().into_iter().collect()
            } else {
                marked
            }
        };
        for act in acts {
            self.market.notice.clear();
            self.market.notice_error = false;
            match act {
                Act::Back => {
                    self.market_ui.opened = false;
                    self.screen = Screen::Menu { cur: 6 };
                }
                Act::Add => self.market_open_entry(false),
                Act::Token => self.market_open_entry(true),
                Act::Refresh => {
                    if let Some(k) = cur_repo.clone() {
                        self.market.refresh(&k);
                    }
                }
                Act::ToggleView => {
                    self.market_ui.view = if browse {
                        View::Installed
                    } else {
                        View::Browse
                    };
                    self.market_ui.cur = 0;
                    self.market_ui.scroll = 0;
                    self.market_ui.pane = Pane::Items;
                }
                Act::CycleFilter => {
                    let i = FILTERS
                        .iter()
                        .position(|(f, _)| *f == self.market_ui.filter)
                        .unwrap_or(0);
                    self.market_ui.filter = FILTERS[(i + 1) % FILTERS.len()].0;
                    self.market_ui.cur = 0;
                    self.market_ui.scroll = 0;
                }
                Act::SetFormat(f) => {
                    self.market_ui.format = f;
                    self.market_ui.cur = 0;
                    self.market_ui.scroll = 0;
                }
                Act::ToggleOutdated => {
                    self.market_ui.outdated_only = !self.market_ui.outdated_only;
                    self.market_ui.cur = 0;
                    self.market_ui.scroll = 0;
                }
                Act::ToggleLayout => {
                    self.market.state.grid_view = !self.market.state.grid_view;
                    self.market.save_state();
                    self.market_ui.cur = 0;
                    self.market_ui.scroll = 0;
                }
                Act::MarkAll => {
                    for r in &rows {
                        self.market_ui
                            .marked
                            .insert((r.repo_key.clone(), r.id.clone()));
                    }
                }
                Act::ClearMarks => self.market_ui.marked.clear(),
                Act::Install if browse => { self.audio.sfx("purchase");
                    let ids: Vec<String> = targets
                        .iter()
                        .filter(|r| r.installable)
                        .map(|r| r.id.clone())
                        .collect();
                    if let Some(k) = cur_repo.clone() {
                        let n = self.market.queue_install(&k, &ids);
                        self.market.notice = if n == 0 {
                            mt("Nada novo para baixar (itens grandes demais são ignorados).").into()
                        } else {
                            format!("{n} item(ns) na fila de download.")
                        };
                    }
                    self.market_ui.marked.clear();
                }
                Act::Install => { self.audio.sfx("error");
                    self.market.notice = mt("Volte para Explorar para baixar novos itens.").into()
                }
                Act::Enable | Act::Disable => { self.audio.sfx("toggle");
                    let on = act == Act::Enable;
                    let pairs: Vec<(String, String)> = targets
                        .iter()
                        .filter(|r| r.installed.is_some())
                        .map(|r| (r.repo_key.clone(), r.id.clone()))
                        .collect();
                    let n = self
                        .market
                        .set_active(pairs.iter().map(|(a, b)| (a.as_str(), b.as_str())), on);
                    self.market.notice = format!(
                        "{n} {} {}.",
                        mt("item(ns)"),
                        if on { mt("ativado(s)") } else { mt("desativado(s)") }
                    );
                }
                Act::Remove | Act::AskRemoveAll => {
                    let source = if act == Act::Remove { &targets } else { &rows };
                    let pairs: Vec<(String, String)> = source.iter().filter(|r| r.installed.is_some()).map(Row::pair).collect();
                    if pairs.len() > 1 {
                        self.market_ui.confirm = Some(pairs);
                    } else {
                        self.market_remove(pairs);
                    }
                }
                Act::ConfirmRemove => {
                    let pairs = self.market_ui.confirm.take().unwrap_or_default();
                    self.market_remove(pairs);
                }
                Act::CancelRemove => self.market_ui.confirm = None,
                Act::ToggleMissing => {
                    self.market_ui.missing_only = !self.market_ui.missing_only;
                    self.market_ui.cur = 0;
                    self.market_ui.scroll = 0;
                }
                Act::InstallAll => {
                    self.audio.sfx("purchase");
                    let ids: Vec<String> = rows.iter().filter(|r| r.installed.is_none() && r.installable).map(|r| r.id.clone()).collect();
                    let n = cur_repo.as_ref().map_or(0, |k| self.market.queue_install(k, &ids));
                    self.market.notice = format!("{n} {}", mt("item(ns) na fila de download."));
                }
                Act::UpdateAll => {
                    self.audio.sfx("purchase");
                    let mut keys: Vec<&str> = rows.iter().filter(|r| r.outdated).map(|r| r.repo_key.as_str()).collect();
                    keys.dedup();
                    let mut n = 0;
                    for key in keys {
                        let ids: Vec<String> = rows.iter().filter(|r| r.outdated && r.repo_key == key).map(|r| r.id.clone()).collect();
                        n += self.market.queue_install(key, &ids);
                    }
                    self.market.notice = format!("{n} {}", mt("item(ns) na fila de download."));
                }
                Act::RemoveRepo => {
                    if let Some(k) = cur_repo.clone() {
                        self.market.remove_repo(&k);
                        self.market_ui.repo = 0;
                        self.market.notice =
                            mt("Repositório removido da lista (itens instalados continuam).").into();
                    }
                }
                Act::StopDownloads => {
                    if self.market.busy() {
                        self.market.cancel_all();
                    }
                }
            }
        }
    }
}
