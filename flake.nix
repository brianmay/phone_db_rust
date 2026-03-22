{
  description = "Phone Database";

  inputs.flake-utils.url = "github:numtide/flake-utils";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
  inputs.nixpkgs-unstable.url = "github:NixOS/nixpkgs/nixos-unstable";
  inputs.rust-overlay.url = "github:oxalica/rust-overlay";
  inputs.devenv.url = "github:cachix/devenv";
  inputs.flockenzeit.url = "github:balsoft/flockenzeit";

  outputs =
    inputs@{
      self,
      nixpkgs,
      nixpkgs-unstable,
      flake-utils,
      rust-overlay,
      devenv,
      flockenzeit,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = inputs.nixpkgs.legacyPackages.${system}.extend (import rust-overlay);
        pkgs-unstable = nixpkgs-unstable.legacyPackages.${system};

        wasm-bindgen-cli = pkgs.buildWasmBindgenCli rec {
          src = pkgs.fetchCrate {
            pname = "wasm-bindgen-cli";
            version = "0.2.106";
            hash = "sha256-M6WuGl7EruNopHZbqBpucu4RWz44/MSdv6f0zkYw+44=";
          };

          cargoDeps = pkgs.rustPlatform.fetchCargoVendor {
            inherit src;
            inherit (src) pname version;
            hash = "sha256-ElDatyOwdKwHg3bNH/1pcxKI7LXkhsotlDPQjiLHBwA=";
          };
        };

        dioxus-cli = pkgs.callPackage ./nix/dioxus-cli.nix { };
        # dioxus-cli = pkgs.dioxus-cli;

        rustPlatform = pkgs.rust-bin.stable.latest.default.override {
          targets = [ "wasm32-unknown-unknown" ];
          extensions = [ "rust-src" ];
        };

        nodejs = pkgs.nodejs_20;

        build_env = {
          BUILD_DATE = with flockenzeit.lib.splitSecondsSinceEpoch { } self.lastModified; "${F}T${T}${Z}";
          VCS_REF = "${self.shortRev or self.dirtyShortRev or "dirty"}";
        };

        postgres = pkgs.postgresql_15;
        tailwindcss = pkgs.tailwindcss_4;

        nodePackages = pkgs.buildNpmPackage {
          name = "node-packages";
          src = ./.;
          npmDepsHash = "sha256-Vr3B7aXo+bOGF2TDh+SU4w/m6EjaaU3LueXEFk5oLT8=";
          dontNpmBuild = true;
          inherit nodejs;

          installPhase = ''
            mkdir $out
            cp -r node_modules $out
            ln -s $out/node_modules/.bin $out/bin
          '';
        };

        combined =
          let
            cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
            rev = build_env.VCS_REF;
          in
          pkgs.rustPlatform.buildRustPackage {
            pname = cargoToml.package.name;
            version = "${cargoToml.package.version}-${rev}";
            src = ./.;
            strictDeps = true;
            buildInputs = [ pkgs.openssl ];
            nativeBuildInputs = [
              dioxus-cli
              rustPlatform
              wasm-bindgen-cli
              postgres
              pkgs.binaryen
              pkgs.pkg-config
            ];
            buildPhase = ''
              export VCS_REF="${build_env.VCS_REF}"
              export BUILD_DATE="${build_env.BUILD_DATE}"
              export NO_DOWNLOADS=1
              ln -s ${nodePackages}/node_modules ./node_modules
              ${tailwindcss}/bin/tailwindcss -i ./input.css -o ./assets/tailwind.css
              dx --version
              dx build --release --verbose
            '';
            installPhase = ''
              mkdir -p $out
              cp -r target/dx/$pname/release/web $out/bin
            '';
            cargoLock.lockFile = ./Cargo.lock;
            cargoLock.outputHashes = {
              # "const-serialize-0.7.0-rc.2" = "sha256-G2M0SyCWitPORvI3IeR2juuzLn1cOLhzbH6Y9lq71I8=";
              # "const-serialize-0.7.0-rc.2" = pkgs.lib.fakeHash;
            };
            meta.mainProgram = "phone_db";
          };

        test_module = pkgs.testers.nixosTest {
          name = "phone-db-test";
          nodes.machine =
            { ... }:
            {
              imports = [
                self.nixosModules.default
              ];
              services.openldap = {
                enable = true;
                urlList = [ "ldap://localhost:${toString ldap_port}" ];
                settings = {
                  attrs = {
                    olcLogLevel = "stats";
                  };
                  children = {
                    "cn=schema".includes = [
                      "${pkgs.openldap}/etc/schema/core.ldif"
                      "${pkgs.openldap}/etc/schema/cosine.ldif"
                      "${pkgs.openldap}/etc/schema/inetorgperson.ldif"
                      "${pkgs.openldap}/etc/schema/nis.ldif"
                    ];
                    "olcDatabase={1}mdb" = {
                      attrs = {
                        objectClass = [
                          "olcDatabaseConfig"
                          "olcMdbConfig"
                        ];
                        olcDatabase = "{1}mdb";
                        olcDbDirectory = "/var/lib/openldap/db";
                        olcSuffix = "${dn_suffix}";
                        olcRootDN = "${root_dn}";
                        olcRootPW = "${root_password}";
                        olcAccess = [
                          "{0}to attrs=userPassword by self write by anonymous auth by * none"
                          "{1}to * by * read"
                        ];
                      };
                    };
                  };
                };
                declarativeContents."${dn_suffix}" = ''
                  dn: ${dn_suffix}
                  objectClass: top
                  objectClass: dcObject
                  objectClass: organization
                  o: Example Organization
                  dc: example

                  dn: cn=admin,${dn_suffix}
                  objectClass: top
                  objectClass: person
                  objectClass: organizationalPerson
                  cn: admin
                  sn: Administrator
                  userPassword: ${root_password}

                  dn: cn=pwdDefaultPolicy,${dn_suffix}
                  cn: pwdDefaultPolicy
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
              services.phone-db = {
                enable = true;
                port = 4000;
                secretsFile = builtins.toFile "phone-db.env" ''
                  LDAP_SERVER=localhost
                  LDAP_PORT=${toString ldap_port}
                  LDAP_BASE_DN=${dn_suffix}
                  LDAP_USERNAME=${root_dn}
                  LDAP_PASSWORD=${root_password}
                  PHONE_USERNAME=${phone_username}
                  PHONE_PASSWORD=${phone_password}
                '';
              };
              system.stateVersion = "24.11";

              services.postgresql = {
                enable = true;
                package = pkgs.postgresql_15;
                extensions = ps: [ ps.postgis ];
                initialScript = pkgs.writeText "init.psql" ''
                  CREATE DATABASE phone_db;
                  CREATE USER phone_db with encrypted password 'your_secure_password_here';
                  ALTER DATABASE phone_db OWNER TO phone_db;
                  ALTER USER phone_db WITH SUPERUSER;
                '';
              };
            };

          testScript = ''
            machine.wait_for_unit("phone-db.service")
            machine.wait_for_open_port(4000)
            machine.succeed("${pkgs.curl}/bin/curl --fail -v http://localhost:4000/_health")
          '';
        };

        port = 4000;
        postgres_port = 6301;

        phone_username = "phone";
        phone_password = "password";

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
          if [ -z "$1" ] || [ -z "$2" ]; then
            echo "Usage: $0 <source_number> <destination_number>"
            exit 1
          fi
          source_number="$1"
          destination_number="$2"

          curl --json '{"phone_number":"'$source_number'", "destination_number":"'$destination_number'"}' --user "${phone_username}:${phone_password}" "http://localhost:${toString port}/api/incoming_call/"
        '';

        devShell = devenv.lib.mkShell {
          inherit inputs pkgs;
          modules = [
            {
              packages = [
                rustPlatform
                pkgs-unstable.rust-analyzer
                wasm-bindgen-cli
                pkgs.binaryen
                nodejs
                pkgs.cargo-watch
                pkgs.sqlx-cli
                # pkgs.jq
                pkgs.openssl
                pkgs.prefetch-npm-deps
                dioxus-cli
                # pkgs.b3sum
                pkgs.diesel-cli
                pkgs.diesel-cli-ext
                postgres
                tailwindcss
                pkgs.watchman

                pd_ldapsearch
                test_phone_call
              ];
              enterShell = ''
                export PORT="${toString port}"
                export BASE_URL="http://localhost:$PORT/"
                export DATABASE_URL="postgresql://phone_db:your_secure_password_here@localhost:${toString postgres_port}/phone_db"

                export LDAP_SERVER="localhost"
                export LDAP_PORT="${toString ldap_port}"
                export LDAP_BASE_DN="${dn_suffix}"
                export LDAP_USERNAME="${root_dn}"
                export LDAP_PASSWORD="${root_password}"

                export PHONE_USERNAME="${phone_username}"
                export PHONE_PASSWORD="${phone_password}"
              '';
              services.postgres = {
                enable = true;
                package = pkgs.postgresql_15.withPackages (ps: [ ps.postgis ]);
                listen_addresses = "127.0.0.1";
                port = postgres_port;
                initialDatabases = [ { name = "phone_db"; } ];
                initialScript = ''
                  \c phone_db;
                  CREATE USER phone_db with encrypted password 'your_secure_password_here';
                  GRANT ALL PRIVILEGES ON DATABASE phone_db TO phone_db;
                  -- GRANT ALL ON SCHEMA public TO phone_db;
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
      in
      {
        checks = {
          # brian-backend = backend.clippy;
          # frontend-bindgen = frontend.clippy;
          test_module = test_module;
        };
        packages = {
          devenv-up = devShell.config.procfileScript;
          # backend = backend.pkg;
          # frontend = frontend-bindgen;
          # combined = combined;
          default = combined;
        };
        devShells.default = devShell;
      }
    )
    // {
      nixosModules.default = import ./nix/module.nix { inherit self; };
    };
}
