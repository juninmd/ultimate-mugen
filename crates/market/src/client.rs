use crate::catalog::{parse_tree, Tree};
use crate::repo::RepoRef;
use crate::safety::{percent_encode_path, url_allowed, MAX_FILE_BYTES, MAX_TREE_BODY_BYTES};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    pub url: String,
    pub etag: Option<String>,
    pub token: Option<String>,
    pub max_bytes: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RateLimit {
    pub remaining: u32,
    pub reset_epoch: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Response {
    pub status: u16,
    pub etag: Option<String>,
    pub body: Vec<u8>,
    pub rate: Option<RateLimit>,
    pub retry_after_secs: Option<u64>,
    pub location: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MarketError {
    Network(String),
    NotFound,
    RateLimited { reset_epoch: Option<u64> },
    Forbidden,
    Http(u16),
    TooLarge,
    Invalid(String),
    HostNotAllowed,
    Cancelled,
}

impl std::fmt::Display for MarketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarketError::Network(e) => write!(f, "network error: {e}"),
            MarketError::NotFound => write!(f, "repository or file not found (private repositories are not supported)"),
            MarketError::RateLimited { reset_epoch: Some(t) } => write!(f, "GitHub rate limit reached, resets at unix time {t} (add a token to raise it)"),
            MarketError::RateLimited { reset_epoch: None } => write!(f, "GitHub rate limit reached, try again later (add a token to raise it)"),
            MarketError::Forbidden => write!(f, "access denied by GitHub"),
            MarketError::Http(c) => write!(f, "unexpected HTTP status {c}"),
            MarketError::TooLarge => write!(f, "download exceeds the size limit"),
            MarketError::Invalid(e) => write!(f, "{e}"),
            MarketError::HostNotAllowed => write!(f, "blocked: only github.com hosts are allowed"),
            MarketError::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl std::error::Error for MarketError {}

pub fn tree_request(repo: &RepoRef, etag: Option<&str>, token: Option<&str>) -> Request {
    Request {
        url: format!("https://api.github.com/repos/{}/{}/git/trees/{}?recursive=1", repo.owner, repo.name, percent_encode_path(repo.git_ref())),
        etag: etag.map(str::to_string),
        token: token.map(str::to_string),
        max_bytes: MAX_TREE_BODY_BYTES,
    }
}

pub fn file_request(repo: &RepoRef, path: &str, max_bytes: u64) -> Request {
    Request {
        url: format!("https://raw.githubusercontent.com/{}/{}/{}/{}", repo.owner, repo.name, percent_encode_path(repo.git_ref()), percent_encode_path(path)),
        etag: None,
        token: None,
        max_bytes: max_bytes.min(MAX_FILE_BYTES),
    }
}

pub fn check_request(req: &Request) -> Result<(), MarketError> {
    if url_allowed(&req.url) { Ok(()) } else { Err(MarketError::HostNotAllowed) }
}

pub fn redirect_target(from: &str, location: &str) -> Result<String, MarketError> {
    let next = if location.starts_with("https://") {
        location.to_string()
    } else if location.starts_with('/') && !location.starts_with("//") {
        let host = crate::safety::host_of(from).ok_or(MarketError::HostNotAllowed)?;
        format!("https://{host}{location}")
    } else {
        return Err(MarketError::HostNotAllowed);
    };
    if url_allowed(&next) { Ok(next) } else { Err(MarketError::HostNotAllowed) }
}

pub fn status_error(resp: &Response) -> MarketError {
    match resp.status {
        404 | 410 => MarketError::NotFound,
        429 => MarketError::RateLimited { reset_epoch: resp.rate.map(|r| r.reset_epoch) },
        403 if resp.rate.is_some_and(|r| r.remaining == 0) || resp.retry_after_secs.is_some() => MarketError::RateLimited { reset_epoch: resp.rate.map(|r| r.reset_epoch) },
        401 | 403 => MarketError::Forbidden,
        s => MarketError::Http(s),
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum TreeOutcome {
    NotModified,
    Fresh { tree: Tree, etag: Option<String> },
}

pub fn interpret_tree(resp: &Response) -> Result<TreeOutcome, MarketError> {
    match resp.status {
        304 => Ok(TreeOutcome::NotModified),
        200 => {
            let body = std::str::from_utf8(&resp.body).map_err(|_| MarketError::Invalid("tree response is not text".into()))?;
            let tree = parse_tree(body).map_err(MarketError::Invalid)?;
            Ok(TreeOutcome::Fresh { tree, etag: resp.etag.clone() })
        }
        _ => Err(status_error(resp)),
    }
}

pub fn interpret_file(resp: &Response, expected_max: u64) -> Result<Vec<u8>, MarketError> {
    match resp.status {
        200 if resp.body.len() as u64 <= expected_max.min(MAX_FILE_BYTES) => Ok(resp.body.clone()),
        200 => Err(MarketError::TooLarge),
        _ => Err(status_error(resp)),
    }
}

pub fn is_transient(e: &MarketError) -> bool {
    matches!(e, MarketError::Network(_) | MarketError::Http(500..=599))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo() -> RepoRef {
        RepoRef::parse("a/b").unwrap()
    }

    fn resp(status: u16) -> Response {
        Response { status, ..Default::default() }
    }

    #[test]
    fn builds_github_urls_with_encoded_paths() {
        let r = tree_request(&repo(), Some("\"abc\""), Some("tok"));
        assert_eq!(r.url, "https://api.github.com/repos/a/b/git/trees/HEAD?recursive=1");
        assert_eq!((r.etag.as_deref(), r.token.as_deref()), (Some("\"abc\""), Some("tok")));
        let f = file_request(&RepoRef::parse("a/b/tree/dev/x").unwrap(), "chars/My Char/a#b.def", 10);
        assert_eq!(f.url, "https://raw.githubusercontent.com/a/b/dev/chars/My%20Char/a%23b.def");
        assert!(f.token.is_none());
        assert!(check_request(&f).is_ok());
        assert!(check_request(&Request { url: "https://evil.example/x".into(), etag: None, token: None, max_bytes: 1 }).is_err());
    }

    #[test]
    fn redirects_are_only_followed_inside_the_allowlist() {
        assert_eq!(redirect_target("https://raw.githubusercontent.com/a", "/b/c").unwrap(), "https://raw.githubusercontent.com/b/c");
        assert!(redirect_target("https://raw.githubusercontent.com/a", "https://media.githubusercontent.com/x").is_ok());
        assert_eq!(redirect_target("https://raw.githubusercontent.com/a", "https://evil.com/x"), Err(MarketError::HostNotAllowed));
        assert_eq!(redirect_target("https://raw.githubusercontent.com/a", "http://github.com/x"), Err(MarketError::HostNotAllowed));
        assert_eq!(redirect_target("https://raw.githubusercontent.com/a", "//evil.com/x"), Err(MarketError::HostNotAllowed));
    }

    #[test]
    fn maps_statuses_to_errors() {
        assert_eq!(status_error(&resp(404)), MarketError::NotFound);
        assert_eq!(status_error(&resp(403)), MarketError::Forbidden);
        let limited = Response { status: 403, rate: Some(RateLimit { remaining: 0, reset_epoch: 99 }), ..Default::default() };
        assert_eq!(status_error(&limited), MarketError::RateLimited { reset_epoch: Some(99) });
        assert_eq!(status_error(&resp(429)), MarketError::RateLimited { reset_epoch: None });
        assert_eq!(status_error(&resp(502)), MarketError::Http(502));
        assert!(is_transient(&MarketError::Http(503)) && is_transient(&MarketError::Network("x".into())) && !is_transient(&MarketError::NotFound));
    }

    #[test]
    fn tree_response_interpretation() {
        assert_eq!(interpret_tree(&resp(304)), Ok(TreeOutcome::NotModified));
        let ok = Response { status: 200, etag: Some("W/\"1\"".into()), body: br#"{"tree":[{"path":"a.ogg","type":"blob","size":3}]}"#.to_vec(), ..Default::default() };
        match interpret_tree(&ok).unwrap() {
            TreeOutcome::Fresh { tree, etag } => assert_eq!((tree.entries.len(), etag.as_deref()), (1, Some("W/\"1\""))),
            other => panic!("{other:?}"),
        }
        assert!(matches!(interpret_tree(&Response { status: 200, body: b"nope".to_vec(), ..Default::default() }), Err(MarketError::Invalid(_))));
    }

    #[test]
    fn file_size_limit_is_enforced_on_the_body() {
        let big = Response { status: 200, body: vec![0; 11], ..Default::default() };
        assert_eq!(interpret_file(&big, 10), Err(MarketError::TooLarge));
        assert_eq!(interpret_file(&Response { status: 200, body: vec![1; 3], ..Default::default() }, 10), Ok(vec![1, 1, 1]));
    }
}
