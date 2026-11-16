use super::*;
use crate::cookies::{self, CookieRecord, SameSite};
use std::{
    collections::VecDeque,
    io::Read,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};

pub(super) static ACTIVE: AtomicBool = AtomicBool::new(false);
struct Batch {
    manager: CookieManager,
    queue: Mutex<VecDeque<CookieRecord>>,
    written: AtomicUsize,
    failed: AtomicUsize,
    expired: usize,
}

fn finish(batch: &Batch, flushed: bool) {
    ACTIVE.store(false, Ordering::Release);
    if BROWSERS.with_borrow(|b| b.is_empty()) {
        quit_message_loop();
        return;
    }
    notice(
        "Cookie import",
        &format!(
            "{} imported · {} rejected · {} expired skipped.{}",
            batch.written.load(Ordering::Relaxed),
            batch.failed.load(Ordering::Relaxed),
            batch.expired,
            if flushed {
                ""
            } else {
                " The store could not be flushed. Changes may not survive a restart."
            }
        ),
    );
}
wrap_completion_callback! {
    struct Flushed { batch: Arc<Batch> }
    impl CompletionCallback {
        fn on_complete(&self) { finish(&self.batch,true); }
    }
}
wrap_set_cookie_callback! {
    struct Written { batch: Arc<Batch> }
    impl SetCookieCallback {
        fn on_complete(&self, success:i32) {
            if success!=0 {self.batch.written.fetch_add(1,Ordering::Relaxed);}
            else {self.batch.failed.fetch_add(1,Ordering::Relaxed);}
            advance(self.batch.clone());
        }
    }
}
fn advance(batch: Arc<Batch>) {
    loop {
        let next = batch.queue.lock().unwrap().pop_front();
        let Some(record) = next else {
            if batch
                .manager
                .flush_store(Some(&mut Flushed::new(batch.clone())))
                == 0
            {
                finish(&batch, false);
            }
            return;
        };
        let cookie = Cookie {
            name: string(&record.name),
            value: string(&record.value),
            domain: if record.host_only {
                CefString::default()
            } else {
                string(&format!(".{}", record.host))
            },
            path: string(&record.path),
            secure: record.secure.into(),
            httponly: record.http_only.into(),
            has_expires: record.expires.is_some().into(),
            expires: Basetime {
                val: record
                    .expires
                    .map(|s| (s + 11_644_473_600) * 1_000_000)
                    .unwrap_or(0),
            },
            same_site: match record.same_site {
                SameSite::Unspecified => CookieSameSite::UNSPECIFIED,
                SameSite::None => CookieSameSite::NO_RESTRICTION,
                SameSite::Lax => CookieSameSite::LAX_MODE,
                SameSite::Strict => CookieSameSite::STRICT_MODE,
            },
            priority: CookiePriority::MEDIUM,
            ..Default::default()
        };
        let origin = format!(
            "{}://{}/",
            if record.secure { "https" } else { "http" },
            record.host
        );
        if batch.manager.set_cookie(
            Some(&string(&origin)),
            Some(&cookie),
            Some(&mut Written::new(batch.clone())),
        ) != 0
        {
            return;
        }
        batch.failed.fetch_add(1, Ordering::Relaxed);
    }
}
pub(super) fn run(session: &Session, context: &RequestContext) -> Result<(), String> {
    if session.private {
        return Err("Open an ordinary window to import cookies.".into());
    }
    if ACTIVE.load(Ordering::Acquire) {
        return Err("A cookie import is already running.".into());
    }
    let mut path = vec![0u16; 32768];
    if !unsafe { shell_pick_file(path.as_mut_ptr(), path.len() as i32, 1) } {
        return Ok(());
    }
    let end = path.iter().position(|&c| c == 0).unwrap_or(path.len());
    let file = std::fs::File::open(PathBuf::from(OsString::from_wide(&path[..end])))
        .map_err(|e| e.to_string())?;
    let mut bytes = vec![];
    file.take((cookies::MAX_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs() as i64;
    let export = cookies::parse_export(&bytes, now)?;
    let manager = context
        .cookie_manager(None)
        .ok_or("The profile's cookie store is unavailable.")?;
    if ACTIVE.swap(true, Ordering::AcqRel) {
        return Err("A cookie import is already running.".into());
    }
    advance(Arc::new(Batch {
        manager,
        queue: Mutex::new(export.cookies.into()),
        written: AtomicUsize::new(0),
        failed: AtomicUsize::new(0),
        expired: export.expired,
    }));
    Ok(())
}
