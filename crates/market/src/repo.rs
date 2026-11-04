use crate::safety::{safe_subdir, valid_owner, valid_ref, valid_repo_name};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepoRef {
    pub owner: String,
    pub name: String,
    pub reference: Option<String>,
    pub subdir: String,
}

impl RepoRef {
    pub fn parse(input: &str) -> Result<RepoRef, String> {
        let raw = input.trim().trim_matches(|c| c == '"' || c == '\'' || c == '<' || c == '>');
        if raw.is_empty() {
            return Err("empty repository link".into());
        }
        let path = if let Some(rest) = raw.strip_prefix("git@github.com:") {
            rest.to_string()
        } else if let Some(rest) = raw.strip_prefix("https://") {
            host_path(rest)?
        } else if raw.starts_with("http://") {
            return Err("only https links are accepted".into());
        } else if raw.contains("://") {
            return Err("only github.com links are accepted".into());
        } else if raw.split('/').next().is_some_and(|h| h.contains('.') && !valid_owner(h)) {
            host_path(raw)?
        } else {
            raw.to_string()
        };
        let path = path.split(['?', '#']).next().unwrap_or("").trim_matches('/').to_string();
        let mut parts = path.split('/');
        let owner = parts.next().unwrap_or("");
        let name = parts.next().unwrap_or("");
        let name = name.strip_suffix(".git").unwrap_or(name);
        if !valid_owner(owner) {
            return Err(format!("invalid owner: {owner:?}"));
        }
        if !valid_repo_name(name) {
            return Err(format!("invalid repository name: {name:?}"));
        }
        let mut reference = None;
        let mut subdir = String::new();
        match parts.next() {
            None | Some("") => {}
            Some("tree" | "blob") => {
                let r = parts.next().ok_or("missing branch after /tree/")?;
                if !valid_ref(r) {
                    return Err(format!("invalid branch: {r:?}"));
                }
                reference = Some(r.to_string());
                let rest: Vec<&str> = parts.collect();
                subdir = safe_subdir(&rest.join("/")).ok_or("invalid folder in link")?;
            }
            Some(other) => return Err(format!("unsupported link section: {other:?}")),
        }
        Ok(RepoRef { owner: owner.to_string(), name: name.to_string(), reference, subdir })
    }

    pub fn key(&self) -> String {
        let mut k = format!("{}/{}", self.owner.to_ascii_lowercase(), self.name.to_ascii_lowercase());
        if let Some(r) = &self.reference {
            k.push('@');
            k.push_str(r);
        }
        if !self.subdir.is_empty() {
            k.push(':');
            k.push_str(&self.subdir);
        }
        k
    }

    pub fn display(&self) -> String {
        let mut s = format!("{}/{}", self.owner, self.name);
        if let Some(r) = &self.reference {
            s.push('@');
            s.push_str(r);
        }
        if !self.subdir.is_empty() {
            s.push('/');
            s.push_str(&self.subdir);
        }
        s
    }

    pub fn git_ref(&self) -> &str {
        self.reference.as_deref().unwrap_or("HEAD")
    }

    pub fn web_url(&self) -> String {
        format!("https://github.com/{}/{}", self.owner, self.name)
    }
}

fn host_path(rest: &str) -> Result<String, String> {
    let (host, path) = rest.split_once('/').ok_or("missing repository path")?;
    if host.eq_ignore_ascii_case("github.com") || host.eq_ignore_ascii_case("www.github.com") {
        Ok(path.to_string())
    } else {
        Err("only github.com repositories are supported".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> RepoRef {
        RepoRef::parse(s).unwrap_or_else(|e| panic!("{s}: {e}"))
    }

    #[test]
    fn accepts_common_link_shapes() {
        let base = RepoRef { owner: "juninmd".into(), name: "ultimate-mugen".into(), reference: None, subdir: String::new() };
        for s in ["juninmd/ultimate-mugen", "https://github.com/juninmd/ultimate-mugen", "https://github.com/juninmd/ultimate-mugen/", "https://github.com/juninmd/ultimate-mugen.git", "github.com/juninmd/ultimate-mugen", "git@github.com:juninmd/ultimate-mugen.git", "  https://www.github.com/juninmd/ultimate-mugen?tab=readme#x ", "<https://github.com/juninmd/ultimate-mugen>"] {
            assert_eq!(p(s), base, "{s}");
        }
    }

    #[test]
    fn tree_links_carry_branch_and_folder() {
        let r = p("https://github.com/a/b/tree/dev/mugen/chars");
        assert_eq!((r.reference.as_deref(), r.subdir.as_str()), (Some("dev"), "mugen/chars"));
        assert_eq!(r.key(), "a/b@dev:mugen/chars");
        assert_eq!(r.display(), "a/b@dev/mugen/chars");
        assert_eq!(p("a/b").git_ref(), "HEAD");
    }

    #[test]
    fn rejects_foreign_hosts_and_hostile_input() {
        for bad in ["", "https://gitlab.com/a/b", "http://github.com/a/b", "ftp://github.com/a/b", "https://github.com/a", "https://github.com/a/b/tree/../x", "https://github.com/a/b/tree/main/../../x", "a/b/issues", "https://evil.com/github.com/a/b", "-a/b", "a/b c", "javascript:alert(1)"] {
            assert!(RepoRef::parse(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn key_is_case_insensitive() {
        assert_eq!(p("Foo/Bar").key(), p("foo/bar").key());
    }
}
