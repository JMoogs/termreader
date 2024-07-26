{ lib, rustPlatform, fetchFromGitHub }:
rustPlatform.buildRustPackage rec {
  pname = "termreader";
  version = "0.1.0";

  src = fetchFromGitHub {
    owner = "JMoogs";
    repo = "termreader";
    # rev = version;
    rev = "74461b6";
    sha256 = "sha256-Y2WDDlT2Ch2kGUTL8QxVBGcwW75c2+cs8khooO4juas=";
  };

  cargoSha256 = lib.fakeHash;

  meta = with lib; {
    description = "A TUI reading app";
    homepage = "https://github.com/JMoogs/termreader";
    license = [ licenses.mit licenses.asl20 ];
  };

}
