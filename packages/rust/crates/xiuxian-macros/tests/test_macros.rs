//! Tests for xiuxian-macros.

use xiuxian_macros::{
    assert_timing, bench_case, env_first_non_empty, env_non_empty, patterns, project_config_paths,
    py_from, string_first_non_empty, temp_dir, topics,
};

// Test patterns! macro
mod test_patterns {
    use super::*;

    patterns![
        (TEST_PATTERN_1, "pattern one"),
        (TEST_PATTERN_2, "pattern two"),
    ];

    #[test]
    fn test_patterns_generated() {
        assert_eq!(TEST_PATTERN_1, "pattern one");
        assert_eq!(TEST_PATTERN_2, "pattern two");
    }
}

// Test topics! macro
mod test_topics {
    use super::*;

    topics![(TOPIC_ONE, "topic/one"), (TOPIC_TWO, "topic/two"),];

    #[test]
    fn test_topics_generated() {
        assert_eq!(TOPIC_ONE, "topic/one");
        assert_eq!(TOPIC_TWO, "topic/two");
    }
}

// Test py_from! macro
mod test_py_from {
    use super::*;

    struct Inner {
        value: i32,
    }

    struct PyWrapper {
        inner: Inner,
    }

    py_from!(PyWrapper, Inner);

    #[test]
    fn test_py_from_generated() {
        let inner = Inner { value: 42 };
        let wrapper = PyWrapper::from(inner);
        assert_eq!(wrapper.inner.value, 42);
    }
}

// Test temp_dir! macro
mod test_temp_dir {
    use super::*;
    use std::fs;

    #[test]
    fn test_temp_dir_creates_directory() {
        let temp_path = temp_dir!();
        assert!(temp_path.exists());
        assert!(temp_path.is_dir());

        // Clean up
        if let Err(error) = fs::remove_dir_all(&temp_path) {
            panic!("failed to remove temporary directory {temp_path:?}: {error}");
        }
    }

    #[test]
    fn test_temp_dir_is_unique() {
        let temp_path1 = temp_dir!();
        let temp_path2 = temp_dir!();

        assert_ne!(temp_path1, temp_path2);

        // Clean up
        let _ = fs::remove_dir_all(&temp_path1);
        let _ = fs::remove_dir_all(&temp_path2);
    }
}

// Test assert_timing! macro
mod test_assert_timing {
    use super::*;

    #[test]
    fn test_assert_timing_passes_fast_operation() {
        let elapsed = assert_timing!(100.0, {
            // Fast operation
            let x = 1 + 1;
            assert_eq!(x, 2);
        });
        assert!(elapsed.as_millis() < 100);
    }

    #[test]
    fn test_assert_timing_returns_elapsed() {
        let elapsed = assert_timing!(1000.0, {
            std::thread::sleep(std::time::Duration::from_millis(1));
        });
        assert!(elapsed.as_millis() >= 1);
    }
}

// Test bench_case! macro
mod test_bench_case {
    use super::*;

    #[test]
    fn test_bench_case_measures_time() {
        let elapsed = bench_case!({
            let sum: i32 = (0..100).sum();
            assert_eq!(sum, 4950);
        });
        assert!(elapsed.as_nanos() > 0);
    }

    #[test]
    fn test_bench_case_simple() {
        let elapsed = bench_case!(1 + 1);
        // Verify that bench_case returns a duration value.
        let _ = elapsed;
    }
}

// Test project_config_paths! macro
mod test_project_config_paths {
    use super::*;
    use std::path::PathBuf;
    use std::sync::{Mutex, OnceLock};

    struct EnvRestore {
        key: &'static str,
        previous: Option<String>,
    }

    impl Drop for EnvRestore {
        fn drop(&mut self) {
            if let Some(value) = self.previous.take() {
                std::env::set_var(self.key, value);
            } else {
                std::env::remove_var(self.key);
            }
        }
    }

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn set_env_for_test(key: &'static str, value: &str) -> EnvRestore {
        let previous = std::env::var(key).ok();
        std::env::set_var(key, value);
        EnvRestore { key, previous }
    }

    #[test]
    fn test_project_config_paths_generates_layered_candidates() {
        let _guard = env_lock()
            .lock()
            .unwrap_or_else(|_| panic!("failed to lock environment mutex for config path test"));
        let _root = set_env_for_test("PRJ_ROOT", "/tmp/omni-macro-prj");
        let _config_home = set_env_for_test("PRJ_CONFIG_HOME", "/tmp/omni-macro-conf");
        let _explicit = set_env_for_test("QIANJI_CONFIG_PATH", "/tmp/custom/qianji.toml");

        let paths = project_config_paths!("qianji.toml", "QIANJI_CONFIG_PATH");
        assert_eq!(paths.len(), 2);
        assert_eq!(
            paths[0],
            PathBuf::from("/tmp/omni-macro-conf/xiuxian-artisan-workshop/qianji.toml")
        );
        assert_eq!(paths[1], PathBuf::from("/tmp/custom/qianji.toml"));
    }
}

// Test env_non_empty! and string_first_non_empty! macros
mod test_llm_env_macros {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    struct EnvRestore {
        key: &'static str,
        previous: Option<String>,
    }

    impl Drop for EnvRestore {
        fn drop(&mut self) {
            if let Some(value) = self.previous.take() {
                std::env::set_var(self.key, value);
            } else {
                std::env::remove_var(self.key);
            }
        }
    }

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn set_env_for_test(key: &'static str, value: &str) -> EnvRestore {
        let previous = std::env::var(key).ok();
        std::env::set_var(key, value);
        EnvRestore { key, previous }
    }

    #[test]
    fn test_env_non_empty_trims_value() {
        let _guard = env_lock()
            .lock()
            .unwrap_or_else(|_| panic!("failed to lock environment mutex for env_non_empty test"));
        let _restore = set_env_for_test("OMNI_MACROS_TEST_KEY", "  test-key  ");

        let value = env_non_empty!("OMNI_MACROS_TEST_KEY");
        assert_eq!(value.as_deref(), Some("test-key"));
    }

    #[test]
    fn test_string_first_non_empty_prefers_first_non_blank() {
        let value = string_first_non_empty!(
            None::<&str>,
            Some(""),
            Some("   "),
            Some("winner"),
            Some("later")
        );
        assert_eq!(value, "winner");
    }

    #[test]
    fn test_env_first_non_empty_prefers_first_present_key() {
        let _guard = env_lock().lock().unwrap_or_else(|_| {
            panic!("failed to lock environment mutex for env_first_non_empty test")
        });
        let _primary = set_env_for_test("OMNI_MACROS_KEY_PRIMARY", "primary-secret");
        let _fallback = set_env_for_test("OMNI_MACROS_KEY_FALLBACK", "fallback-secret");

        let value = env_first_non_empty!("OMNI_MACROS_KEY_PRIMARY", "OMNI_MACROS_KEY_FALLBACK");
        assert_eq!(value.as_deref(), Some("primary-secret"));
    }

    #[test]
    fn test_env_first_non_empty_skips_blank_and_uses_fallback() {
        let _guard = env_lock().lock().unwrap_or_else(|_| {
            panic!("failed to lock environment mutex for env_first_non_empty fallback test")
        });
        let _primary = set_env_for_test("OMNI_MACROS_KEY_PRIMARY", "   ");
        let _fallback = set_env_for_test("OMNI_MACROS_KEY_FALLBACK", "fallback-secret");

        let dynamic_primary = "OMNI_MACROS_KEY_PRIMARY";
        let value = env_first_non_empty!(dynamic_primary, "OMNI_MACROS_KEY_FALLBACK");
        assert_eq!(value.as_deref(), Some("fallback-secret"));
    }
}
