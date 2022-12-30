//! Runner for the `.test` integration test format

use util::TestManager;

#[test]
fn test_pfx_sfx() {
    let mgr = TestManager::new_from_file("1_pfxsfx.test");
    let dict = mgr.build_dict();
    mgr.check_all(&dict);
}

#[test]
fn test_nosuggest_forbid() {
    let mgr = TestManager::new_from_file("2_nosuggest_forbid.test");
    let dict = mgr.build_dict();
    mgr.check_all(&dict);
}
