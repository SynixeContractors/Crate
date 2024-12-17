#include "script_component.hpp"

params [
    ["_class", "", [""]],
    ["_lookup", true, [true]]
];

if (_class == "") exitWith {[-1,-1,-1-1]};

if (_lookup) then {
    _class = [_class] call FUNC(shop_item_listing);
};

(GVAR(shop_items) getOrDefault [_class, [[],[-1,-1,-1-1]]]) select 1
