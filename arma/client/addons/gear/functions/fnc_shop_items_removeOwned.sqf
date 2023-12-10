#include "script_component.hpp"

params ["_items"];

private _missing = createHashMap;

{
    private _need = _y - ([_x] call FUNC(shop_item_owned));
    if (_need > 0) then {
        _missing set [_x, _need];
    };
} forEach _missing;

_missing
