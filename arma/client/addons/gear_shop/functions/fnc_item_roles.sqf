#include "script_component.hpp"

params [
    ["_class", "", [""]],
    ["_lookup", true, [true]]
];

if (_class == "") exitWith {[]};

if (_lookup) then {
    _class = [_class] call FUNC(item_listing);
};

(GVAR(items) getOrDefault [_class, [-1,[],false]]) select 0
