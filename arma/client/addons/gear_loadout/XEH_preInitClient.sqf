#include "script_component.hpp"
ADDON = false;
#include "XEH_PREP.hpp"
ADDON = true;

if !(isMultiplayer) exitWith {};

GVAR(tracking) = false;

[QEGVAR(discord,updatedId), {
    if !(EGVAR(gear_main,enabled)) exitWith {};
    [QGVAR(get),[
        player getVariable [QEGVAR(discord,id), ""],
        getPlayerUID player
    ]] call CBA_fnc_serverEvent;
}] call CBA_fnc_addEventHandler;

[QGVAR(set), {
    params ["_loadout"];
    [player, _loadout, false] call CBA_fnc_setLoadout;
    [{
        if (count _loadout == 2) then {
            _loadout = _loadout select 0;
        };
        player addGoggles (_loadout select 7);
        player setVariable [QGVAR(loaded), true, true];
        GVAR(tracking) = true;
    }] call CBA_fnc_execNextFrame;
}] call CBA_fnc_addEventHandler;

[QGVAR(stored), {
    params ["_result"];
    switch (_result) do {
        case "ok": {
            [QEGVAR(gear_shop,loaderReady), QUOTE(ADDON)] call CBA_fnc_localEvent;
        };
        default {
            [QEGVAR(gear_shop,loaderError), QUOTE(ADDON)] call CBA_fnc_localEvent;
        };
    }
}] call CBA_fnc_addEventHandler;

[QEGVAR(gear_shop,opening), FUNC(shop_handle_opening)] call CBA_fnc_addEventHandler;
[QEGVAR(gear_shop,closing), FUNC(shop_handle_closing)] call CBA_fnc_addEventHandler;
[QEGVAR(gear_shop,reverting), FUNC(shop_handle_reverting)] call CBA_fnc_addEventHandler;
