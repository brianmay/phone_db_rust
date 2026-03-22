{ self }:
{
  lib,
  pkgs,
  config,
  ...
}:
let
  inherit (lib)
    types
    mkOption
    mkEnableOption
    mkIf
    ;

  cfg = config.services.phone-db;

  system = pkgs.stdenv.hostPlatform.system;
  phone-db = self.packages.${system}.default;

in
{
  options.services.phone-db = {
    enable = mkEnableOption "phone-db service";
    data_dir = mkOption {
      type = types.str;
      default = "/var/lib/phone-db";
      description = lib.mdDoc ''
        The directory where phone-db stores its home directory.
      '';
    };
    port = mkOption {
      type = types.int;
      default = 8080;
      description = lib.mdDoc ''
        The port on which the phone-db service listens.
      '';
    };
    base_url = mkOption {
      type = types.str;
      default = "http://localhost:${toString cfg.port}";
      description = lib.mdDoc ''
        The external base URL of the phone-db service.
        Used to generate the OIDC redirect URL. Not used if OIDC not configured.
      '';
    };
    secretsFile = mkOption {
      type = types.nullOr types.str;
      default = null;
      example = "/run/secrets/phone-db.env";
      description = lib.mdDoc ''
         Optional path to an env file containing the secrets used by phone-db.

        Might contain:
         - `OIDC_DISCOVERY_URL` - The URL to the OIDC.
         - `OIDC_CLIENT_ID` - The Client ID for the OIDC.
         - `OIDC_CLIENT_SECRET` - The Client secret for the OIDC.
         - `OIDC_AUTH_SCOPE` - "openid profile groups email" or similar.
         - `LDAP_SERVER` - The LDAP server.
         - `LDAP_PORT` - The LDAP server port.
         - `LDAP_BASE_DN` - The base DN for LDAP.
         - `LDAP_USERNAME` - The username to connect to the LDAP server.
         - `LDAP_PASSWORD` - The password for the LDAP server.
         - `PHONE_USERNAME` - The username to connect to the API.
         - `PHONE_PASSWORD` - The password to connect to the API.
      '';
    };
  };

  config = mkIf cfg.enable {
    users.users.phone_db = {
      isSystemUser = true;
      description = "Penguin Phone DB user";
      group = "phone_db";
      createHome = true;
      home = "${cfg.data_dir}";
    };

    users.groups.phone_db = { };

    systemd.services.phone-db = {
      wantedBy = [ "multi-user.target" ];
      after = [ "postgresql.service" ];
      wants = [ "postgresql.service" ];
      serviceConfig = {
        User = "phone_db";
        ExecStart = "${lib.getExe phone-db}";
        EnvironmentFile = cfg.secretsFile;
      };
      environment = {
        RUST_LOG = "info";
        PORT = toString cfg.port;
        BASE_URL = cfg.base_url;
        DATABASE_URL = "postgresql:///phone_db?host=/var/run/postgresql";
      };
    };
  };
}
