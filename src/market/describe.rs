use ume_market::MarketError;

pub fn describe(e: &MarketError) -> String {
    match e {
        MarketError::Network(m) => format!("{}: {m}", crate::i18n::tr3("Falha de rede", "Network error", "Error de red")),
        MarketError::NotFound => {
            crate::i18n::tr3("Repositório ou arquivo não encontrado (repositórios privados não são suportados).", "Repository or file not found (private repositories are not supported).", "Repositorio o archivo no encontrado (los repositorios privados no son compatibles).").into()
        }
        MarketError::RateLimited {
            reset_epoch: Some(t),
        } => {
            let wait = t.saturating_sub(ume_market::now_secs()).div_ceil(60);
            crate::i18n::fill(crate::i18n::tr3("Limite de requisições do GitHub atingido; tente em ~{wait} min ou informe um token.", "GitHub rate limit reached; try again in ~{wait} min or provide a token.", "Límite de peticiones de GitHub alcanzado; prueba en ~{wait} min o indica un token."), &[("wait", &wait.to_string())])
        }
        MarketError::RateLimited { reset_epoch: None } => {
            crate::i18n::tr3("Limite de requisições do GitHub atingido; tente mais tarde ou informe um token.", "GitHub rate limit reached; try again later or provide a token.", "Límite de peticiones de GitHub alcanzado; prueba más tarde o indica un token.").into()
        }
        MarketError::Forbidden => crate::i18n::tr3("Acesso negado pelo GitHub.", "Access denied by GitHub.", "Acceso denegado por GitHub.").into(),
        MarketError::Http(c) => format!("{} (HTTP {c}).", crate::i18n::tr3("Resposta inesperada do GitHub", "Unexpected GitHub response", "Respuesta inesperada de GitHub")),
        MarketError::TooLarge => crate::i18n::tr3("Download maior que o limite permitido.", "Download exceeds the allowed limit.", "La descarga supera el límite permitido.").into(),
        MarketError::Invalid(m) => m.clone(),
        MarketError::HostNotAllowed => crate::i18n::tr3("Bloqueado: só é permitido baixar de github.com.", "Blocked: downloads are only allowed from github.com.", "Bloqueado: solo se permite descargar de github.com.").into(),
        MarketError::Cancelled => "Cancelado.".into(),
    }
}
