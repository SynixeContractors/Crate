#include "script_component.hpp"
ADDON = false;
#include "XEH_PREP.hpp"
ADDON = true;

if !(isMultiplayer) exitWith {};

GVAR(loadout_tracking) = false;

[QEGVAR(discord,updatedId), {
    if !(GVAR(enabled)) exitWith {};
    [QGVAR(loadout_get),[
        player getVariable [QEGVAR(discord,id), ""],
        getPlayerUID player
    ]] call CBA_fnc_serverEvent;
}] call CBA_fnc_addEventHandler;

[QGVAR(loadout_set), {
    params ["_loadout"];
    [player, _loadout, false] call CBA_fnc_setLoadout;
    [{
        if (count _loadout == 2) then {
            _loadout = _loadout select 0;
        };
        player addGoggles (_loadout select 7);
        player setVariable [QGVAR(loaded), true, true];
        GVAR(loadout_tracking) = true;
    }] call CBA_fnc_execNextFrame;
}] call CBA_fnc_addEventHandler;
