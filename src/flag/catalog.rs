pub(crate) fn codes() -> impl Iterator<Item = &'static str> {
    super::designs::all().map(|(code, _)| code)
}
