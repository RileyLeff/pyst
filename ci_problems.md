on our first CI attempt, we had some problems where:

linux arm64 native tests failed due to not being able to find openssl

code coverage failed with a few of these: ---- integration::execution_tests::test_pyst_scripts_with_uv stdout ----

thread 'integration::execution_tests::test_pyst_scripts_with_uv' panicked at pyst/tests/integration/execution_tests.rs:144:41:
Failed to start container: Client(PullImage { descriptor: "pyst-test:latest", err: DockerResponseServerError { status_code: 404, message: "pull access denied for pyst-test, repository does not exist or may require 'docker login': denied: requested access to the resource is denied" } })



I then asked claude to fix. 

Windows now fails with this: 
ParserError: D:\a\_temp\6c7d99a5-2fd1-4229-bd44-e6816587a9f0.ps1:2
Line |
   2 |  if [ "x86_64-pc-windows-msvc" = "aarch64-unknown-linux-gnu" ]; then
     |    ~
     | Missing '(' after 'if' in if statement.
Error: Process completed with exit code 1.

Linux arm64 fails with this:
Run sudo apt-get update
Get:1 file:/etc/apt/apt-mirrors.txt Mirrorlist [144 B]
Hit:6 https://packages.microsoft.com/repos/azure-cli noble InRelease
Get:7 https://packages.microsoft.com/ubuntu/24.04/prod noble InRelease [3600 B]
Hit:2 http://azure.archive.ubuntu.com/ubuntu noble InRelease
Get:3 http://azure.archive.ubuntu.com/ubuntu noble-updates InRelease [126 kB]
Get:4 http://azure.archive.ubuntu.com/ubuntu noble-backports InRelease [126 kB]
Get:5 http://azure.archive.ubuntu.com/ubuntu noble-security InRelease [126 kB]
Get:8 https://packages.microsoft.com/ubuntu/24.04/prod noble/main amd64 Packages [47.9 kB]
Get:9 https://packages.microsoft.com/ubuntu/24.04/prod noble/main arm64 Packages [33.0 kB]
Get:10 http://azure.archive.ubuntu.com/ubuntu noble-updates/main amd64 Packages [1325 kB]
Get:11 http://azure.archive.ubuntu.com/ubuntu noble-updates/main Translation-en [265 kB]
Get:12 http://azure.archive.ubuntu.com/ubuntu noble-updates/main amd64 Components [175 kB]
Get:13 http://azure.archive.ubuntu.com/ubuntu noble-updates/universe amd64 Packages [1120 kB]
Get:14 http://azure.archive.ubuntu.com/ubuntu noble-updates/universe Translation-en [287 kB]
Get:15 http://azure.archive.ubuntu.com/ubuntu noble-updates/universe amd64 Components [377 kB]
Get:16 http://azure.archive.ubuntu.com/ubuntu noble-updates/restricted amd64 Packages [1672 kB]
Get:17 http://azure.archive.ubuntu.com/ubuntu noble-updates/restricted Translation-en [367 kB]
Get:18 http://azure.archive.ubuntu.com/ubuntu noble-updates/restricted amd64 Components [212 B]
Get:19 http://azure.archive.ubuntu.com/ubuntu noble-updates/multiverse amd64 Components [940 B]
Get:20 http://azure.archive.ubuntu.com/ubuntu noble-backports/main amd64 Components [7084 B]
Get:21 http://azure.archive.ubuntu.com/ubuntu noble-backports/universe amd64 Packages [30.2 kB]
Get:22 http://azure.archive.ubuntu.com/ubuntu noble-backports/universe Translation-en [17.4 kB]
Get:23 http://azure.archive.ubuntu.com/ubuntu noble-backports/universe amd64 Components [19.2 kB]
Get:24 http://azure.archive.ubuntu.com/ubuntu noble-backports/restricted amd64 Components [216 B]
Get:25 http://azure.archive.ubuntu.com/ubuntu noble-backports/multiverse amd64 Components [212 B]
Get:26 http://azure.archive.ubuntu.com/ubuntu noble-security/main amd64 Packages [1060 kB]
Get:27 http://azure.archive.ubuntu.com/ubuntu noble-security/main Translation-en [184 kB]
Get:28 http://azure.archive.ubuntu.com/ubuntu noble-security/main amd64 Components [21.6 kB]
Get:29 http://azure.archive.ubuntu.com/ubuntu noble-security/universe amd64 Packages [879 kB]
Get:30 http://azure.archive.ubuntu.com/ubuntu noble-security/universe Translation-en [194 kB]
Get:31 http://azure.archive.ubuntu.com/ubuntu noble-security/universe amd64 Components [52.3 kB]
Get:32 http://azure.archive.ubuntu.com/ubuntu noble-security/restricted amd64 Packages [1572 kB]
Get:33 http://azure.archive.ubuntu.com/ubuntu noble-security/restricted Translation-en [346 kB]
Get:34 http://azure.archive.ubuntu.com/ubuntu noble-security/restricted amd64 Components [212 B]
Get:35 http://azure.archive.ubuntu.com/ubuntu noble-security/multiverse amd64 Components [208 B]
Fetched 10.4 MB in 1s (7656 kB/s)
Reading package lists...
Reading package lists...
Building dependency tree...
Reading state information...
E: Unable to locate package pkg-config-aarch64-linux-gnu
E: Unable to locate package libssl-dev:arm64
Error: Process completed with exit code 100.

Codecov fails with this:
==> linux OS detected
https://uploader.codecov.io/latest/linux/codecov.SHA256SUM
==> SHASUM file signed by key id 806bb28aed779869
==> Uploader SHASUM verified (b37359013b48fbc3b0790d59fc474a52a260fb96e28e1b2c2ae001dc9b9cc996  codecov)
==> Running version latest
==> Running version v0.8.0
/home/runner/work/_actions/codecov/codecov-action/v3/dist/codecov -n  -Q github-action-3.1.6 -Z -f lcov.info
[2025-08-16T22:45:39.386Z] ['info'] 
     _____          _
    / ____|        | |
   | |     ___   __| | ___  ___ _____   __
   | |    / _ \ / _` |/ _ \/ __/ _ \ \ / /
   | |___| (_) | (_| |  __/ (_| (_) \ V /
    \_____\___/ \__,_|\___|\___\___/ \_/

  Codecov report uploader 0.8.0
[2025-08-16T22:45:39.393Z] ['info'] => Project root located at: /home/runner/work/pyst/pyst
[2025-08-16T22:45:39.394Z] ['info'] -> No token specified or token is empty
[2025-08-16T22:45:39.404Z] ['info'] Searching for coverage files...
[2025-08-16T22:45:39.473Z] ['info'] => Found 1 possible coverage files:
  lcov.info
[2025-08-16T22:45:39.473Z] ['info'] Processing /home/runner/work/pyst/pyst/lcov.info...
[2025-08-16T22:45:39.478Z] ['info'] Detected GitHub Actions as the CI provider.
[2025-08-16T22:45:39.975Z] ['info'] Pinging Codecov: https://codecov.io/upload/v4?package=github-action-3.1.6-uploader-0.8.0&token=*******&branch=main&build=17013708603&build_url=https%3A%2F%2Fgithub.com%2FRileyLeff%2Fpyst%2Factions%2Fruns%2F17013708603&commit=a7f7d5c699b9c713cd9fb6d45f34bfbb059af820&job=CI&pr=&service=github-actions&slug=RileyLeff%2Fpyst&name=&tag=&flags=&parent=
[2025-08-16T22:45:40.244Z] ['error'] There was an error running the uploader: Error uploading to https://codecov.io: Error: There was an error fetching the storage URL during POST: 429 - {"message":"Rate limit reached. Please upload with the Codecov repository upload token to resolve issue. Expected time to availability: 2029s."}

Error: Codecov: Failed to properly upload: The process '/home/runner/work/_actions/codecov/codecov-action/v3/dist/codecov' failed with exit code 255

which honestly to me looks like I might need to do something with a token on my end. please let me know if that's the case.