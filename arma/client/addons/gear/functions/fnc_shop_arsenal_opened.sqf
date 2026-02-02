#include "script_component.hpp"

params ["_display"];

if !(GVAR(enabled)) exitWith {};

if (ace_player isNotEqualTo player) exitWith {};
if !(player getVariable [QGVAR(shop_open), false]) exitWith {};

GVAR(shop_balanceHandle) = [FUNC(shop_pfh_balance), 0.2, [_display]] call CBA_fnc_addPerFrameHandler;

private _roleComboCtrl = _display displayCtrl IDC_leftRoleFilter;

{
    private _index = _roleComboCtrl lbAdd _y;
    _roleComboCtrl lbSetData [_index, _x];
} forEach EGVAR(discord,roles);
