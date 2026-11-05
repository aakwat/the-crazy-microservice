{
  description = "Polyglot support-ticket PoC: Go, Java, Rust, Node, Python + k3d";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { nixpkgs, ... }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        packages = with pkgs; [
          # gateway, image
          go
          gopls
          # auth; jdtls is here because neovim has mason disabled.
          jdk21
          maven
          jdt-language-server
          # tickets
          cargo
          rustc
          clippy
          rustfmt
          rust-analyzer
          # realtime
          nodejs_24
          pnpm
          typescript
          typescript-language-server
          # jobs
          python3
          uv
          pyright
          ruff
          # cluster
          k3d
          kubectl
          yaml-language-server
          dockerfile-language-server
          # repo
          dprint
          nixd
          taplo
        ];

        JAVA_HOME = "${pkgs.jdk21}";
      };
    };
}
