#include "script_component.hpp"

params ["_items"];

{
    private _need = _y - ([_x] call FUNC(shop_item_owned));
    if (_need > 0) then {
        _items set [_x, ];
    };
} forEach _items;

_items
