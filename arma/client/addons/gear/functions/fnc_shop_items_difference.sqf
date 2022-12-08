#include "script_component.hpp"

params [
    ["_items", createHashMap, [createHashMap]]
];

private _ret = [];

{
    // (desired) - (already owned)
    private _need = _y - ([_x] call FUNC(shop_item_owned));
    if (_need > 0) then {
        private _class = [_x] call FUNC(shop_item_listing);
        private _price = [_class, false] call FUNC(shop_item_price);
        _ret pushBack [_x, _price#1, _need, _price#1 * _need, _price#2]
    };
} forEach _items;

_ret
