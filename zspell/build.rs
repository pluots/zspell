use std::fmt::Write;
use std::path::Path;
use std::{env, fs};

use indoc::indoc;

fn main() {
    update_tests();
    emit_autocfg();
}

const TEST_PREFIX: &str = "// autogenerated file, do not edit manually
// one test is generated for each `.test` file

";

/// Autogenerate an integration test for every `.test` file
fn update_tests() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let out_path = Path::new(&env::var("OUT_DIR").unwrap()).join("auto_suite.rs");
    let suite_dir = root.join("test-suite");
    let test_paths = fs::read_dir(suite_dir).unwrap();

    let mut to_write = TEST_PREFIX.to_owned();

    for path in test_paths {
        let path = path.unwrap().path();
        // let path_str = path.display();
        let fname = path.file_name().unwrap().to_string_lossy();
        let test_name = fname
            .strip_suffix(".test")
            .unwrap()
            .trim_start_matches(char::is_numeric)
            .trim_start_matches('_')
            .replace('-', "_");

        if test_name == "example" {
            continue;
        }

        write!(
            to_write,
            indoc! {"

                #[test]
                fn test_{test_name}() {{
                    let path = std::path::Path::new(env!(\"CARGO_MANIFEST_DIR\"));
                    let path = path.join(\"test-suite/{fname}\");
                    let mgr = test_util::TestManager::new_from_file(path);
                    let dict = mgr.build_dict();
                    mgr.check_all(&dict);
                }}
            "},
            test_name = test_name,
            fname = fname,
        )
        .unwrap();
    }

    fs::write(out_path, to_write).unwrap();
}

/// Add configuration that depends on rust version
fn emit_autocfg() {
    const PROBE_BOX: &str = r#" || {
        let s = "foo".to_owned();
        let _b: Box<[String]> = [s].as_slice().into();
    }
    "#;

    let ac = autocfg::new();

    // check if we have `Box<[T]>: From<&[T: Clone]>` loosened from `T: Copy` (1.71)
    ac.emit_expression_cfg(PROBE_BOX, "box_from_slice_has_clone_bound");
}
