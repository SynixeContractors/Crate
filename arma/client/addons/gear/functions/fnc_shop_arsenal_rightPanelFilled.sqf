#include "script_component.hpp"

if !(GVAR(enabled)) exitWith {};

params ["_display", "_leftIDC"]; // , "_rightIDC"];
if (_leftIDC in [IDC_buttonPrimaryWeapon, IDC_buttonSecondaryWeapon, IDC_buttonHandgun]) then {
    private _ctrlPanel = _display displayCtrl IDC_rightTabContent;
    private _loadout = [player] call CBA_fnc_getLoadout;
    private _items = [_loadout] call FUNC(loadout_items);
    for "_lbIndex" from 0 to (lbSize _ctrlPanel - 1) do {
        private _raw_class = configName ((_ctrlPanel lbData _lbIndex) call CBA_fnc_getItemConfig);
        if (_raw_class isNotEqualTo "") then {
            private _class = [_raw_class] call FUNC(shop_item_listing);
            private _price = [_class, false] call FUNC(shop_item_price);
            private _tooltip = if (EGVAR(common,readOnly)) then {
                format ["%1\nPrice: %2\nGlobal: %3", _raw_class, _price#1, _price#2]
            } else {
                private _owned = [_raw_class] call FUNC(shop_item_owned);
                if (_price#1 > _price#2) then {
                _ctrlPanel lbSetColor [_lbIndex, [1, 0, 0, 1]];
            };
            if (_owned > 0) then {
                _ctrlPanel lbSetColor [_lbIndex, [0, 1, 0, 1]];
            };
            if (_price#1 < _price#2) then {
                _ctrlPanel lbSetColor [_lbIndex, [0, 0, 1, 1]];
            };
                private _equipped = _items getOrDefault [_raw_class, 0];
                format ["%1\nOwned: %2\nEquipped: %3\nPrice: %4\nGlobal: %5", _raw_class, _owned, _equipped, _price#1, _price#2]
            };
            _ctrlPanel lbSetTooltip [_lbIndex, _tooltip];
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
        private _class = [_raw_class] call FUNC(shop_item_listing);
        private _price = [_class, false] call FUNC(shop_item_price);
        private _tooltip = if (EGVAR(common,readOnly)) then {
            format ["%1\nPrice: %2\nGlobal: %3", _raw_class, _price#1, _price#2]
        } else {
            private _owned = [_raw_class] call FUNC(shop_item_owned);
            if (_price#1 > _price#2) then {
                _ctrlPanel lnbSetColor [[_lbIndex, 1], [1, 0, 0, 1]];
            };
            if (_owned > 0) then {
                _ctrlPanel lnbSetColor [[_lbIndex, 1], [0, 1, 0, 1]];
            };
            if (_price#1 < _price#2) then {
                _ctrlPanel lnbSetColor [[_lbIndex, 1], [0, 0, 1, 1]];
            };
            private _equipped = _items getOrDefault [_raw_class, 0];
            format ["%1\nOwned: %2\nEquipped: %3\nPrice: %4\nGlobal: %5", _raw_class, _owned, _equipped, _price#1, _price#2]
        };
        _ctrlPanel lnbSetTooltip [[_lbIndex, 0], _tooltip];
    };
};
