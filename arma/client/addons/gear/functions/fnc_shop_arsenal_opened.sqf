#include "script_component.hpp"

params ["_display"];

if !(GVAR(enabled)) exitWith {};

if (ace_player isNotEqualTo player) exitWith {};
if !(player getVariable [QGVAR(shop_open), false]) exitWith {};

GVAR(shop_balanceHandle) = [FUNC(shop_pfh_balance), 0.2, [_display]] call CBA_fnc_addPerFrameHandler;

private _leftRoleFilterCtrl = _display displayCtrl IDC_leftRoleFilter;
private _rightRoleFilterCtrl = _display displayCtrl IDC_rightRoleFilter;

GVAR(shop_changingRoles) = true;

private _index = _leftRoleFilterCtrl lbAdd "All Roles";
_leftRoleFilterCtrl lbSetData [_index, "all"];
_leftRoleFilterCtrl lbSetCurSel 0;
private _index = _rightRoleFilterCtrl lbAdd "All Roles";
_rightRoleFilterCtrl lbSetData [_index, "all"];
_rightRoleFilterCtrl lbSetCurSel 0;

GVAR(shop_currentRoles) = (player getVariable [QEGVAR(discord,roles), []]) arrayIntersect (keys GVAR(shop_roles));

{
    private _name = EGVAR(discord,roles) getOrDefault [_x, _x];
    private _index = _leftRoleFilterCtrl lbAdd _name;
    _leftRoleFilterCtrl lbSetData [_index, _x];
    private _index = _rightRoleFilterCtrl lbAdd _name;
    _rightRoleFilterCtrl lbSetData [_index, _x];
} forEach GVAR(shop_currentRoles);

GVAR(shop_changingRoles) = false;
