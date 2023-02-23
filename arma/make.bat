cd .\client
hemtt %1
cd ..\server
cargo build --release
copy ..\..\target\release\crate_server.dll .\crate_server_x64.dll
hemtt %1
