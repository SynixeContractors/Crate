#include "script_component.hpp"

params ["_display"];

if !(GVAR(enabled)) exitWith {};
if (GVAR(read_only)) exitWith {};

if (ace_player isNotEqualTo player) exitWith {};
if !(player getVariable [QGVAR(shop_open), false]) exitWith {};

GVAR(shop_balanceHandle) = [FUNC(shop_pfh_balance), 0.2, [_display]] call CBA_fnc_addPerFrameHandler;
