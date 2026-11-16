use super::*;
use crate::bookmarks::{self, Bookmark};
use std::{fs::File, io::Read};
const DB: &str = "bookmarks.json";
fn limited(path: &std::path::Path) -> Result<Vec<u8>, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let mut data = vec![];
    file.take((bookmarks::MAX_BYTES + 1) as u64)
        .read_to_end(&mut data)
        .map_err(|e| e.to_string())?;
    if data.len() > bookmarks::MAX_BYTES {
        return Err("This file exceeds 8 MiB.".into());
    }
    Ok(data)
}
fn library(s: &Session) -> Result<Vec<Bookmark>, String> {
    let path = s.home.join(DB);
    if !path.try_exists().map_err(|e| e.to_string())? {
        return Ok(vec![]);
    }
    let list: Vec<Bookmark> = serde_json::from_slice(&limited(&path)?).map_err(
        |_| "The bookmark file could not be read. Your existing file has been preserved.",
    )?;
    if list.len() > bookmarks::LIMIT
        || list
            .iter()
            .any(|b| Bookmark::new(&b.title, &b.url).is_none())
    {
        return Err("Invalid bookmark library.".into());
    }
    Ok(list)
}
fn store(s: &Session, books: &[Bookmark]) -> Result<(), String> {
    let bytes = serde_json::to_vec(books).map_err(|e| e.to_string())?;
    if bytes.len() > bookmarks::MAX_BYTES || books.len() > bookmarks::LIMIT {
        return Err("The bookmark library is full.".into());
    }
    let mut file = tempfile::NamedTempFile::new_in(&s.home).map_err(|e| e.to_string())?;
    file.write_all(&bytes)
        .and_then(|_| file.as_file().sync_all())
        .map_err(|e| e.to_string())?;
    file.persist(s.home.join(DB)).map_err(|e| e.to_string())?;
    Ok(())
}
pub(super) fn action(command: i32, s: &Session, b: Option<&Browser>) -> Result<(), String> {
    if s.private {
        return Err(
            "Bookmarks belong to the everyday profile. Open an ordinary window to manage them."
                .into(),
        );
    }
    let mut records = library(s)?;
    match command {
        20 => loop {
            let names: Vec<_> = records
                .iter()
                .map(|b| {
                    wide(&if b.folders.is_empty() {
                        b.title.clone()
                    } else {
                        format!("{}  /  {}", b.folders.join("  /  "), b.title)
                    })
                })
                .collect();
            let pointers: Vec<_> = names.iter().map(|s| s.as_ptr()).collect();
            let mut index = -1;
            let result =
                unsafe { shell_library(pointers.as_ptr(), records.len() as i32, &mut index) };
            let Some(item) = records.get(index as usize) else {
                break;
            };
            if result == 1 {
                new_page(&item.url);
                break;
            }
            if result != 2 {
                break;
            }
            records.remove(index as usize);
            store(s, &records)?;
        },
        21 => {
            let frame = b
                .and_then(|b| b.main_frame())
                .ok_or("Open a website to bookmark it.")?;
            let url = CefString::from(&frame.url()).to_string();
            let title = b
                .and_then(|b| b.host())
                .and_then(|h| h.visible_navigation_entry())
                .map(|e| CefString::from(&e.title()).to_string())
                .unwrap_or_default();
            let entry =
                Bookmark::new(&title, &url).ok_or("Open an HTTP or HTTPS page to bookmark it.")?;
            if records.iter().any(|b| b.url == entry.url) {
                notice("Bookmarks", "This page is already saved.");
            } else {
                records.push(entry);
                store(s, &records)?;
                notice("Bookmarks", "Page saved to Bookmarks.");
            }
        }
        22 => {
            let mut buf = vec![0u16; 32768];
            if !unsafe { shell_pick_file(buf.as_mut_ptr(), buf.len() as i32) } {
                return Ok(());
            }
            let file = PathBuf::from(OsString::from_wide(
                &buf[..buf.iter().position(|&v| v == 0).unwrap_or(buf.len())],
            ));
            let result = bookmarks::merge_export(&mut records, &limited(&file)?)?;
            store(s, &records)?;
            notice(
                "Bookmark import",
                &format!(
                    "{} added · {} duplicates · {} unsupported links skipped.",
                    result.added, result.duplicates, result.skipped
                ),
            );
        }
        _ => {}
    }
    Ok(())
}
