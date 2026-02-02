#include "script_component.hpp"

params ["_ctrl", "_index"];

if (GVAR(shop_changingRoles)) exitWith {};

private _display = ctrlParent _ctrl;
private _leftRoleFilterCtrl = _display displayCtrl IDC_leftRoleFilter;
private _rightRoleFilterCtrl = _display displayCtrl IDC_rightRoleFilter;

GVAR(shop_changingRoles) = true;

_leftRoleFilterCtrl lbSetCurSel _index;
_rightRoleFilterCtrl lbSetCurSel _index;

[] call ace_arsenal_fnc_refresh;

GVAR(shop_changingRoles) = false;
