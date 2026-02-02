#include "script_component.hpp"

params ["_ctrl", "_left"];

private _display = ctrlParent _ctrl;
private _leftRoleFilterCtrl = _display displayCtrl IDC_leftRoleFilter;
private _currentRole = _leftRoleFilterCtrl lbData (lbCurSel _leftRoleFilterCtrl);

if (_currentRole == "all") exitWith {};

private _currentClass = ["", _ctrl lbData (lbCurSel _ctrl)] select _left;

private _size = if (_left ) then { lbSize _ctrl } else { lnbSize _ctrl select 0 };
for "_i" from _size - 1 to 0 step -1 do {
    private _class = if (_left) then { _ctrl lbData _i } else { _ctrl lnbData [_i, 0] };
    if (_class == _currentClass) then { continue; };
    private _roles = [_class] call FUNC(shop_item_roles);
    if (_roles find _currentRole == -1) then {
        if (_left) then {
            _ctrl lbDelete _i;
        } else {
            _ctrl lnbDeleteRow _i;
        }
    };
};

if !(_left) exitWith {};

if (_currentClass != "") then {
    for "_i" from 0 to (lbSize _ctrl) - 1 do {
        if ((_ctrl lbData _i) == _currentClass) exitWith {
            _ctrl lbSetCurSel _i;
        };
    };
} else {
    _ctrl lbSetCurSel -1;
};
