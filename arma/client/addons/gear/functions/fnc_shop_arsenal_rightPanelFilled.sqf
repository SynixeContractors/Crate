#include "script_component.hpp"

if !(GVAR(enabled)) exitWith {};
if (ace_player isNotEqualTo player) exitWith {};
if !(player getVariable [QGVAR(shop_open), false]) exitWith {};

params ["_display", "_leftIDC"]; // , "_rightIDC"];
if (_leftIDC in [IDC_buttonPrimaryWeapon, IDC_buttonSecondaryWeapon, IDC_buttonHandgun]) then {
    private _ctrlPanel = _display displayCtrl IDC_rightTabContent;
    private _loadout = [player] call CBA_fnc_getLoadout;
    private _items = [_loadout] call FUNC(loadout_items);
    for "_lbIndex" from 0 to (lbSize _ctrlPanel - 1) do {
        private _raw_class = configName ((_ctrlPanel lbData _lbIndex) call CBA_fnc_getItemConfig);
        if (_raw_class isNotEqualTo "") then {
            ([_raw_class] call FUNC(shop_arsenal_tooltip)) params ["_tooltip", "_color"];
            _ctrlPanel lbSetTooltip [_lbIndex, _tooltip];
            _ctrlPanel lbSetColor [_lbIndex, _color];
        };
    };
};
if (_leftIDC in [IDC_buttonUniform, IDC_buttonVest, IDC_buttonBackpack]) then {
    private _ctrlPanel = _display displayCtrl IDC_rightTabContentListnBox;
    private _loadout = [player] call CBA_fnc_getLoadout;
    private _items = [_loadout] call FUNC(loadout_items);
    (lnbSize _ctrlPanel) params ["_rows", "_columns"];

    for "_lbIndex" from 0 to (_rows - 1) do {
        private _raw_class = _ctrlPanel lnbData [_lbIndex, 0];
        ([_raw_class] call FUNC(shop_arsenal_tooltip)) params ["_tooltip", "_color"];
        _ctrlPanel lnbSetTooltip [[_lbIndex, 0], _tooltip];
        _ctrlPanel lnbSetColor [[_lbIndex, 1], _color];
    };
};
