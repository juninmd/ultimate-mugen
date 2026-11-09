use super::*;
use super::market_text::mt;
use crate::market::{ask_text, typing_supported, SUGGESTED};
use std::collections::HashSet;
use ume_market::Kind;

#[derive(Clone, Copy, PartialEq)]
pub enum View {
    Browse,
    Installed,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Pane {
    Repos,
    Items,
}

pub struct TextEntry {
    pub token: bool,
    pub buf: String,
    pub suggestion: usize,
}

pub struct MarketUi {
    pub view: View,
    pub pane: Pane,
    pub repo: usize,
    pub repo_scroll: usize,
    pub cur: usize,
    pub scroll: usize,
    pub filter: Option<Kind>,
    pub marked: HashSet<(String, String)>,
    pub entry: Option<TextEntry>,
    pub backspace_hold: f32,
    pub opened: bool,
}

impl Default for MarketUi {
    fn default() -> Self {
        MarketUi {
            view: View::Browse,
            pane: Pane::Items,
            repo: 0,
            repo_scroll: 0,
            cur: 0,
            scroll: 0,
            filter: None,
            marked: HashSet::new(),
            entry: None,
            backspace_hold: 0.0,
            opened: false,
        }
    }
}

#[derive(Clone)]
struct Row {
    repo_key: String,
    id: String,
    kind: Kind,
    name: String,
    bytes: u64,
    installable: bool,
    installed: Option<bool>,
}

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
}

const ROWS: usize = 12;
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
    pub(crate) fn market_ui_view_installed(&mut self) {
        self.market_ui.view = View::Installed;
        self.market_ui.cur = 0;
    }

    fn market_rows(&mut self) -> Vec<Row> {
        let filter = self.market_ui.filter;
        let keep = |k: Kind| filter.is_none_or(|f| f == k);
        match self.market_ui.view {
            View::Installed => self
                .market
                .state
                .installed
                .iter()
                .filter(|i| keep(i.kind))
                .map(|i| Row {
                    repo_key: i.repo_key.clone(),
                    id: i.item_id.clone(),
                    kind: i.kind,
                    name: i.name.clone(),
                    bytes: i.bytes,
                    installable: true,
                    installed: Some(i.active),
                })
                .collect(),
            View::Browse => {
                let Some(key) = self
                    .market
                    .state
                    .repos
                    .get(self.market_ui.repo)
                    .map(|r| r.key.clone())
                else {
                    return vec![];
                };
                self.market.catalog(&key);
                let installed: std::collections::HashMap<&str, bool> = self
                    .market
                    .state
                    .installed
                    .iter()
                    .filter(|i| i.repo_key == key)
                    .map(|i| (i.item_id.as_str(), i.active))
                    .collect();
                let Some(cat) = self.market.catalog_cached(&key) else {
                    return vec![];
                };
                cat.items
                    .iter()
                    .filter(|i| keep(i.kind))
                    .map(|i| Row {
                        repo_key: key.clone(),
                        id: i.id.clone(),
                        kind: i.kind,
                        name: i.name.clone(),
                        bytes: i.bytes,
                        installable: i.installable,
                        installed: installed.get(i.id.as_str()).copied(),
                    })
                    .collect()
            }
        }
    }

    fn market_button(&self, x: f32, y: f32, w: f32, label: &str, key: &str, accent: Color) -> bool {
        let hover = self.hover_rect(x, y, w, 48.0);
        self.ui.rect(
            x,
            y,
            w,
            48.0,
            if hover {
                alpha(accent, 0.85)
            } else {
                alpha(accent, 0.45)
            },
        );
        self.ui.text(
            label,
            x + w / 2.0,
            y + 25.0,
            19.0,
            WHITE,
            Align::Center,
            true,
        );
        if !key.is_empty() {
            self.ui.text(
                key,
                x + w / 2.0,
                y + 42.0,
                12.0,
                Color::new(1.0, 1.0, 1.0, 0.7),
                Align::Center,
                false,
            );
        }
        self.clicked_rect(x, y, w, 48.0)
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
        self.ui
            .rect(0.0, 0.0, VW, VH, Color::new(0.0, 0.0, 0.0, 0.7));
        self.ui.rect(x, y, w, h, hex("#1a1030"));
        self.ui.rect_lines(x, y, w, h, 2.0, hex("#ff4d6d"));
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
        self.ui.rect(
            x + 30.0,
            y + 104.0,
            w - 60.0,
            46.0,
            Color::new(1.0, 1.0, 1.0, 0.1),
        );
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
                self.ui.rect(
                    x + 30.0,
                    ry,
                    w - 60.0,
                    40.0,
                    if sel {
                        alpha(hex("#ff4d6d"), 0.4)
                    } else {
                        Color::new(1.0, 1.0, 1.0, 0.06)
                    },
                );
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
            );
        let cancel = is_key_pressed(KeyCode::Escape)
            || self.market_button(
                x + w - 170.0,
                y + h - 70.0,
                140.0,
                mt("Cancelar"),
                "Esc",
                hex("#ff4d6d"),
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
        self.ui.background();
        self.ui.text(
            mt("MERCADO · REPOSITÓRIOS GITHUB"),
            40.0,
            56.0,
            30.0,
            hex("#ffcc33"),
            Align::Left,
            true,
        );
        self.ui.text(
            mt("Importe personagens, cenários e músicas de qualquer repositório público do GitHub"),
            40.0,
            86.0,
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
                self.screen = Screen::Menu { cur: 8 };
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
        let modal = self.market_ui.entry.is_some();
        let n_repos = self.market.state.repos.len();
        self.market_ui.repo = self.market_ui.repo.min(n_repos.saturating_sub(1));
        let rows = self.market_rows();
        let total = rows.len();
        self.market_ui.cur = self.market_ui.cur.min(total.saturating_sub(1));
        let mut acts: Vec<Act> = vec![];
        let browse = self.market_ui.view == View::Browse;

        let (lx, ly, lw) = (40.0, 112.0, 330.0);
        self.ui
            .rect(lx, ly, lw, 484.0, Color::new(1.0, 1.0, 1.0, 0.05));
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
        self.ui.rect(
            lx + lw - 130.0,
            ly + 8.0,
            120.0,
            32.0,
            if add_h {
                hex("#2ecc71")
            } else {
                alpha(hex("#2ecc71"), 0.6)
            },
        );
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
            self.ui.rect(
                lx + 8.0,
                y,
                lw - 16.0,
                44.0,
                if sel {
                    alpha(
                        hex("#ff4d6d"),
                        if self.market_ui.pane == Pane::Repos {
                            0.6
                        } else {
                            0.35
                        },
                    )
                } else if hov {
                    Color::new(1.0, 1.0, 1.0, 0.12)
                } else {
                    Color::new(1.0, 1.0, 1.0, 0.05)
                },
            );
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
        self.ui
            .rect(rx, ry, rw, 484.0, Color::new(1.0, 1.0, 1.0, 0.05));
        for (i, (v, label)) in [(View::Browse, mt("Explorar")), (View::Installed, mt("Instalados"))]
            .iter()
            .enumerate()
        {
            let x = rx + 12.0 + i as f32 * 150.0;
            let on = self.market_ui.view == *v;
            self.ui.rect(
                x,
                ry + 8.0,
                140.0,
                34.0,
                if on {
                    hex("#ff4d6d")
                } else {
                    Color::new(1.0, 1.0, 1.0, 0.1)
                },
            );
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
        for (i, (f, label)) in FILTERS.iter().map(|(f, l)| (f, mt(l))).enumerate() {
            let x = rx + 330.0 + i as f32 * 100.0;
            let on = self.market_ui.filter == *f;
            self.ui.rect(
                x,
                ry + 10.0,
                94.0,
                30.0,
                if on {
                    alpha(hex("#22e8ff"), 0.55)
                } else {
                    Color::new(1.0, 1.0, 1.0, 0.08)
                },
            );
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
        if self.market_ui.cur < self.market_ui.scroll {
            self.market_ui.scroll = self.market_ui.cur;
        }
        if self.market_ui.cur >= self.market_ui.scroll + ROWS {
            self.market_ui.scroll = self.market_ui.cur + 1 - ROWS;
        }
        let wheel = mouse_wheel().1;
        if !modal && wheel != 0.0 && self.hover_rect(rx, ry, rw, 484.0) {
            self.market_ui.scroll = (self.market_ui.scroll as i64 - wheel.signum() as i64 * 3)
                .clamp(0, total.saturating_sub(ROWS) as i64)
                as usize;
        }
        let cur_repo = self
            .market
            .state
            .repos
            .get(self.market_ui.repo)
            .map(|r| r.key.clone());
        let listing_now = cur_repo
            .as_ref()
            .is_some_and(|k| self.market.listing.contains(k));
        if total == 0 {
            self.market_empty_state(browse, cur_repo.is_some(), listing_now, rx, ry, rw);
        }
        let installing = self.market.current_name();
        for (vi, row) in rows
            .iter()
            .enumerate()
            .skip(self.market_ui.scroll)
            .take(ROWS)
        {
            let y = ry + 52.0 + (vi - self.market_ui.scroll) as f32 * 34.0;
            let focused = vi == self.market_ui.cur && self.market_ui.pane == Pane::Items;
            let hov = self.hover_rect(rx + 8.0, y, rw - 16.0, 32.0);
            if hov && mouse_delta_position().length() > 0.0 && !modal {
                self.market_ui.cur = vi;
            }
            self.ui.rect(
                rx + 8.0,
                y,
                rw - 16.0,
                32.0,
                if focused {
                    alpha(hex("#ff4d6d"), 0.4)
                } else if vi % 2 == 0 {
                    Color::new(1.0, 1.0, 1.0, 0.04)
                } else {
                    Color::new(1.0, 1.0, 1.0, 0.0)
                },
            );
            let marked = self
                .market_ui
                .marked
                .contains(&(row.repo_key.clone(), row.id.clone()));
            self.ui
                .rect_lines(rx + 20.0, y + 6.0, 20.0, 20.0, 2.0, WHITE);
            if marked {
                self.ui
                    .rect(rx + 24.0, y + 10.0, 12.0, 12.0, hex("#6dff9e"));
            }
            self.ui.rect(
                rx + 52.0,
                y + 8.0,
                96.0,
                18.0,
                alpha(kind_color(row.kind), 0.35),
            );
            self.ui.text(
                kind_label(row.kind),
                rx + 100.0,
                y + 22.0,
                12.0,
                WHITE,
                Align::Center,
                true,
            );
            self.ui.text(
                &short(&row.name, 44),
                rx + 162.0,
                y + 23.0,
                19.0,
                WHITE,
                Align::Left,
                false,
            );
            self.ui.text(
                &fmt_bytes(row.bytes),
                rx + 620.0,
                y + 23.0,
                16.0,
                hex("#a49cc0"),
                Align::Right,
                false,
            );
            let (label, col) = match row.installed {
                _ if installing.as_deref() == Some(row.name.as_str()) && self.market.busy() => {
                    (mt("baixando…"), hex("#22e8ff"))
                }
                Some(true) => (mt("ATIVO"), hex("#6dff9e")),
                Some(false) => (mt("instalado"), hex("#ffcc33")),
                None if !row.installable => (mt("grande demais"), hex("#ff4d6d")),
                None => ("", WHITE),
            };
            self.ui.text(
                label,
                rx + rw - 28.0,
                y + 23.0,
                16.0,
                col,
                Align::Right,
                true,
            );
            if !modal && self.clicked_rect(rx + 8.0, y, rw - 16.0, 32.0) {
                self.market_ui.cur = vi;
                self.market_ui.pane = Pane::Items;
                let k = (row.repo_key.clone(), row.id.clone());
                if !self.market_ui.marked.remove(&k) {
                    self.market_ui.marked.insert(k);
                }
            }
        }
        if total > ROWS {
            let frac = self.market_ui.scroll as f32 / (total - ROWS) as f32;
            self.ui.rect(
                rx + rw - 6.0,
                ry + 52.0,
                4.0,
                ROWS as f32 * 34.0,
                Color::new(1.0, 1.0, 1.0, 0.08),
            );
            self.ui.rect(
                rx + rw - 6.0,
                ry + 52.0 + frac * (ROWS as f32 * 34.0 - 40.0),
                4.0,
                40.0,
                hex("#ffcc33"),
            );
        }

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
            self.ui
                .rect(px, py, rw, 14.0, Color::new(1.0, 1.0, 1.0, 0.1));
            self.ui
                .rect(px, py, rw * frac.clamp(0.0, 1.0), 14.0, hex("#22e8ff"));
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
            self.ui.rect(
                px + rw - 110.0,
                py + 12.0,
                110.0,
                26.0,
                alpha(hex("#ff4d6d"), 0.7),
            );
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
        let mut x = rx;
        let buttons: [(&str, &str, Color, Act); 7] = [
            (mt("Todos"), "M", hex("#5865f2"), Act::MarkAll),
            (mt("Nenhum"), "N", hex("#5865f2"), Act::ClearMarks),
            (mt("Instalar"), "I", hex("#2ecc71"), Act::Install),
            (mt("Ativar"), "E", hex("#22b8cf"), Act::Enable),
            (mt("Desativar"), "O", hex("#f59f00"), Act::Disable),
            (mt("Remover"), "X", hex("#ff4d6d"), Act::Remove),
            (mt("Atualizar"), "R", hex("#8a2be2"), Act::Refresh),
        ];
        for (label, key, col, act) in buttons {
            let w = 114.0;
            if self.market_button(x, by, w, label, key, col) && !modal {
                acts.push(act);
            }
            x += w + 8.0;
        }
        self.ui.rect(40.0, by, 330.0, 48.0, hex("#ff4d6d"));
        self.ui
            .text(mt("Voltar"), 205.0, by + 32.0, 26.0, WHITE, Align::Center, true);
        if self.clicked_rect(40.0, by, 330.0, 48.0) && !modal {
            acts.push(Act::Back);
        }
        self.ui.text(mt("U adicionar · T token · Tab explorar/instalados · F filtro · Espaço marca · Enter/Start aplica"), 40.0, 702.0, 15.0, hex("#a79fc4"), Align::Left, false);

        self.market_entry_modal();
        if !modal {
            if self.c.pressed(nav::DOWN) {
                if self.market_ui.pane == Pane::Items {
                    self.market_ui.cur = (self.market_ui.cur + 1).min(total.saturating_sub(1));
                } else if n_repos > 0 {
                    self.market_ui.repo = (self.market_ui.repo + 1).min(n_repos - 1);
                    self.market_ui.cur = 0;
                    self.market_ui.scroll = 0;
                }
            }
            if self.c.pressed(nav::UP) {
                if self.market_ui.pane == Pane::Items {
                    self.market_ui.cur = self.market_ui.cur.saturating_sub(1);
                } else {
                    self.market_ui.repo = self.market_ui.repo.saturating_sub(1);
                    self.market_ui.cur = 0;
                    self.market_ui.scroll = 0;
                }
            }
            if is_key_pressed(KeyCode::PageDown) {
                self.market_ui.cur = (self.market_ui.cur + ROWS).min(total.saturating_sub(1));
            }
            if is_key_pressed(KeyCode::PageUp) {
                self.market_ui.cur = self.market_ui.cur.saturating_sub(ROWS);
            }
            if self.c.pressed(nav::LEFT) {
                self.market_ui.pane = Pane::Repos;
                self.market_ui.view = View::Browse;
            }
            if self.c.pressed(nav::RIGHT) {
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
                    self.screen = Screen::Menu { cur: 8 };
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
                Act::MarkAll => {
                    for r in &rows {
                        self.market_ui
                            .marked
                            .insert((r.repo_key.clone(), r.id.clone()));
                    }
                }
                Act::ClearMarks => self.market_ui.marked.clear(),
                Act::Install if browse => {
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
                Act::Install => {
                    self.market.notice = mt("Volte para Explorar para baixar novos itens.").into()
                }
                Act::Enable | Act::Disable => {
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
                Act::Remove => {
                    let pairs: Vec<(String, String)> = targets
                        .iter()
                        .filter(|r| r.installed.is_some())
                        .map(|r| (r.repo_key.clone(), r.id.clone()))
                        .collect();
                    for (a, b) in &pairs {
                        self.market.uninstall(a, b);
                    }
                    self.market.notice =
                        format!("{} item(ns) removido(s) deste aparelho.", pairs.len());
                    self.market_ui.marked.clear();
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
