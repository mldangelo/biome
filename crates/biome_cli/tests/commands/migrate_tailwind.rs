use crate::run_cli;
use crate::snap_test::{SnapshotPayload, assert_cli_snapshot};
use biome_console::BufferConsole;
use biome_fs::MemoryFileSystem;
use bpaf::Args;
use camino::Utf8Path;

#[test]
fn tailwind_migrate() {
    let fs = MemoryFileSystem::default();
    let mut console = BufferConsole::default();

    let configuration = r#"{ "linter": { "enabled": true } }"#;
    let tailwind = r#"
export default {
    theme: {
        extend: {
            spacing: {
                "13": "3.25rem",
                "18": "4.5rem"
            },
            opacity: {
                "15": "0.15"
            }
        }
    }
}
"#;

    let configuration_path = Utf8Path::new("biome.json");
    fs.insert(configuration_path.into(), configuration.as_bytes());

    let tailwind_path = Utf8Path::new("tailwind.config.js");
    fs.insert(tailwind_path.into(), tailwind.as_bytes());

    let (fs, result) = run_cli(
        fs,
        &mut console,
        Args::from(["migrate", "tailwind"].as_slice()),
    );

    assert!(result.is_ok(), "run_cli returned {result:?}");

    assert_cli_snapshot(SnapshotPayload::new(
        module_path!(),
        "tailwind_migrate",
        fs,
        console,
        result,
    ));
}

#[test]
fn tailwind_migrate_write() {
    let fs = MemoryFileSystem::default();
    let mut console = BufferConsole::default();

    let configuration = r#"{ "linter": { "enabled": true } }"#;
    let tailwind = r#"
export default {
    theme: {
        extend: {
            spacing: {
                "13": "3.25rem"
            }
        }
    }
}
"#;

    let configuration_path = Utf8Path::new("biome.json");
    fs.insert(configuration_path.into(), configuration.as_bytes());

    let tailwind_path = Utf8Path::new("tailwind.config.js");
    fs.insert(tailwind_path.into(), tailwind.as_bytes());

    let (fs, result) = run_cli(
        fs,
        &mut console,
        Args::from(["migrate", "tailwind", "--write"].as_slice()),
    );

    assert!(result.is_ok(), "run_cli returned {result:?}");

    assert_cli_snapshot(SnapshotPayload::new(
        module_path!(),
        "tailwind_migrate_write",
        fs,
        console,
        result,
    ));
}

#[test]
fn tailwind_migrate_no_file() {
    let fs = MemoryFileSystem::default();
    let mut console = BufferConsole::default();

    let configuration = r#"{ "linter": { "enabled": true } }"#;

    let configuration_path = Utf8Path::new("biome.json");
    fs.insert(configuration_path.into(), configuration.as_bytes());

    let (fs, result) = run_cli(
        fs,
        &mut console,
        Args::from(["migrate", "tailwind"].as_slice()),
    );

    assert!(result.is_err(), "run_cli returned {result:?}");

    assert_cli_snapshot(SnapshotPayload::new(
        module_path!(),
        "tailwind_migrate_no_file",
        fs,
        console,
        result,
    ));
}

#[test]
fn tailwind_migrate_ts() {
    let fs = MemoryFileSystem::default();
    let mut console = BufferConsole::default();

    let configuration = r#"{ "linter": { "enabled": true } }"#;
    let tailwind = r#"
import type { Config } from 'tailwindcss';

export default {
    theme: {
        extend: {
            zIndex: {
                "60": "60",
                "70": "70"
            }
        }
    }
} satisfies Config;
"#;

    let configuration_path = Utf8Path::new("biome.json");
    fs.insert(configuration_path.into(), configuration.as_bytes());

    let tailwind_path = Utf8Path::new("tailwind.config.ts");
    fs.insert(tailwind_path.into(), tailwind.as_bytes());

    let (fs, result) = run_cli(
        fs,
        &mut console,
        Args::from(["migrate", "tailwind"].as_slice()),
    );

    assert!(result.is_ok(), "run_cli returned {result:?}");

    assert_cli_snapshot(SnapshotPayload::new(
        module_path!(),
        "tailwind_migrate_ts",
        fs,
        console,
        result,
    ));
}

#[test]
fn tailwind_migrate_commonjs() {
    let fs = MemoryFileSystem::default();
    let mut console = BufferConsole::default();

    let configuration = r#"{ "linter": { "enabled": true } }"#;
    let tailwind = r#"
module.exports = {
    theme: {
        extend: {
            borderRadius: {
                "4xl": "2rem"
            }
        }
    }
}
"#;

    let configuration_path = Utf8Path::new("biome.json");
    fs.insert(configuration_path.into(), configuration.as_bytes());

    let tailwind_path = Utf8Path::new("tailwind.config.cjs");
    fs.insert(tailwind_path.into(), tailwind.as_bytes());

    let (fs, result) = run_cli(
        fs,
        &mut console,
        Args::from(["migrate", "tailwind"].as_slice()),
    );

    assert!(result.is_ok(), "run_cli returned {result:?}");

    assert_cli_snapshot(SnapshotPayload::new(
        module_path!(),
        "tailwind_migrate_commonjs",
        fs,
        console,
        result,
    ));
}

#[test]
fn tailwind_migrate_no_theme_values() {
    let fs = MemoryFileSystem::default();
    let mut console = BufferConsole::default();

    let configuration = r#"{ "linter": { "enabled": true } }"#;
    // Config with dynamic values that can't be statically analyzed
    let tailwind = r#"
export default {
    theme: {
        extend: {
            spacing: createSpacing()
        }
    }
}
"#;

    let configuration_path = Utf8Path::new("biome.json");
    fs.insert(configuration_path.into(), configuration.as_bytes());

    let tailwind_path = Utf8Path::new("tailwind.config.js");
    fs.insert(tailwind_path.into(), tailwind.as_bytes());

    let (fs, result) = run_cli(
        fs,
        &mut console,
        Args::from(["migrate", "tailwind"].as_slice()),
    );

    assert!(result.is_ok(), "run_cli returned {result:?}");

    assert_cli_snapshot(SnapshotPayload::new(
        module_path!(),
        "tailwind_migrate_no_theme_values",
        fs,
        console,
        result,
    ));
}

#[test]
fn tailwind_migrate_fix() {
    let fs = MemoryFileSystem::default();
    let mut console = BufferConsole::default();

    let configuration = r#"{ "linter": { "enabled": true } }"#;
    let tailwind = r#"
export default {
    theme: {
        extend: {
            spacing: {
                "13": "3.25rem"
            }
        }
    }
}
"#;

    let configuration_path = Utf8Path::new("biome.json");
    fs.insert(configuration_path.into(), configuration.as_bytes());

    let tailwind_path = Utf8Path::new("tailwind.config.js");
    fs.insert(tailwind_path.into(), tailwind.as_bytes());

    let (fs, result) = run_cli(
        fs,
        &mut console,
        Args::from(["migrate", "tailwind", "--fix"].as_slice()),
    );

    assert!(result.is_ok(), "run_cli returned {result:?}");

    assert_cli_snapshot(SnapshotPayload::new(
        module_path!(),
        "tailwind_migrate_fix",
        fs,
        console,
        result,
    ));
}

#[test]
fn tailwind_migrate_missing_biomejson() {
    let fs = MemoryFileSystem::default();
    let mut console = BufferConsole::default();

    let tailwind = r#"
export default {
    theme: {
        extend: {
            spacing: {
                "13": "3.25rem"
            }
        }
    }
}
"#;

    let tailwind_path = Utf8Path::new("tailwind.config.js");
    fs.insert(tailwind_path.into(), tailwind.as_bytes());

    let (fs, result) = run_cli(
        fs,
        &mut console,
        Args::from(["migrate", "tailwind"].as_slice()),
    );

    assert!(result.is_err(), "run_cli returned {result:?}");

    assert_cli_snapshot(SnapshotPayload::new(
        module_path!(),
        "tailwind_migrate_missing_biomejson",
        fs,
        console,
        result,
    ));
}

#[test]
fn tailwind_migrate_merge_existing_config() {
    let fs = MemoryFileSystem::default();
    let mut console = BufferConsole::default();

    // biome.json already has tailwind config with negatable and ignored_classes
    let configuration = r#"{
        "linter": { "enabled": true },
        "tailwind": {
            "negatable": ["custom-margin"],
            "ignoredClasses": ["^legacy-"]
        }
    }"#;
    let tailwind = r#"
export default {
    theme: {
        extend: {
            spacing: {
                "13": "3.25rem"
            }
        }
    }
}
"#;

    let configuration_path = Utf8Path::new("biome.json");
    fs.insert(configuration_path.into(), configuration.as_bytes());

    let tailwind_path = Utf8Path::new("tailwind.config.js");
    fs.insert(tailwind_path.into(), tailwind.as_bytes());

    let (fs, result) = run_cli(
        fs,
        &mut console,
        Args::from(["migrate", "tailwind", "--write"].as_slice()),
    );

    assert!(result.is_ok(), "run_cli returned {result:?}");

    assert_cli_snapshot(SnapshotPayload::new(
        module_path!(),
        "tailwind_migrate_merge_existing_config",
        fs,
        console,
        result,
    ));
}

#[test]
fn tailwind_migrate_merge_spacing() {
    let fs = MemoryFileSystem::default();
    let mut console = BufferConsole::default();

    // biome.json already has some spacing values
    let configuration = r#"{
        "linter": { "enabled": true },
        "tailwind": {
            "spacing": {
                "3.25rem": "existing-value"
            }
        }
    }"#;
    let tailwind = r#"
export default {
    theme: {
        extend: {
            spacing: {
                "13": "3.25rem",
                "18": "4.5rem"
            }
        }
    }
}
"#;

    let configuration_path = Utf8Path::new("biome.json");
    fs.insert(configuration_path.into(), configuration.as_bytes());

    let tailwind_path = Utf8Path::new("tailwind.config.js");
    fs.insert(tailwind_path.into(), tailwind.as_bytes());

    let (fs, result) = run_cli(
        fs,
        &mut console,
        Args::from(["migrate", "tailwind", "--write"].as_slice()),
    );

    assert!(result.is_ok(), "run_cli returned {result:?}");

    assert_cli_snapshot(SnapshotPayload::new(
        module_path!(),
        "tailwind_migrate_merge_spacing",
        fs,
        console,
        result,
    ));
}
