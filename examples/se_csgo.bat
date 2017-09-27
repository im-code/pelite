@echo OFF
set GAME_PATH=C:\Program Files (x86)\Steam\steamapps\common\Counter-Strike Global Offensive

cargo build --release

echo. && echo ---------------------------------------------------------------- && echo Interfaces && echo.

cargo run --release --example se_interfaces -- ^
	"%GAME_PATH%\bin\engine.dll" ^
	"%GAME_PATH%\bin\inputsystem.dll" ^
	"%GAME_PATH%\bin\materialsystem.dll" ^
	"%GAME_PATH%\bin\shaderapidx9.dll" ^
	"%GAME_PATH%\bin\vgui2.dll" ^
	"%GAME_PATH%\bin\vguimatsurface.dll" ^
	"%GAME_PATH%\bin\vphysics.dll" ^
	"%GAME_PATH%\bin\vstdlib.dll" ^
	"%GAME_PATH%\csgo\bin\client.dll" ^
	"%GAME_PATH%\csgo\bin\matchmaking.dll" ^
	"%GAME_PATH%\csgo\bin\server.dll"

echo. && echo ---------------------------------------------------------------- && echo Classes && echo.

cargo run --release --example se_classes -- "%GAME_PATH%\csgo\bin\client.dll"

echo. && echo ---------------------------------------------------------------- && echo Console variables && echo.

cargo run --release --example se_cvars -- ^
	"%GAME_PATH%\bin\engine.dll" ^
	"%GAME_PATH%\csgo\bin\client.dll" ^
	"%GAME_PATH%\csgo\bin\server.dll"

echo. && echo ---------------------------------------------------------------- && echo Networked variables && echo.

cargo run --release --example se_recvtables_csgo -- "%GAME_PATH%\csgo\bin\client.dll"

echo. && echo ---------------------------------------------------------------- && echo Datamaps && echo.

cargo run --release --example se_datamaps_csgo -- "%GAME_PATH%\csgo\bin\client.dll"

echo. && echo ---------------------------------------------------------------- && echo Weapondata && echo.

cargo run --release --example se_weapondata_csgo -- "%GAME_PATH%\csgo\bin\client.dll"
