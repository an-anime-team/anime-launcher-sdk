#![cfg(all(feature = "zzz", feature = "config", feature = "sessions"))]

use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};

use anime_launcher_sdk::sessions::SessionsExt;
use anime_launcher_sdk::zzz::sessions::Sessions;

static LAUNCHER_FOLDER_LOCK: Mutex<()> = Mutex::new(());

const GLOBAL_KEY: &str = r"Software\\miHoYo\\ZenlessZoneZero";
// Wine escapes the Chinese product name in user.reg.
const CHINA_KEY: &str = r"Software\\miHoYo\\\x7edd\x533a\x96f6";
const SDK_KEY: &str = r"Software\\miHoYoSDK";

struct TestPrefix {
    folder: PathBuf,
    previous_launcher_folder: Option<OsString>,
    _lock: MutexGuard<'static, ()>,
}

impl TestPrefix {
    fn new() -> Self {
        let lock = LAUNCHER_FOLDER_LOCK
            .lock()
            .unwrap_or_else(|err| err.into_inner());
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let folder =
            std::env::temp_dir().join(format!("zzz-sessions-{}-{nonce}", std::process::id()));
        std::fs::create_dir(&folder).unwrap();
        let previous_launcher_folder = std::env::var_os("LAUNCHER_FOLDER");
        std::env::set_var("LAUNCHER_FOLDER", &folder);

        Self {
            folder,
            previous_launcher_folder,
            _lock: lock,
        }
    }

    fn write_registry(&self, sections: &[String]) {
        std::fs::write(
            self.folder.join("user.reg"),
            format!("WINE REGISTRY Version 2\n\n{}\n", sections.join("\n\n")),
        )
        .unwrap();
    }

    fn registry(&self) -> String {
        std::fs::read_to_string(self.folder.join("user.reg")).unwrap()
    }
}

impl Drop for TestPrefix {
    fn drop(&mut self) {
        match &self.previous_launcher_folder {
            Some(value) => std::env::set_var("LAUNCHER_FOLDER", value),
            None => std::env::remove_var("LAUNCHER_FOLDER"),
        }
        let _ = std::fs::remove_dir_all(&self.folder);
    }
}

fn section(key: &str, account: &str) -> String {
    format!("[{key}] 1234567890\n\"account\"=\"{account}\"")
}

#[test]
fn china_sessions_restore_game_and_sdk_registry() {
    let prefix = TestPrefix::new();
    let china = section(CHINA_KEY, "saved-cn");
    let sdk = section(SDK_KEY, "saved-sdk");
    let unrelated = section(r"Software\\Unrelated", "keep");
    prefix.write_registry(&[china.clone(), sdk.clone(), unrelated.clone()]);
    Sessions::update("cn".into(), &prefix.folder).unwrap();

    prefix.write_registry(&[
        section(CHINA_KEY, "other-cn"),
        section(SDK_KEY, "other-sdk"),
        unrelated.clone(),
    ]);
    Sessions::apply("cn".into(), &prefix.folder).unwrap();

    let registry = prefix.registry();
    assert!(
        registry.contains(&china),
        "the China account section must be restored"
    );
    assert!(registry.contains(&sdk));
    assert!(registry.contains(&unrelated));
    assert!(!registry.contains("other-cn"));
}

#[test]
fn global_sessions_remain_compatible() {
    let prefix = TestPrefix::new();
    let global = section(GLOBAL_KEY, "saved-global");
    let sdk = section(SDK_KEY, "saved-sdk");
    prefix.write_registry(&[global.clone(), sdk.clone()]);
    Sessions::update("global".into(), &prefix.folder).unwrap();
    prefix.write_registry(&[
        section(GLOBAL_KEY, "other-global"),
        section(SDK_KEY, "other-sdk"),
    ]);
    Sessions::apply("global".into(), &prefix.folder).unwrap();

    let registry = prefix.registry();
    assert!(registry.contains(&global));
    assert!(registry.contains(&sdk));
}

#[test]
fn mixed_edition_prefixes_restore_both_game_sections() {
    let prefix = TestPrefix::new();
    let global = section(GLOBAL_KEY, "saved-global");
    let china = section(CHINA_KEY, "saved-cn");
    let sdk = section(SDK_KEY, "saved-sdk");
    prefix.write_registry(&[global.clone(), china.clone(), sdk.clone()]);
    Sessions::update("both".into(), &prefix.folder).unwrap();
    prefix.write_registry(&[
        section(GLOBAL_KEY, "other-global"),
        section(CHINA_KEY, "other-cn"),
        section(SDK_KEY, "other-sdk"),
    ]);
    Sessions::apply("both".into(), &prefix.folder).unwrap();

    let registry = prefix.registry();
    assert!(registry.contains(&global));
    assert!(registry.contains(&china));
    assert!(registry.contains(&sdk));
}

#[test]
fn legacy_global_sessions_preserve_china_settings() {
    let prefix = TestPrefix::new();
    let global = section(GLOBAL_KEY, "saved-global");
    let china = section(CHINA_KEY, "keep-cn");
    let sdk = section(SDK_KEY, "saved-sdk");
    std::fs::write(
        prefix.folder.join("sessions.json"),
        serde_json::to_vec(&serde_json::json!({
            "current": null,
            "sessions": {"legacy": {"game_reg": global, "sdk_reg": sdk}}
        }))
        .unwrap(),
    )
    .unwrap();
    prefix.write_registry(&[
        section(GLOBAL_KEY, "other-global"),
        china.clone(),
        section(SDK_KEY, "other-sdk"),
    ]);
    Sessions::apply("legacy".into(), &prefix.folder).unwrap();

    let registry = prefix.registry();
    assert!(registry.contains(&global));
    assert!(registry.contains(&china));
    assert!(registry.contains(&sdk));
}

#[test]
fn china_sessions_do_not_erase_unsaved_global_settings() {
    let prefix = TestPrefix::new();
    let china = section(CHINA_KEY, "saved-cn");
    let global = section(GLOBAL_KEY, "keep-global");
    prefix.write_registry(&[china.clone(), section(SDK_KEY, "saved-sdk")]);
    Sessions::update("cn-only".into(), &prefix.folder).unwrap();
    prefix.write_registry(&[
        global.clone(),
        section(CHINA_KEY, "other-cn"),
        section(SDK_KEY, "other-sdk"),
    ]);
    Sessions::apply("cn-only".into(), &prefix.folder).unwrap();

    let registry = prefix.registry();
    assert!(
        registry.contains(&global),
        "an unsaved edition must not be erased"
    );
    assert!(registry.contains(&china));
}
