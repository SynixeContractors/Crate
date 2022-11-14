#include "script_component.hpp"

if !(EGVAR(gear_main,enabled)) exitWith {};
if (EGVAR(gear_main,read_only)) exitWith {};

if !(GVAR(tracking)) exitWith {};
if (player getVariable [QGVAR(gear_shop,open), false]) exitWith {};

[QGVAR(store), [player, [player] call CBA_fnc_getLoadout]] call CBA_fnc_serverEvent;
