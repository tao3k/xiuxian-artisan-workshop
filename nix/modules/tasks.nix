{
  config,
  lib,
  pkgs,
  ...
}:
let
  mkPath = packages: lib.makeBinPath (lib.filter lib.isDerivation packages);

  pythonBaseEnv = [
    config.languages.python.uv.package
    config.languages.python.package
    pkgs.bash
    pkgs.coreutils
  ];

  pythonScriptEnv = pythonBaseEnv ++ [
    pkgs.just
    pkgs.findutils
    pkgs.gawk
    pkgs.gitMinimal
    pkgs.gnugrep
    pkgs.gnused
  ];

  pythonBenchmarkEnv = pythonScriptEnv ++ [
    pkgs.ripgrep
  ];

  rustBaseEnv = pythonScriptEnv ++ [
    pkgs.ripgrep
    config.languages.rust.toolchainPackage
    pkgs.clang
    pkgs.openssl
    pkgs.pkg-config
    pkgs.protobuf
    pkgs.python3
    pkgs.zlib
  ];

  rustQualityEnv = rustBaseEnv ++ [
    pkgs.cargo-audit
    pkgs.cargo-deny
    pkgs.cargo-nextest
  ];

  rustSecurityEnv = rustBaseEnv ++ [
    pkgs.cargo-audit
    pkgs.cargo-deny
  ];

  # Reuse CI-relevant tool packages from global config, but exclude heavy runtime-only tools.
  ciSupportEnv = lib.filter (
    pkg:
    lib.isDerivation pkg
    && !(lib.elem (lib.getName pkg) [
      "ollama"
      "ngrok"
      "secretspec"
      "valkey"
    ])
  ) config.packages;

  hookEnv = pythonBenchmarkEnv ++ ciSupportEnv;
  pythonTaskEnv = pythonBaseEnv;
  pythonScriptTaskEnv = pythonScriptEnv;
  pythonBenchmarkTaskEnv = pythonBenchmarkEnv;
  rustTaskEnv = rustBaseEnv;
  rustQualityTaskEnv = rustQualityEnv;
  rustSecurityTaskEnv = rustSecurityEnv;
  runtimeTaskEnv = rustBaseEnv ++ [ pkgs.valkey ];

  mkTask = envPackages: command: {
    exec = command;
    env = {
      PATH = "${mkPath envPackages}:$PATH";
    };
  };

  mkRustTaskWith = envPackages: command: {
    exec = ''
      export PKG_CONFIG_PATH="${pkgs.zlib.dev}/lib/pkgconfig:${pkgs.zlib.out}/lib/pkgconfig:''${PKG_CONFIG_PATH:-}"
      ${command}
    '';
    env = {
      PATH = "${mkPath envPackages}:$PATH";
      PROTOC = "${pkgs.protobuf}/bin/protoc";
      PYO3_PYTHON = "${config.languages.python.package}/bin/python";
    };
  };

  mkRustTask = command: mkRustTaskWith rustTaskEnv command;
  mkRustQualityTask = command: mkRustTaskWith rustQualityTaskEnv command;
  mkRustSecurityTask = command: mkRustTaskWith rustSecurityTaskEnv command;

  mkPythonTask = command: mkTask pythonTaskEnv command;
  mkPythonScriptTask = command: mkTask pythonScriptTaskEnv command;
  mkPythonBenchmarkTask = command: mkTask pythonBenchmarkTaskEnv command;
  mkRuntimeTask = command: mkTask runtimeTaskEnv command;
in
{
  tasks = {
    "ci:architecture-gate" = mkPythonScriptTask ''
      just architecture-gate
    '';

    "ci:lint" = mkTask hookEnv ''
      just lint
    '';

    "ci:check-format" = mkTask hookEnv ''
      just check-format
    '';

    "ci:check-commits" = mkTask hookEnv ''
      just check-commits
    '';

    "ci:rust-quality-gate" = mkRustQualityTask ''
      just rust-quality-gate-ci "''${RUST_CHECK_TIMEOUT_SECS:-3600}"
    '';

    "ci:rust-security-gate" = mkRustSecurityTask ''
      just rust-security-gate
    '';

    "ci:rust-omni-core-rs-lib" = mkRustTask ''
      just rust-omni-core-rs-lib
    '';

    "ci:rust-omni-agent-profiles" = mkRustTask ''
      just rust-omni-agent-profiles
    '';

    "ci:rust-omni-agent-dependency-assertions" = mkRustTask ''
      just rust-omni-agent-dependency-assertions
    '';

    "ci:rust-omni-agent-backend-role-contracts" = mkRustTask ''
      just rust-omni-agent-backend-role-contracts
    '';

    "ci:rust-omni-agent-embedding-role-perf-medium-gate" = mkRustTask ''
      just rust-omni-agent-embedding-role-perf-medium-gate
    '';

    "ci:rust-omni-agent-embedding-role-perf-heavy-gate" = mkRustTask ''
      just rust-omni-agent-embedding-role-perf-heavy-gate
    '';

    "ci:rust-fusion-snapshots" = mkRustTask ''
      just rust-fusion-snapshots
    '';

    "ci:rust-search-perf-guard" = mkRustTask ''
      just rust-search-perf-guard
    '';

    "ci:rust-retrieval-audits" = mkRustTask ''
      just rust-retrieval-audits
    '';

    "ci:contract-e2e-route-test-json" = mkPythonScriptTask ''
      just contract-e2e-route-test-json
    '';

    "ci:contract-freeze" = mkPythonScriptTask ''
      just test-contract-freeze
    '';

    "ci:docs-vector-search-options-check" = mkPythonScriptTask ''
      just docs-vector-search-options-check
    '';

    "ci:scripts-smoke" = mkPythonScriptTask ''
      just ci-scripts-smoke
    '';

    "ci:test-quick" = mkPythonScriptTask ''
      just test-quick
    '';

    "ci:no-inline-python-guard" = mkPythonScriptTask ''
      just no-inline-python-guard
    '';

    "ci:benchmark-skills-tools" = mkPythonBenchmarkTask ''
      just benchmark-skills-tools-ci \
        "''${OMNI_SKILLS_TOOLS_REPORT_DIR:-.run/reports/skills-tools-benchmark}" \
        "''${OMNI_SKILLS_TOOLS_DETERMINISTIC_RUNS:-3}" \
        "''${OMNI_SKILLS_TOOLS_NETWORK_RUNS:-5}"
    '';

    "ci:mcp-tools-list-sweep" = mkPythonScriptTask ''
      just benchmark-mcp-tools-list-sweep \
        "''${OMNI_MCP_TOOLS_LIST_BASE_URL:-}" \
        "''${OMNI_MCP_TOOLS_LIST_HOST:-}" \
        "''${OMNI_MCP_TOOLS_LIST_PORT:-}" \
        "''${OMNI_MCP_TOOLS_LIST_NO_EMBEDDING:-true}" \
        "''${OMNI_MCP_TOOLS_LIST_HEALTH_TIMEOUT_SECS:-120}" \
        "''${OMNI_MCP_TOOLS_LIST_TOTAL:-1000}" \
        "''${OMNI_MCP_TOOLS_LIST_CONCURRENCY_VALUES:-40,80,120,160,200}" \
        "''${OMNI_MCP_TOOLS_LIST_WARMUP_CALLS:-2}" \
        "''${OMNI_MCP_TOOLS_LIST_TIMEOUT_SECS:-30}" \
        "''${OMNI_MCP_TOOLS_LIST_P95_SLO_MS:-400}" \
        "''${OMNI_MCP_TOOLS_LIST_P99_SLO_MS:-800}" \
        "''${OMNI_MCP_TOOLS_LIST_STRICT_SNAPSHOT:-true}" \
        "''${OMNI_MCP_TOOLS_LIST_WRITE_SNAPSHOT:-false}" \
        "''${OMNI_MCP_TOOLS_LIST_REPORT_DIR:-.run/reports/mcp-tools-list-sweep}"
    '';

    "ci:knowledge-recall-gates" = mkPythonScriptTask ''
      just knowledge-recall-perf-ci \
        "''${OMNI_KNOWLEDGE_RECALL_RUNS:-3}" \
        "''${OMNI_KNOWLEDGE_RECALL_WARM_RUNS:-1}" \
        "''${OMNI_KNOWLEDGE_RECALL_QUERY:-x}" \
        "''${OMNI_KNOWLEDGE_RECALL_LIMIT:-2}" \
        "''${OMNI_KNOWLEDGE_RECALL_REPORT_DIR:-.run/reports/knowledge-recall-perf}"
    '';

    "ci:wendao-ppr-gate" = mkPythonScriptTask ''
      just gate-wendao-ppr
    '';

    "ci:wendao-ppr-report" = mkPythonScriptTask ''
      just gate-wendao-ppr-report
    '';

    "ci:wendao-ppr-mixed-canary" = mkPythonScriptTask ''
      just gate-wendao-ppr-mixed-canary
    '';

    "ci:wendao-ppr-report-validate" = mkPythonScriptTask ''
      just validate-wendao-ppr-reports
    '';

    "ci:wendao-ppr-gate-summary" = mkPythonScriptTask ''
      just wendao-ppr-gate-summary
    '';

    "ci:wendao-ppr-rollout-status" = mkPythonScriptTask ''
      just wendao-ppr-rollout-status
    '';

    "ci:memory-gate-quick" = mkRuntimeTask ''
      just memory-gate-quick
    '';

    "ci:memory-gate-nightly" = mkRuntimeTask ''
      just memory-gate-nightly
    '';

    "ci:memory-gate-a7" = mkRuntimeTask ''
      just memory-gate-a7
    '';

    "ci:native-runtime-smoke" = mkPythonScriptTask ''
      just verify-native-runtime
    '';

    "ci:valkey-live" = mkRuntimeTask ''
      just valkey-live
    '';

    "ci:telegram-session-isolation-rust" = mkRustTask ''
      just telegram-session-isolation-rust
    '';

    "ci:telegram-session-isolation-python" = mkPythonScriptTask ''
      just telegram-session-isolation-python
    '';

    "dev:clean-generated" = mkTask hookEnv ''
      just clean-generated
    '';

    "dev:clean-rust" = mkRustTask ''
      just clean-rust
    '';

    "dev:clean-all" = mkRustTask ''
      just clean-all
    '';
  };
}
