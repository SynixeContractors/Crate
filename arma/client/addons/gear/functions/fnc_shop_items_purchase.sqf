#include "script_component.hpp"

if !(GVAR(enabled)) exitWith {};
if (GVAR(readOnly)) exitWith {};

params [
    ["_items", createHashMap, [createHashMap]]
];

GVAR(shop_processing) = true;

[QGVAR(shop_purchase), [player, _items]] call CBA_fnc_serverEvent;
