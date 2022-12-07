#include "script_component.hpp"

params ["_items"];

{
    _items set [_x, _y - ([_x] call FUNC(owned))];
} forEach _items;

_items
