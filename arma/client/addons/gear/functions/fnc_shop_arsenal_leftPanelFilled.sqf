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
        private _class = [_raw_class] call FUNC(shop_item_listing);
        private _price = [_class, false] call FUNC(shop_item_price);
        private _tooltip = if (GVAR(readOnly)) then {
            format ["%1\nPrice: %2\nGlobal: %3", _raw_class, _price#1, _price#2]
        } else {
            private _owned = [_raw_class] call FUNC(shop_item_owned);
            private _cost = _price#1;
            if (_price#1 > _price#0) then {
                _ctrlPanel lbSetColor [_lbIndex, [1, 0, 0, 1]];
                _cost = format ["%1\nRegular: %2", _cost, _price#0];
            };
            if (_owned > 0) then {
                _ctrlPanel lbSetColor [_lbIndex, [0, 1, 0, 1]];
            };
            if (_price#1 < _price#0) then {
                _ctrlPanel lbSetColor [_lbIndex, [0.25, 0.33, 1, 1]];
                _cost = format ["%1\nRegular: %2", _cost, _price#0];
            };
            private _equipped = _items getOrDefault [_raw_class, 0];
            format ["%1\nOwned: %2\nEquipped: %3\nPrice: %4\nGlobal: %5", _raw_class, _owned, _equipped, _cost, _price#2]
        };
        _ctrlPanel lbSetTooltip [_lbIndex, _tooltip];
    };
};
