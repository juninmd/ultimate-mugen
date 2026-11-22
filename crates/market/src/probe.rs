use crate::catalog::{FileRef, TreeEntry};
use crate::client::{interpret_range, range_request, MarketError};
use crate::native::Transport;
use crate::repo::RepoRef;
use crate::zipdir::{from_directory, from_tail, tail_range, Step};

fn fetch(t: &dyn Transport, repo: &RepoRef, f: &FileRef, start: u64, end: u64) -> Result<Vec<u8>, MarketError> {
    let req = range_request(repo, &f.path, &f.sha, start, end);
    let mut last = MarketError::Cancelled;
    for _ in 0..2 {
        match t.get(&req) {
            Ok(resp) => return interpret_range(&resp, start, end),
            Err(e) => last = e,
        }
    }
    Err(last)
}

pub fn probe_zip(t: &dyn Transport, repo: &RepoRef, f: &FileRef) -> Result<Vec<TreeEntry>, MarketError> {
    let (start, end) = tail_range(f.size).ok_or_else(|| MarketError::Invalid("not a zip".into()))?;
    let tail = fetch(t, repo, f, start, end)?;
    match from_tail(&tail, start).map_err(MarketError::Invalid)? {
        Step::Done(entries) => Ok(entries),
        Step::Fetch { start, end, count } => from_directory(&fetch(t, repo, f, start, end)?, count).map_err(MarketError::Invalid),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::{Request, Response};

    struct Fake(Vec<u8>);

    impl Transport for Fake {
        fn get(&self, req: &Request) -> Result<Response, MarketError> {
            let (a, b) = req.range.expect("probe must only issue ranged requests");
            Ok(Response { status: 206, body: self.0[a as usize..=b as usize].to_vec(), ..Default::default() })
        }
    }

    #[test]
    fn probe_reads_only_the_tail_and_the_directory_of_a_big_zip() {
        let mut z = vec![0u8; 300_000];
        let dir = z.len();
        for name in ["a/a.def", "a/a.sff"] {
            z.extend_from_slice(&[0x50, 0x4b, 0x01, 0x02]);
            let mut head = [0u8; 42];
            head[24..26].copy_from_slice(&(name.len() as u16).to_le_bytes());
            z.extend_from_slice(&head);
            z.extend_from_slice(name.as_bytes());
        }
        let dir_len = z.len() - dir;
        z.extend(vec![0u8; 100_000]);
        z.extend_from_slice(&[0x50, 0x4b, 0x05, 0x06]);
        let mut rec = [0u8; 18];
        rec[6..8].copy_from_slice(&2u16.to_le_bytes());
        rec[8..12].copy_from_slice(&(dir_len as u32).to_le_bytes());
        rec[12..16].copy_from_slice(&(dir as u32).to_le_bytes());
        z.extend_from_slice(&rec);
        let file = FileRef { path: "chars/a.zip".into(), size: z.len() as u64, sha: "ab".into() };
        let entries = probe_zip(&Fake(z), &RepoRef::parse("a/b").unwrap(), &file).unwrap();
        assert_eq!(entries.iter().map(|e| e.path.as_str()).collect::<Vec<_>>(), vec!["a/a.def", "a/a.sff"]);
    }
}
