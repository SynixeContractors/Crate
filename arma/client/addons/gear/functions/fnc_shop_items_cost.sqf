#include "script_component.hpp"

/// Mode
/// 0 - Personal Cost
/// 1 - Company Cost
/// 2 - Both
params [
    ["_items", createHashMap, [createHashMap]],
    ["_mode", 2, [0]],
    ["_newOnly", true, [true]]
];

private _cost = 0;

{
    // (desired) - (already owned)
    private _need = if (GVAR(readOnly) || EGVAR(campaigns,loadouts) || !_newOnly) then { _y } else { _y - ([_x] call FUNC(shop_item_owned)) };
    if (_need > 0) then {
        private _class = [_x] call FUNC(shop_item_listing);
        ([_class, false] call FUNC(shop_item_price)) params ["_personal", "_company"];
        private _price = switch (_mode) do {
            case 0: {
                _personal
            };
            case 1: {
                _company
            };
            case 2: {
                _personal + _company
            };
            default {
                0
            };
        };
        _cost = _cost + (_price * _need);
    };
} forEach _items;

_cost
