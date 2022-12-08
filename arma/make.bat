cd .\client
hemtt dev
cd ..\server
cargo build --release
copy ..\..\target\release\crate_server.dll .\crate_server_x64.dll
hemtt dev
