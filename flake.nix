{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.11";
  };

  outputs =
    { nixpkgs, ... }:
    {
      packages = {
        x86_64-linux =
          let
            pkgs = nixpkgs.legacyPackages.x86_64-linux;
          in
          {
            browser-utils = pkgs.rustPlatform.buildRustPackage {
              pname = "browser-utils";
              version = "0.1.0";

              src = ./.;

              cargoLock.lockFile = ./Cargo.lock;

              postInstall = ''
                mkdir -p $ext/{bin,lib/mozilla/native-messaging-hosts}
                mv -v $out/bin/browser-utils-history-host $ext/bin/
                sed -e "s|@out_dir@|$ext|" history/extension-host/manifest.json > $ext/lib/mozilla/native-messaging-hosts/browser_utils_history_host.json
              '';

              outputs = [
                "out"
                "ext"
              ];
            };

            history-extension =
              let
                manifest = builtins.fromJSON (builtins.readFile ./history/extension/manifest.json);
                addonId = manifest.browser_specific_settings.gecko.id;
                pname = "browser_utils_history_recorder";
                version = manifest.version;
              in
              pkgs.stdenvNoCC.mkDerivation {
                inherit pname version;

                src = ./history/extension;

                nativeBuildInputs = [ pkgs.web-ext ];

                buildCommand = ''
                  web-ext build -s "$src"
                  dest="$out/share/mozilla/extensions/{ec8030f7-c20a-464f-9b0e-13a3a9e97384}"
                  mkdir -p "$dest"
                  install -v -m644 "web-ext-artifacts/${pname}-${version}.zip" "$dest/${addonId}.xpi"
                '';

                passthru = {
                  inherit addonId;
                };
              };
          };
      };
    };
}
