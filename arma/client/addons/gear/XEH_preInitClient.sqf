#include "script_component.hpp"

if !(isMultiplayer) exitWith {};

GVAR(loadout_tracking) = false;

[QGVAR(loadout_set), {
    params ["_loadout"];
    [player, _loadout, false] call CBA_fnc_setLoadout;
    [{
        if (count _loadout == 2) then {
            _loadout = _loadout select 0;
        };
        player addGoggles (_loadout select 7);
        GVAR(loadout_tracking) = true;
    }] call CBA_fnc_execNextFrame;
}] call CBA_fnc_addEventHandler;

[QGVAR(loadout_track), {
    [{
        GVAR(loadout_tracking) = true;
    }] call CBA_fnc_execNextFrame;
}] call CBA_fnc_addEventHandler;

[QGVAR(loadout_get_err), {
    systemChat "Error getting loadout from server, your loadout will not be tracked.";
}] call CBA_fnc_addEventHandler;

[QGVAR(loadout_store_err), {
    ERROR("Error storing loadout on server");
}] call CBA_fnc_addEventHandler;

[QEGVAR(discord,updatedId), {
    if !(GVAR(enabled)) exitWith {};
    [QGVAR(loadout_get),[
        player getVariable [QEGVAR(discord,id), ""],
        getPlayerUID player
    ]] call CBA_fnc_serverEvent;
}] call CBA_fnc_addEventHandler;
