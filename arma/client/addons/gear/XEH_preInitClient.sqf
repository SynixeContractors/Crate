#include "script_component.hpp"
ADDON = false;
#include "XEH_PREP.hpp"
ADDON = true;

[QGVAR(loadout_set), {
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
