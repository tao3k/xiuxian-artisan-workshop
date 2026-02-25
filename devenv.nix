{
  pkgs,
  lib,
  config,
  inputs,
  ...
}:

let
  nixpkgs-latest = import inputs.nixpkgs-latest {
    system = pkgs.stdenv.hostPlatform.system;
    config = {
      allowUnfree = true;
    };
  };
  nixosModules =
    (inputs.omnibus.pops.nixosProfiles.addLoadExtender {
      load = {
        src = ./nix/modules;
        inputs = {
          __nixpkgs__ = nixpkgs-latest;
          __inputs__ = {
            inherit (inputs) llm-agents;
            inherit nixpkgs-latest packages;
          };
          inputs = {
            nixpkgs = nixpkgs-latest;
          };
        };
      };
    }).exports.default;

  packages =
    (inputs.omnibus.pops.packages.addLoadExtender {
      load = {
        src = ./nix/packages;
        inputs = {
          inputs = {
            nixpkgs = nixpkgs-latest;
          };
        };
      };
    }).exports.packages;
in
{
  imports = [
    nixosModules.claude
    nixosModules.flake-parts.omnibus
    nixosModules.files
    nixosModules.lefthook
    nixosModules.python
    nixosModules.llm
    nixosModules.rust
    nixosModules.packages
    nixosModules.tasks
    nixosModules.process
    #./modules/flake-parts/omnibus-hive.nix
    ({
      config = lib.mkMerge [
        {
          omnibus = {
            inputs = {
              inputs = {
                nixpkgs = pkgs;
                inherit nixpkgs-latest;
                inherit (inputs.omnibus.flake.inputs) nixago;
              };
            };
          };
        }
      ];
    })
  ];

  devcontainer.enable = true;
  # https://devenv.sh/basics/
  env.GREET = "devenv";
  # devenv.warnOnNewVersion = false;
  # https://devenv.sh/packages/
  packages = [
    packages.secretspec
    pkgs.ollama
    pkgs.valkey
    pkgs.ngrok
  ];

  dotenv.enable = true;
  dotenv.filename = [ ".env" ];
  # https://devenv.sh/processes/
  # processes.cargo-watch.exec = "cargo-watch";

  # https://devenv.sh/services/
  # services.postgres.enable = true;

  # https://devenv.sh/scripts/
  scripts.hello.exec = ''
    echo hello from $GREET
  '';

  # https://devenv.sh/tasks/
  # tasks = {
  #   "myproj:setup".exec = "mytool build";
  #   "devenv:enterShell".after = [ "myproj:setup" ];
  # };

  enterShell = ''
    export PATH="$PATH:$DEVENV_ROOT/.venv/bin"
    export OLLAMA_MODELS="''${OLLAMA_MODELS:-''${PRJ_DATA_HOME:-$DEVENV_ROOT/.data}/models}"
  '';
  # https://devenv.sh/tests/
  enterTest = "";

  # https://devenv.sh/pre-commit-hooks/
  # git-hooks.hooks.shellcheck.enable = true;
  # git-hooks.hooks.nixfmt.enable = true;
  # See full reference at https://devenv.sh/reference/options/
}
