#include "script_component.hpp"

if !(EGVAR(gear_main,enabled)) exitWith {};
if (EGVAR(gear_main,read_only)) exitWith {};

[player, GVAR(preLoadout), false] call CBA_fnc_setLoadout;

[QGVAR(store), [player, GVAR(preLoadout)]] call CBA_fnc_serverEvent;

[{
    GVAR(tracking) = true;
}] call CBA_fnc_execNextFrame;
