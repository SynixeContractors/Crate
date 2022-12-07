#include "script_component.hpp"
#include "script_component.hpp"

params [
    ["_items", createHashMap, [createHashMap]]
];

private _invalid = [];

{
    private _need = _y - ([_x] call EFUNC(gear_locker,owned));
    if (_need > 0) then {
        if (([_x] call FUNC(item_price)) == -1) exitWith {
            _invalid pushBack _x;
        };
    };
} forEach _items;

_invalid
