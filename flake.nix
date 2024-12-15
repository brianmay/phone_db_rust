{
  description = "Time tracking application";

  inputs.flake-utils.url = "github:numtide/flake-utils";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
  inputs.rust-overlay.url = "github:oxalica/rust-overlay";
  inputs.devenv.url = "github:cachix/devenv";
  inputs.crane.url = "github:ipetkov/crane";
  inputs.flockenzeit.url = "github:balsoft/flockenzeit";

  outputs = inputs @ {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
    devenv,
    crane,
    flockenzeit,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [(import rust-overlay)];
        };
        wasm-bindgen-cli = pkgs.wasm-bindgen-cli.override (old: {
          version = "0.2.99";
          hash = "sha256-1AN2E9t/lZhbXdVznhTcniy+7ZzlaEp/gwLEAucs6EA=";
          # hash = pkgs.lib.fakeHash;
          cargoHash = "sha256-DbwAh8RJtW38LJp+J9Ht8fAROK9OabaJ85D9C/Vkve4=";
          # cargoHash = pkgs.lib.fakeHash;
        });
        rustPlatform = pkgs.rust-bin.stable.latest.default.override {
          targets = ["wasm32-unknown-unknown"];
          extensions = ["rust-src"];
        };
        craneLib = (crane.mkLib pkgs).overrideToolchain rustPlatform;

        build_env = {
          BUILD_DATE = with flockenzeit.lib.splitSecondsSinceEpoch {} self.lastModified; "${F}T${T}${Z}";
          VCS_REF = "${self.rev or "dirty"}";
        };

        backend = let
          common = {
            src = ./.;
            pname = "phone_db-backend";
            version = "0.0.0";
            cargoExtraArgs = "-p backend";
            nativeBuildInputs = with pkgs; [pkg-config];
            buildInputs = with pkgs; [
              openssl
              python3
              protobuf
            ];
            # See https://github.com/ipetkov/crane/issues/414#issuecomment-1860852084
            # for possible work around if this is required in the future.
            # installCargoArtifactsMode = "use-zstd";
          };

          # Build *just* the cargo dependencies, so we can reuse
          # all of that work (e.g. via cachix) when running in CI
          cargoArtifacts = craneLib.buildDepsOnly common;

          # Run clippy (and deny all warnings) on the crate source.
          clippy = craneLib.cargoClippy (
            {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "-- --deny warnings";
            }
            // common
          );

          # Next, we want to run the tests and collect code-coverage, _but only if
          # the clippy checks pass_ so we do not waste any extra cycles.
          coverage = craneLib.cargoTarpaulin ({cargoArtifacts = clippy;} // common);

          # Build the actual crate itself.
          pkg = craneLib.buildPackage (
            {
              inherit cargoArtifacts;
              doCheck = true;
              # CARGO_LOG = "cargo::core::compiler::fingerprint=info";
            }
            // common
            // build_env
          );
        in {
          inherit clippy coverage pkg;
        };

        port = 4000;
        postgres_port = 8100;

        ldap_port = 6102;
        ldap_dir = ".devenv/state/ldap";
        ldap_url = "ldap://localhost:${toString ldap_port}/";

        dn_suffix = "dc=python-ldap,dc=org";
        root_dn = "cn=root,${dn_suffix}";
        root_password = "your_secure_password_here";

        slapd_config = pkgs.writeTextFile {
          name = "slapd.conf";
          text = ''
            include ${pkgs.openldap}/etc/schema/core.schema
            include ${pkgs.openldap}/etc/schema/cosine.schema
            include ${pkgs.openldap}/etc/schema/inetorgperson.schema

            allow bind_v2

            # Database
            moduleload back_mdb
            moduleload ppolicy

            database mdb
            directory ${ldap_dir}

            suffix ${dn_suffix}
            overlay ppolicy
            ppolicy_default cn=default,${dn_suffix}

            access to dn.sub=${dn_suffix} attrs=userPassword
               by anonymous auth

            access to dn.sub=${dn_suffix}
               by dn.exact=${root_dn} write
          '';
        };

        admin_ldif = pkgs.writeTextFile {
          name = "admin.ldif";
          text = ''
            dn: ${dn_suffix}
            o: Test Org
            objectClass: dcObject
            objectClass: organization

            dn: ${root_dn}
            cn: ${root_dn}
            objectClass: simpleSecurityObject
            objectClass: organizationalRole
            userPassword: ${root_password}

            dn: cn=default,${dn_suffix}
            objectClass: top
            objectClass: device
            objectClass: pwdPolicy
            pwdAttribute: userPassword
            pwdLockout: TRUE

            dn: ou=People,${dn_suffix}
            objectClass: top
            objectClass: OrganizationalUnit
            ou: People

            dn: ou=Groups,${dn_suffix}
            objectClass: top
            objectClass: OrganizationalUnit
          '';
        };

        phone_username = "phone";
        phone_password = "password";

        pd_ldapsearch = pkgs.writeShellScriptBin "pd_ldapsearch" ''
          exec ${pkgs.openldap}/bin/ldapsearch -H "${ldap_url}" -D "${root_dn}" -b "${dn_suffix}" -w your_secure_password_here
        '';

        start_ldap = pkgs.writeShellScriptBin "start_ldap" ''
          set -e
          set -x
          if ! test -d "${ldap_dir}"; then
            mkdir "${ldap_dir}"
            cat "${admin_ldif}"
            "${pkgs.openldap}/bin/slapadd" -n 1 -f "${slapd_config}" -l "${admin_ldif}"
          fi
          "${pkgs.openldap}/libexec/slapd" -f "${slapd_config}" -h "${ldap_url}"  -d 1
        '';

        test_phone_call = pkgs.writeShellScriptBin "test_phone_call" ''
          curl --json '{"phone_number":"1", "destination_number":"2"}' --user "${phone_username}:${phone_password}" "http://localhost:${toString port}/api/incoming_call/"
        '';

        devShell = devenv.lib.mkShell {
          inherit inputs pkgs;
          modules = [
            {
              packages = [
                rustPlatform
                pkgs.rust-analyzer
                wasm-bindgen-cli
                pkgs.nodejs_20
                pkgs.cargo-watch
                pkgs.sqlx-cli
                pkgs.jq
                pkgs.openssl
                test_phone_call
                pd_ldapsearch
              ];
              enterShell = ''
                export HTTP_LISTEN="localhost:${toString port}"
                export DATABASE_URL="postgresql://phone_db:your_secure_password_here@localhost:${toString postgres_port}/phone_db"

                export LDAP_SERVER="localhost"
                export LDAP_PORT="${toString ldap_port}"
                export LDAP_BASE_DN="${dn_suffix}"
                export LDAP_USERNAME="${root_dn}"
                export LDAP_PASSWORD="${root_password}"

                export PHONE_USERNAME="${phone_username}"
                export PHONE_PASSWORD="${phone_password}"

                export STATIC_PATH="frontend/dist"
              '';
              services.postgres = {
                enable = true;
                package = pkgs.postgresql_15;
                listen_addresses = "127.0.0.1";
                port = postgres_port;
                initialDatabases = [{name = "phone_db";}];
                initialScript = ''
                  \c phone_db;
                  CREATE USER phone_db with encrypted password 'your_secure_password_here';
                  GRANT ALL PRIVILEGES ON DATABASE phone_db TO robotica;
                  GRANT ALL ON SCHEMA public TO phone_db;
                  ALTER USER phone_db WITH SUPERUSER;
                '';
              };
              processes.slapd = {
                exec = "${start_ldap}/bin/start_ldap";
                process-compose = {
                  readiness_probe = {
                    exec.command = "${pd_ldapsearch}/bin/pd_ldapsearch";
                    initial_delay_seconds = 2;
                    period_seconds = 10;
                    timeout_seconds = 1;
                    success_threshold = 1;
                    failure_threshold = 3;
                  };
                };
              };
            }
          ];
        };
      in {
        checks = {
          brian-backend = backend.clippy;
        };
        packages = {
          devenv-up = devShell.config.procfileScript;
          backend = backend.pkg;
        };
        devShells.default = devShell;
      }
    );
}
