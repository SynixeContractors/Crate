#include "script_component.hpp"

if !(GVAR(enabled)) exitWith {};
if (GVAR(readOnly)) exitWith {};

if !(GVAR(loadout_tracking)) exitWith {};
if (player getVariable [QGVAR(shop_open), false]) exitWith {};

[QGVAR(loadout_store), [player, [player] call CBA_fnc_getLoadout]] call CBA_fnc_serverEvent;
