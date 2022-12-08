#include "script_component.hpp"

/// Global Mode
/// 0 - Include only non global
/// 1 - Include only global
/// 2 - Include all items
params [
    ["_items", createHashMap, [createHashMap]],
    ["_globalMode", 2, [0]]
];

private _cost = 0;

{
    // (desired) - (already owned)
    private _need = _y - ([_x] call FUNC(shop_item_owned));
    if (_need > 0) then {
        private _class = [_x] call FUNC(shop_item_listing);
        ([_class, false] call FUNC(shop_item_price)) params ["_basePrice", "_currentPrice", "_global"];
        if (
            _globalMode == 2 || { _globalMode == (parseNumber _global) }
        ) then {
            _cost = _cost + (_currentPrice * _need);
        };
    };
} forEach _items;

_cost
