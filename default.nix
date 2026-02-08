(import
  (fetchTarball {
    url = "https://github.com/edolstra/flake-compat/archive/master.tar.gz";
    sha256 = "17zv76a6p0i9i5s41shk134g615z508znid3c7y986161ay19mvw";
  })
  { src = ./.; }).defaultNix
