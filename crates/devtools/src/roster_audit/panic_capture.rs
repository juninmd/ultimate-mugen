use std::cell::RefCell;

thread_local! {
    static LAST: RefCell<Option<String>> = const { RefCell::new(None) };
}

pub fn install() {
    std::panic::set_hook(Box::new(|info| {
        let mut parts: Vec<String> = info.to_string().lines().take(2).map(|l| l.trim().to_string()).collect();
        let trace = std::backtrace::Backtrace::force_capture().to_string();
        parts.extend(trace.lines().filter(|l| l.contains("ume_engine") || l.contains("ume-engine")).take(4).map(|l| l.trim().to_string()));
        LAST.with(|slot| *slot.borrow_mut() = Some(parts.join(" <- ")));
    }));
}

pub fn take() -> String {
    LAST.with(|slot| slot.borrow_mut().take()).unwrap_or_else(|| "panic without a message".to_string())
}
