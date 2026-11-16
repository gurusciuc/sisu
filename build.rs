use adblock::{
    Engine,
    lists::{FilterSet, ParseOptions},
};
use sha2::{Digest, Sha256};
use std::{env, fs, path::PathBuf};

fn main() {
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && env::var_os("CARGO_FEATURE_DESKTOP").is_some()
    {
        println!("cargo:rerun-if-changed=native");
        cc::Build::new()
            .cpp(true)
            .flag_if_supported("/utf-8")
            .file("native/shell.cpp")
            .flag_if_supported("/std:c++17")
            .compile("win_shell");
    }
    let mut rules = FilterSet::new(false);
    for (name, digest) in [
        (
            "easylist.txt",
            "ff795ed6920935ee66e56e014e95e62e14e17ed6fcd186b43dba0feed2d5f26d",
        ),
        (
            "easyprivacy.txt",
            "042fefa0afb7ce2e61fe7f25ead218a98106725c3a578f724cd81a49ed2efba8",
        ),
        ("probe.txt", ""),
    ] {
        let path = format!("rules/{name}");
        println!("cargo:rerun-if-changed={path}");
        assert!(fs::metadata(&path).unwrap().len() <= 8 * 1024 * 1024);
        let input = fs::read(&path).unwrap();
        if !digest.is_empty() {
            assert_eq!(
                format!("{:x}", Sha256::digest(&input)),
                digest,
                "{name} integrity check"
            );
        }
        rules.add_filter_list(String::from_utf8(input).unwrap(), ParseOptions::default());
    }
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    fs::write(
        out.join("rules.bin"),
        Engine::new_with_filter_set(rules).serialize(),
    )
    .unwrap();
}
