#include "script_component.hpp"

if !(GVAR(enabled)) exitWith {};
if (ace_player isNotEqualTo player) exitWith {};
if !(player getVariable [QGVAR(shop_open), false]) exitWith {};

params ["_display"];
private _ctrlPanel = _display displayCtrl IDC_leftTabContent;

private _loadout = [player] call CBA_fnc_getLoadout;
private _items = [_loadout] call FUNC(loadout_items);

for "_lbIndex" from 0 to (lbSize _ctrlPanel - 1) do {
    private _raw_class = _ctrlPanel lbData _lbIndex;
    if (_raw_class isNotEqualTo "") then {
        ([_items, _raw_class] call FUNC(shop_arsenal_tooltip)) params ["_tooltip", "_color"];
        _ctrlPanel lbSetTooltip [_lbIndex, _tooltip];
        _ctrlPanel lbSetColor [_lbIndex, _color];
    };
};

[{
    _this call FUNC(shop_arsenal_roleFilter_apply);
}, [_ctrlPanel, true]] call CBA_fnc_execNextFrame;
