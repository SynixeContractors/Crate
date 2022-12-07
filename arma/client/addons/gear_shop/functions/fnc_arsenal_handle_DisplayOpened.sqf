#include "script_component.hpp"

params ["_display"];

if !(EGVAR(gear_main,enabled)) exitWith {};
if (EGVAR(gear_main,read_only)) exitWith {};

if (ace_player isNotEqualTo player) exitWith {};
if !(player getVariable [QGVAR(open), false]) exitWith {};

player setVariable [QGVAR(inArsenal), true, true];

GVAR(balanceHandle) = [FUNC(pfh_balance), 0.2, [_display]] call CBA_fnc_addPerFrameHandler;
